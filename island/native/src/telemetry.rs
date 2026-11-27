//! Live OS telemetry poller for `hime-island-native` (ADR-017 Phase 4).
//!
//! Polls Windows APIs on a dedicated background thread and exposes thread-safe
//! [`TelemetrySnapshot`] reads via [`TelemetryHandle`]:
//! - Local clock (`GetLocalTime`)
//! - Network throughput (`GetIfTable2`), Wi-Fi vs Ethernet detection (`WlanEnumInterfaces` +
//!   `GetIpForwardTable2`), and internet reachability (`NetworkInformation`)
//! - Battery status (`GetSystemPowerStatus`)
//! - Master audio volume & mute (`IMMDeviceEnumerator` + `IAudioEndpointVolume`)
//! - Active media playback via SMTC (`GlobalSystemMediaTransportControlsSessionManager`)
//! - Running top-level application windows (`EnumWindows` + `DwmGetWindowAttribute`)

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use windows::core::{BOOL, BSTR, GUID, HRESULT, Interface, PCWSTR, PWSTR};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Networking::Connectivity::{NetworkConnectivityLevel, NetworkInformation};
use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, HANDLE, HWND, LPARAM, PROPERTYKEY, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetBestInterface, GetIfTable2, GetIpForwardTable2, MIB_IF_ROW2, MIB_IF_TABLE2,
    MIB_IPFORWARD_ROW2, MIB_IPFORWARD_TABLE2,
};
use windows::Win32::NetworkManagement::WiFi::{
    dot11_BSS_type_infrastructure, wlan_interface_state_connected,
    wlan_intf_opcode_current_connection, WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory,
    WlanGetAvailableNetworkList, WlanOpenHandle, WlanQueryInterface, DOT11_SSID,
    WLAN_AVAILABLE_NETWORK, WLAN_AVAILABLE_NETWORK_CONNECTED, WLAN_AVAILABLE_NETWORK_HAS_PROFILE,
    WLAN_AVAILABLE_NETWORK_LIST, WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO,
    WLAN_INTERFACE_INFO_LIST, WLAN_INTERFACE_STATE,
};
use windows::Win32::Networking::WinSock::{ADDRESS_FAMILY, AF_INET, AF_INET6};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoSetProxyBlanket, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
    STGM_READ,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::Shell::{
    SHGetKnownFolderPath, Shell_NotifyIconGetRect, KF_FLAG_DEFAULT, NOTIFYICONIDENTIFIER,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, FindWindowExW, GetClassNameA, GetWindow, GetWindowLongW,
    GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    GWL_EXSTYLE, GW_OWNER, HWND_MESSAGE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

use crate::dock::DockItem;
use crate::notch::{BatteryStatus, MusicInfo, NetStats};

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(1000);
const UNKNOWN_U8: u8 = 255;
const NO_SYSTEM_BATTERY: u8 = 128;
const CHARGING_FLAG: u8 = 8;

/// Active primary network transport kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NetworkKind {
    #[default]
    Offline,
    Wifi,
    Ethernet,
}

/// Combined network connectivity, adapter identity, and throughput telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NetworkStatus {
    pub kind: NetworkKind,
    pub ssid: Option<String>,
    pub adapter_name: Option<String>,
    pub stats: NetStats,
}

/// Master audio output volume and mute state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VolumeStatus {
    pub percent: u8,
    pub muted: bool,
    pub device_name: Option<String>,
}

/// Discovered nearby Wi-Fi network returned by [`scan_available_wifi_networks`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WifiNetworkInfo {
    pub ssid: String,
    pub signal_percent: u8,
    pub secured: bool,
    pub connected: bool,
    pub saved: bool,
}

/// Metadata for a visible top-level application window suitable for [`DockBar`](crate::dock::DockBar).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunningWindow {
    pub hwnd: isize,
    pub title: String,
    pub app_name: String,
    pub executable: String,
    pub exe_path: String,
}

impl RunningWindow {
    /// Converts this running window into a [`DockItem`] marked as `running: true`.
    pub fn to_dock_item(&self) -> DockItem {
        let id = format!("{}:{}", self.executable.to_lowercase(), self.hwnd);
        let mut item = DockItem::new(id, self.app_name.clone(), self.executable.clone(), false);
        item.exe_path = self.exe_path.clone();
        item.hwnds = vec![self.hwnd];
        item.running = true;
        item
    }
}

/// Formats a human-friendly Dock label from an executable filename (e.g. `"msedge.exe"` -> `"Microsoft Edge"`).
pub fn friendly_app_name(executable: &str) -> String {
    let stem = executable
        .strip_suffix(".exe")
        .or_else(|| executable.strip_suffix(".EXE"))
        .unwrap_or(executable)
        .trim();
    if stem.is_empty() {
        return "App".to_string();
    }
    match stem.to_ascii_lowercase().as_str() {
        "msedge" => "Microsoft Edge".to_string(),
        "wt" | "windowsterminal" => "Terminal".to_string(),
        "explorer" => "Explorer".to_string(),
        "calculatorapp" | "calc" => "Calculator".to_string(),
        "systemsettings" => "Windows Settings".to_string(),
        "microsoft.photos" | "photosapp" => "Photos".to_string(),
        "windowsnotepad" | "notepad" => "Notepad".to_string(),
        "mspaint" => "Paint".to_string(),
        "snippingtool" | "screenclippinghost" => "Snipping Tool".to_string(),
        "soundrec" | "windowssoundrecorder" => "Sound Recorder".to_string(),
        "time" | "windowsalarms" => "Clock".to_string(),
        "applicationframehost" => "Windows App".to_string(),
        "googledrivefs" => "Google Drive".to_string(),
        "dcv2" => "MSI Center Pro".to_string(),
        "xmousebuttoncontrol" => "X-Mouse Button Control".to_string(),
        "securityhealthsystray" => "Windows Security".to_string(),
        _ => {
            let mut chars = stem.chars();
            match chars.next() {
                Some(first) if first.is_ascii_lowercase() => {
                    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
                }
                _ => stem.to_string(),
            }
        }
    }
}

/// Groups [`RunningWindow`] entries by `executable.to_lowercase()`, preserving first-seen order.
///
/// Multiple windows belonging to the same executable collapse into a single [`DockItem`]
/// with `hwnds` containing every window handle and `badge_count` set to the window count
/// when more than one window is open.
pub fn group_running_windows(windows: &[RunningWindow]) -> Vec<DockItem> {
    let mut grouped: Vec<DockItem> = Vec::new();
    let mut key_to_idx: HashMap<String, usize> = HashMap::new();

    for win in windows {
        let exe_key = win.executable.trim().to_lowercase();
        let key = if !exe_key.is_empty() {
            exe_key
        } else if !win.exe_path.trim().is_empty() {
            win.exe_path.trim().to_lowercase()
        } else {
            format!("hwnd:{}", win.hwnd)
        };

        if let Some(&idx) = key_to_idx.get(&key) {
            let item = &mut grouped[idx];
            if !item.hwnds.contains(&win.hwnd) {
                item.hwnds.push(win.hwnd);
            }
            if item.exe_path.is_empty() && !win.exe_path.is_empty() {
                item.exe_path = win.exe_path.clone();
            }
            item.badge_count = if item.hwnds.len() > 1 {
                item.hwnds.len() as u32
            } else {
                0
            };
        } else {
            let label = if win.app_name.trim().is_empty() {
                friendly_app_name(&win.executable)
            } else {
                win.app_name.clone()
            };
            let mut item = DockItem::new(key.clone(), label, win.executable.clone(), false);
            item.exe_path = win.exe_path.clone();
            item.hwnds = vec![win.hwnd];
            item.running = true;
            item.badge_count = 0;
            key_to_idx.insert(key, grouped.len());
            grouped.push(item);
        }
    }

    grouped
}

/// Point-in-time snapshot of all OS telemetry consumed by the native shell surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub hours: u8,
    pub minutes: u8,
    pub network: NetworkStatus,
    pub battery: BatteryStatus,
    pub volume: VolumeStatus,
    pub brightness: u8,
    pub available_wifi: Vec<WifiNetworkInfo>,
    pub tray_count: u32,
    pub music: Option<MusicInfo>,
    pub running_windows: Vec<RunningWindow>,
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        Self {
            hours: 0,
            minutes: 0,
            network: NetworkStatus::default(),
            battery: BatteryStatus::default(),
            volume: VolumeStatus::default(),
            brightness: 80,
            available_wifi: Vec::new(),
            tray_count: 0,
            music: None,
            running_windows: Vec::new(),
        }
    }
}

/// Background OS telemetry poller handle. Stops and joins cleanly on drop.
pub struct TelemetryHandle {
    snapshot: Arc<RwLock<TelemetrySnapshot>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl TelemetryHandle {
    /// Starts the background telemetry poller using the default 1s interval.
    pub fn start() -> Self {
        Self::start_with_interval(DEFAULT_POLL_INTERVAL)
    }

    /// Starts the background telemetry poller with a custom poll `interval`.
    pub fn start_with_interval(interval: Duration) -> Self {
        let snapshot = Arc::new(RwLock::new(TelemetrySnapshot::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_snapshot = Arc::clone(&snapshot);
        let worker_stop = Arc::clone(&stop);

        let worker = thread::Builder::new()
            .name("hime-native-telemetry".to_string())
            .spawn(move || {
                run_poller_loop(worker_snapshot, worker_stop, interval);
            })
            .ok();

        Self {
            snapshot,
            stop,
            worker,
        }
    }

    /// Returns a cloned copy of the latest [`TelemetrySnapshot`].
    pub fn snapshot(&self) -> TelemetrySnapshot {
        match self.snapshot.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Signals the background poller thread to exit and waits for it to join.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TelemetryHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Balances `CoInitializeEx` with `CoUninitialize` on the poller thread.
struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn new() -> Self {
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        Self { initialized }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

/// RAII wrapper for a WLAN client handle.
struct WlanClient(HANDLE);

impl WlanClient {
    fn open() -> Option<Self> {
        let mut version = 0u32;
        let mut handle = HANDLE::default();
        let rc = unsafe { WlanOpenHandle(2, None, &mut version, &mut handle) };
        if rc == 0 {
            Some(Self(handle))
        } else {
            None
        }
    }

    fn interfaces(&self) -> Vec<(GUID, WLAN_INTERFACE_STATE)> {
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let rc = unsafe { WlanEnumInterfaces(self.0, None, &mut list) };
        if rc != 0 || list.is_null() {
            return Vec::new();
        }
        let mut items = Vec::new();
        unsafe {
            if let Some(header) = list.as_ref() {
                let count = (header.dwNumberOfItems as usize).min(64);
                if count > 0 {
                    let entries: &[WLAN_INTERFACE_INFO] =
                        std::slice::from_raw_parts(header.InterfaceInfo.as_ptr(), count);
                    items.extend(entries.iter().map(|info| (info.InterfaceGuid, info.isState)));
                }
            }
            WlanFreeMemory(list as *const _);
        }
        items
    }
}

impl Drop for WlanClient {
    fn drop(&mut self) {
        unsafe {
            WlanCloseHandle(self.0, None);
        }
    }
}

fn run_poller_loop(
    snapshot_lock: Arc<RwLock<TelemetrySnapshot>>,
    stop: Arc<AtomicBool>,
    interval: Duration,
) {
    let _com = ComGuard::new();
    let mut prev_net: Option<(Instant, u64, u64)> = None;
    let enumerator: Option<IMMDeviceEnumerator> =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.ok();
    let mut smtc_mgr: Option<GlobalSystemMediaTransportControlsSessionManager> =
        GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|op| op.get())
            .ok();
    let mut wmi_services: Option<ComRaw> = unsafe { connect_wmi_services() };
    let mut poll_tick: u64 = 0;
    let mut cached_wifi: Vec<WifiNetworkInfo> = Vec::new();
    let mut cached_tray_count: u32 = 0;
    let mut cached_brightness: u8 = CACHED_BRIGHTNESS.load(Ordering::Relaxed);

    while !stop.load(Ordering::Relaxed) {
        let (hours, minutes) = read_local_clock();
        let network = read_network_status(&mut prev_net);
        let battery = read_battery_status();
        let volume = read_volume_status(enumerator.as_ref());
        if poll_tick % 3 == 0 {
            cached_brightness = read_brightness_with_wmi(&mut wmi_services);
        }
        let brightness = cached_brightness;

        if poll_tick % 15 == 0 {
            cached_wifi = scan_available_wifi_networks();
        }
        if poll_tick % 3 == 0 {
            cached_tray_count = read_tray_count();
        }
        poll_tick = poll_tick.wrapping_add(1);

        if smtc_mgr.is_none() {
            smtc_mgr = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .and_then(|op| op.get())
                .ok();
        }
        let music = read_smtc_music(&mut smtc_mgr);
        let running_windows = read_running_windows();

        let next = TelemetrySnapshot {
            hours,
            minutes,
            network,
            battery,
            volume,
            brightness,
            available_wifi: cached_wifi.clone(),
            tray_count: cached_tray_count,
            music,
            running_windows,
        };

        match snapshot_lock.write() {
            Ok(mut guard) => *guard = next,
            Err(poisoned) => *poisoned.into_inner() = next,
        }

        // Sleep in small slices so `stop()` returns promptly.
        let slice = Duration::from_millis(100);
        let mut waited = Duration::ZERO;
        while waited < interval && !stop.load(Ordering::Relaxed) {
            let step = (interval - waited).min(slice);
            thread::sleep(step);
            waited += step;
        }
    }
}

/// Reads the current local time as `(hours, minutes)`.
pub fn read_local_clock() -> (u8, u8) {
    let st = unsafe { GetLocalTime() };
    ((st.wHour % 24) as u8, (st.wMinute % 60) as u8)
}

/// Parses Win32 [`SYSTEM_POWER_STATUS`] into [`BatteryStatus`].
pub fn parse_system_power_status(status: &SYSTEM_POWER_STATUS) -> BatteryStatus {
    let present = status.BatteryFlag != UNKNOWN_U8 && (status.BatteryFlag & NO_SYSTEM_BATTERY) == 0;
    let charging = present && (status.BatteryFlag & CHARGING_FLAG) != 0;
    let percent = if present && status.BatteryLifePercent <= 100 {
        status.BatteryLifePercent
    } else if present {
        100
    } else {
        0
    };
    BatteryStatus {
        percent,
        charging,
        present,
    }
}

/// Queries the current OS battery status via `GetSystemPowerStatus`.
pub fn read_battery_status() -> BatteryStatus {
    let mut status = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() {
        return BatteryStatus::default();
    }
    parse_system_power_status(&status)
}

/// Computes throughput in kilobits per second (kbps) from byte counters and elapsed milliseconds.
///
/// Since `1 byte/ms = 8 bits/ms = 8000 bits/s = 8 kbps`, `delta_bytes * 8 / elapsed_ms`
/// yields exact kilobits per second.
pub fn compute_kbps(prev_bytes: u64, curr_bytes: u64, elapsed_ms: u128) -> f32 {
    if elapsed_ms == 0 || curr_bytes < prev_bytes {
        return 0.0;
    }
    let delta = (curr_bytes - prev_bytes) as f64;
    ((delta * 8.0) / elapsed_ms as f64) as f32
}

fn wide_slice_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

fn ssid_bytes(ssid: &DOT11_SSID) -> &[u8] {
    &ssid.ucSSID[..(ssid.uSSIDLength as usize).min(ssid.ucSSID.len())]
}

/// Returns `true` if a network adapter description or alias belongs to a virtual/loopback interface.
pub fn is_virtual_adapter(description: &str, alias: &str) -> bool {
    let d = description.to_ascii_lowercase();
    let a = alias.to_ascii_lowercase();
    const PATTERNS: &[&str] = &[
        "loopback",
        "virtual",
        "hyper-v",
        "vethernet",
        "vmware",
        "virtualbox",
        "wsl",
        "tailscale",
        "wireguard",
        "zerotier",
        "tap-windows",
        "wintun",
        "teredo",
        "isatap",
        "6to4",
        "wan miniport",
        "kernel debug",
    ];
    PATTERNS
        .iter()
        .any(|p| d.contains(p) || a.contains(p))
}

fn query_connected_wifi_ssid() -> Option<String> {
    let client = WlanClient::open()?;
    let ifaces = client.interfaces();
    for (guid, state) in ifaces {
        if state != wlan_interface_state_connected {
            continue;
        }
        let mut size = 0u32;
        let mut data: *mut std::ffi::c_void = std::ptr::null_mut();
        let rc = unsafe {
            WlanQueryInterface(
                client.0,
                &guid,
                wlan_intf_opcode_current_connection,
                None,
                &mut size,
                &mut data,
                None,
            )
        };
        let ssid = if rc == 0 && !data.is_null() {
            unsafe {
                (data as *const WLAN_CONNECTION_ATTRIBUTES)
                    .as_ref()
                    .filter(|attrs| attrs.isState == wlan_interface_state_connected)
                    .map(|attrs| {
                        String::from_utf8_lossy(ssid_bytes(
                            &attrs.wlanAssociationAttributes.dot11Ssid,
                        ))
                        .into_owned()
                    })
                    .filter(|s| !s.is_empty())
            }
        } else {
            None
        };
        if !data.is_null() {
            unsafe { WlanFreeMemory(data) };
        }
        if ssid.is_some() {
            return ssid;
        }
    }
    None
}

unsafe fn collect_default_gateway_indices(family: ADDRESS_FAMILY, out: &mut HashSet<u32>) {
    let mut forward_out = std::mem::MaybeUninit::<*mut MIB_IPFORWARD_TABLE2>::uninit();
    if !GetIpForwardTable2(family, forward_out.as_mut_ptr()).is_ok() {
        return;
    }
    if let Some(fwd_table) = std::ptr::NonNull::new(forward_out.assume_init()) {
        let fwd_ptr = fwd_table.as_ptr();
        let rows = std::slice::from_raw_parts(
            std::ptr::addr_of!((*fwd_ptr).Table).cast::<MIB_IPFORWARD_ROW2>(),
            fwd_table.as_ref().NumEntries as usize,
        );
        for fwd in rows {
            if fwd.DestinationPrefix.PrefixLength == 0 && !fwd.Loopback {
                out.insert(fwd.InterfaceIndex);
            }
        }
        FreeMibTable(fwd_ptr as *const _);
    }
}

fn check_internet_access() -> bool {
    let winrt_ok = (|| -> Option<bool> {
        let profile = NetworkInformation::GetInternetConnectionProfile().ok()?;
        let level = profile.GetNetworkConnectivityLevel().ok()?;
        Some(level == NetworkConnectivityLevel::InternetAccess)
    })();
    winrt_ok.unwrap_or(false)
}

fn read_network_status(prev_sample: &mut Option<(Instant, u64, u64)>) -> NetworkStatus {
    let mut total_in = 0u64;
    let mut total_out = 0u64;
    let mut eth_name: Option<String> = None;

    let mut gateway_if_indices = HashSet::new();
    unsafe {
        collect_default_gateway_indices(AF_INET, &mut gateway_if_indices);
        collect_default_gateway_indices(AF_INET6, &mut gateway_if_indices);
    }

    let mut best_idx = 0u32;
    let best_if_index = if unsafe { GetBestInterface(0x08080808, &mut best_idx) } == 0 {
        Some(best_idx)
    } else {
        None
    };

    let wifi_ssid = query_connected_wifi_ssid();
    let wifi_connected = wifi_ssid.is_some();

    unsafe {
        let mut out = std::mem::MaybeUninit::<*mut MIB_IF_TABLE2>::uninit();
        if GetIfTable2(out.as_mut_ptr()).is_ok() {
            if let Some(table) = std::ptr::NonNull::new(out.assume_init()) {
                let table_ptr = table.as_ptr();
                let rows = std::slice::from_raw_parts(
                    std::ptr::addr_of!((*table_ptr).Table).cast::<MIB_IF_ROW2>(),
                    table.as_ref().NumEntries as usize,
                );
                for row in rows {
                    total_in = total_in.saturating_add(row.InOctets);
                    total_out = total_out.saturating_add(row.OutOctets);

                    // IF_TYPE_ETHERNET_CSMACD = 6, IfOperStatusUp = 1, MediaConnectStateConnected = 1
                    let is_hw = (row.InterfaceAndOperStatusFlags._bitfield & 0x01) != 0;
                    if row.Type == 6
                        && row.OperStatus.0 == 1
                        && row.MediaConnectState.0 == 1
                        && row.TunnelType.0 == 0
                        && is_hw
                        && row.PhysicalAddressLength > 0
                        && gateway_if_indices.contains(&row.InterfaceIndex)
                    {
                        let desc = wide_slice_to_string(&row.Description);
                        let alias = wide_slice_to_string(&row.Alias);
                        if !is_virtual_adapter(&desc, &alias)
                            && (!wifi_connected || best_if_index == Some(row.InterfaceIndex))
                        {
                            let trimmed = alias.trim();
                            let name = if trimmed.is_empty() {
                                "Ethernet".to_string()
                            } else {
                                trimmed.to_string()
                            };
                            if best_if_index == Some(row.InterfaceIndex) || eth_name.is_none() {
                                eth_name = Some(name);
                            }
                        }
                    }
                }
                FreeMibTable(table_ptr as *const _);
            }
        }
    }

    let now = Instant::now();
    let (down_kbps, up_kbps) = if let Some((prev_time, prev_in, prev_out)) = *prev_sample {
        let elapsed_ms = now.duration_since(prev_time).as_millis().max(1);
        (
            compute_kbps(prev_in, total_in, elapsed_ms),
            compute_kbps(prev_out, total_out, elapsed_ms),
        )
    } else {
        (0.0, 0.0)
    };
    *prev_sample = Some((now, total_in, total_out));

    let (kind, adapter_name) = if let Some(name) = eth_name {
        (NetworkKind::Ethernet, Some(name))
    } else if wifi_connected {
        (NetworkKind::Wifi, wifi_ssid.clone())
    } else {
        (NetworkKind::Offline, None)
    };

    let has_internet = kind != NetworkKind::Offline || check_internet_access();

    NetworkStatus {
        kind,
        ssid: wifi_ssid,
        adapter_name,
        stats: NetStats {
            down_kbps,
            up_kbps,
            is_internet_down: !has_internet,
        },
    }
}

/// Sanitizes an audio endpoint name, falling back to `"Audio Output"` when empty or a raw GUID.
pub fn sanitize_audio_device_name(raw: Option<&str>) -> String {
    match raw.map(str::trim) {
        Some(s) if !s.is_empty() && !s.starts_with('{') => s.to_string(),
        _ => "Audio Output".to_string(),
    }
}

fn read_volume_status(enumerator: Option<&IMMDeviceEnumerator>) -> VolumeStatus {
    let Some(enum_ref) = enumerator else {
        return VolumeStatus::default();
    };
    let Ok(device) = (unsafe { enum_ref.GetDefaultAudioEndpoint(eRender, eConsole) }) else {
        return VolumeStatus::default();
    };
    let Ok(endpoint) = (unsafe { device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) })
    else {
        return VolumeStatus::default();
    };

    const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
        fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
        pid: 14,
    };

    let raw_friendly: Option<String> = unsafe {
        device.OpenPropertyStore(STGM_READ).ok().and_then(|store| {
            let mut prop: PROPVARIANT = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME).ok()?;
            let name = if prop.Anonymous.Anonymous.vt == VT_LPWSTR {
                prop.Anonymous.Anonymous.Anonymous.pwszVal.to_string().ok()
            } else {
                None
            };
            let _ = PropVariantClear(&mut prop);
            name
        })
    };
    let device_name = Some(sanitize_audio_device_name(raw_friendly.as_deref()));

    let scalar = unsafe { endpoint.GetMasterVolumeLevelScalar() }.unwrap_or(0.0);
    let muted: bool = unsafe { endpoint.GetMute() }
        .map(Into::into)
        .unwrap_or(false);

    VolumeStatus {
        percent: (scalar.clamp(0.0, 1.0) * 100.0).round() as u8,
        muted,
        device_name,
    }
}

/// Clamps a normalized master volume scalar to `[0.0, 1.0]` (treating NaN as `0.0`).
pub fn clamp_volume_scalar(level: f32) -> f32 {
    if !level.is_finite() {
        0.0
    } else {
        level.clamp(0.0, 1.0)
    }
}

/// Clamps a display brightness percentage to `0..=100`.
pub fn clamp_brightness_percent(percent: u8) -> u8 {
    percent.min(100)
}

struct ThreadComGuard {
    should_uninit: bool,
}

impl ThreadComGuard {
    fn enter() -> Self {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        Self {
            should_uninit: hr.is_ok(),
        }
    }
}

impl Drop for ThreadComGuard {
    fn drop(&mut self) {
        if self.should_uninit {
            unsafe { CoUninitialize() };
        }
    }
}

fn default_audio_endpoint() -> std::result::Result<IAudioEndpointVolume, String> {
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("MMDeviceEnumerator failed: {e}"))?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| format!("GetDefaultAudioEndpoint failed: {e}"))?;
        device
            .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
            .map_err(|e| format!("IAudioEndpointVolume failed: {e}"))
    }
}

/// Sets the system master output volume using a normalized scalar in `0.0..=1.0`.
pub fn set_master_volume(level: f32) -> std::result::Result<(), String> {
    let _com = ThreadComGuard::enter();
    let endpoint = default_audio_endpoint()?;
    let clamped = clamp_volume_scalar(level);
    unsafe {
        endpoint
            .SetMasterVolumeLevelScalar(clamped, std::ptr::null())
            .map_err(|e| format!("SetMasterVolumeLevelScalar failed: {e}"))
    }
}

/// Convenience wrapper setting master output volume as an integer percentage (`0..=100`).
pub fn set_master_volume_percent(percent: u8) -> std::result::Result<(), String> {
    set_master_volume(percent.min(100) as f32 / 100.0)
}

/// Toggles master output mute and returns the resulting mute state (`true` = muted).
pub fn toggle_mute() -> std::result::Result<bool, String> {
    let _com = ThreadComGuard::enter();
    let endpoint = default_audio_endpoint()?;
    unsafe {
        let currently_muted: bool = endpoint
            .GetMute()
            .map(Into::into)
            .map_err(|e| format!("GetMute failed: {e}"))?;
        let next_muted = !currently_muted;
        endpoint
            .SetMute(next_muted, std::ptr::null())
            .map_err(|e| format!("SetMute failed: {e}"))?;
        Ok(next_muted)
    }
}

// ---------------------------------------------------------------------------
// System Brightness (WMI `WmiMonitorBrightness` + DXVA2 `dxva2.dll` + Cache)
// ---------------------------------------------------------------------------

static CACHED_BRIGHTNESS: AtomicU8 = AtomicU8::new(80);

#[repr(C)]
#[derive(Clone, Copy)]
union RawVariantData {
    b_val: u8,
    i_val: i16,
    l_val: i32,
    ui_val: u16,
    ul_val: u32,
    bstr_val: *const u16,
    _pad: [u64; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawVariant {
    vt: u16,
    w_reserved1: u16,
    w_reserved2: u16,
    w_reserved3: u16,
    data: RawVariantData,
}

impl Default for RawVariant {
    fn default() -> Self {
        Self {
            vt: 0,
            w_reserved1: 0,
            w_reserved2: 0,
            w_reserved3: 0,
            data: RawVariantData { _pad: [0; 2] },
        }
    }
}

#[link(name = "oleaut32")]
extern "system" {
    fn VariantClear(pvarg: *mut RawVariant) -> HRESULT;
}

struct ComRaw(*mut c_void);

impl ComRaw {
    unsafe fn vtable(&self) -> *const *const c_void {
        *(self.0 as *const *const *const c_void)
    }
}

impl Drop for ComRaw {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                    std::mem::transmute(*self.vtable().add(2));
                release(self.0);
            }
        }
    }
}

#[inline]
unsafe fn bstr_raw(b: &BSTR) -> *const u16 {
    *(b as *const BSTR as *const *const u16)
}

unsafe fn connect_wmi_services() -> Option<ComRaw> {
    const CLSID_WBEM_LOCATOR: GUID = GUID::from_u128(0x4590f811_1d3a_11d0_891f_00aa004b2e24);
    const IID_IWEM_LOCATOR: GUID = GUID::from_u128(0xdc12a687_737f_11cf_884d_00aa004b2e24);

    let locator_unk: windows::core::IUnknown =
        CoCreateInstance(&CLSID_WBEM_LOCATOR, None, CLSCTX_ALL).ok()?;
    let mut locator_ptr: *mut c_void = std::ptr::null_mut();
    if locator_unk.query(&IID_IWEM_LOCATOR, &mut locator_ptr).is_err() || locator_ptr.is_null() {
        return None;
    }
    let locator = ComRaw(locator_ptr);

    let ns = BSTR::from("root\\WMI");
    let empty = BSTR::new();
    let mut services_ptr: *mut c_void = std::ptr::null_mut();
    let connect_server: unsafe extern "system" fn(
        *mut c_void,
        *const u16,
        *const u16,
        *const u16,
        *const u16,
        i32,
        *const u16,
        *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT = std::mem::transmute(*locator.vtable().add(3));
    let hr = connect_server(
        locator.0,
        bstr_raw(&ns),
        bstr_raw(&empty),
        bstr_raw(&empty),
        bstr_raw(&empty),
        0,
        bstr_raw(&empty),
        std::ptr::null_mut(),
        &mut services_ptr,
    );
    if hr.is_err() || services_ptr.is_null() {
        return None;
    }
    let services = ComRaw(services_ptr);

    if let Some(svc_unk) = windows::core::IUnknown::from_raw_borrowed(&services.0) {
        let _ = CoSetProxyBlanket(
            svc_unk,
            10, // RPC_C_AUTHN_WINNT
            0,  // RPC_C_AUTHZ_NONE
            PCWSTR::null(),
            RPC_C_AUTHN_LEVEL_CALL,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
        );
    }

    Some(services)
}

unsafe fn wmi_read_brightness() -> Option<u8> {
    let services = connect_wmi_services()?;
    wmi_read_brightness_with_services(&services)
}

unsafe fn wmi_read_brightness_with_services(services: &ComRaw) -> Option<u8> {
    let wql = BSTR::from("WQL");
    let query = BSTR::from("SELECT CurrentBrightness FROM WmiMonitorBrightness");
    let mut enum_ptr: *mut c_void = std::ptr::null_mut();
    let exec_query: unsafe extern "system" fn(
        *mut c_void,
        *const u16,
        *const u16,
        i32,
        *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT = std::mem::transmute(*services.vtable().add(20));
    if exec_query(
        services.0,
        bstr_raw(&wql),
        bstr_raw(&query),
        0,
        std::ptr::null_mut(),
        &mut enum_ptr,
    )
    .is_err()
        || enum_ptr.is_null()
    {
        return None;
    }
    let enum_obj = ComRaw(enum_ptr);

    let mut obj_ptr: *mut c_void = std::ptr::null_mut();
    let mut returned = 0u32;
    let next_fn: unsafe extern "system" fn(
        *mut c_void,
        i32,
        u32,
        *mut *mut c_void,
        *mut u32,
    ) -> HRESULT = std::mem::transmute(*enum_obj.vtable().add(4));
    if next_fn(enum_obj.0, -1, 1, &mut obj_ptr, &mut returned).is_err()
        || returned == 0
        || obj_ptr.is_null()
    {
        return None;
    }
    let obj = ComRaw(obj_ptr);

    let mut var = RawVariant::default();
    let get_fn: unsafe extern "system" fn(
        *mut c_void,
        PCWSTR,
        i32,
        *mut RawVariant,
        *mut i32,
        *mut i32,
    ) -> HRESULT = std::mem::transmute(*obj.vtable().add(4));
    let hr = get_fn(
        obj.0,
        windows::core::w!("CurrentBrightness"),
        0,
        &mut var,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if hr.is_err() {
        return None;
    }

    let val = match var.vt {
        17 => Some(var.data.b_val),                   // VT_UI1
        2 => Some(var.data.i_val.clamp(0, 100) as u8), // VT_I2
        3 => Some(var.data.l_val.clamp(0, 100) as u8), // VT_I4
        18 => Some(var.data.ui_val.min(100) as u8),    // VT_UI2
        19 => Some(var.data.ul_val.min(100) as u8),    // VT_UI4
        _ => None,
    };
    let _ = VariantClear(&mut var);
    val.map(clamp_brightness_percent)
}

unsafe fn wmi_set_brightness(percent: u8) -> bool {
    let Some(services) = connect_wmi_services() else {
        return false;
    };

    let class_name = BSTR::from("WmiMonitorBrightnessMethods");
    let mut class_ptr: *mut c_void = std::ptr::null_mut();
    let get_object: unsafe extern "system" fn(
        *mut c_void,
        *const u16,
        i32,
        *mut c_void,
        *mut *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT = std::mem::transmute(*services.vtable().add(6));
    if get_object(
        services.0,
        bstr_raw(&class_name),
        0,
        std::ptr::null_mut(),
        &mut class_ptr,
        std::ptr::null_mut(),
    )
    .is_err()
        || class_ptr.is_null()
    {
        return false;
    }
    let class_def = ComRaw(class_ptr);

    let mut in_cls_ptr: *mut c_void = std::ptr::null_mut();
    let get_method: unsafe extern "system" fn(
        *mut c_void,
        PCWSTR,
        i32,
        *mut *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT = std::mem::transmute(*class_def.vtable().add(19));
    if get_method(
        class_def.0,
        windows::core::w!("WmiSetBrightness"),
        0,
        &mut in_cls_ptr,
        std::ptr::null_mut(),
    )
    .is_err()
        || in_cls_ptr.is_null()
    {
        return false;
    }
    let in_cls = ComRaw(in_cls_ptr);

    let wql = BSTR::from("WQL");
    let query = BSTR::from("SELECT * FROM WmiMonitorBrightnessMethods");
    let mut enum_ptr: *mut c_void = std::ptr::null_mut();
    let exec_query: unsafe extern "system" fn(
        *mut c_void,
        *const u16,
        *const u16,
        i32,
        *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT = std::mem::transmute(*services.vtable().add(20));
    if exec_query(
        services.0,
        bstr_raw(&wql),
        bstr_raw(&query),
        0,
        std::ptr::null_mut(),
        &mut enum_ptr,
    )
    .is_err()
        || enum_ptr.is_null()
    {
        return false;
    }
    let enum_obj = ComRaw(enum_ptr);

    let next_fn: unsafe extern "system" fn(
        *mut c_void,
        i32,
        u32,
        *mut *mut c_void,
        *mut u32,
    ) -> HRESULT = std::mem::transmute(*enum_obj.vtable().add(4));
    let exec_method: unsafe extern "system" fn(
        *mut c_void,
        *const u16,
        *const u16,
        i32,
        *mut c_void,
        *mut c_void,
        *mut *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT = std::mem::transmute(*services.vtable().add(24));

    let mut applied = false;
    loop {
        let mut obj_ptr: *mut c_void = std::ptr::null_mut();
        let mut returned = 0u32;
        if next_fn(enum_obj.0, -1, 1, &mut obj_ptr, &mut returned).is_err()
            || returned == 0
            || obj_ptr.is_null()
        {
            break;
        }
        let obj = ComRaw(obj_ptr);

        let mut rel_var = RawVariant::default();
        let get_fn: unsafe extern "system" fn(
            *mut c_void,
            PCWSTR,
            i32,
            *mut RawVariant,
            *mut i32,
            *mut i32,
        ) -> HRESULT = std::mem::transmute(*obj.vtable().add(4));
        if get_fn(
            obj.0,
            windows::core::w!("__RELPATH"),
            0,
            &mut rel_var,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
        .is_err()
        {
            continue;
        }
        let relpath_str = if rel_var.vt == 8 && !rel_var.data.bstr_val.is_null() {
            PCWSTR(rel_var.data.bstr_val).to_string().unwrap_or_default()
        } else {
            String::new()
        };
        let _ = VariantClear(&mut rel_var);
        if relpath_str.is_empty() {
            continue;
        }

        let mut in_params_ptr: *mut c_void = std::ptr::null_mut();
        let spawn_inst: unsafe extern "system" fn(*mut c_void, i32, *mut *mut c_void) -> HRESULT =
            std::mem::transmute(*in_cls.vtable().add(15));
        if spawn_inst(in_cls.0, 0, &mut in_params_ptr).is_err() || in_params_ptr.is_null() {
            continue;
        }
        let in_params = ComRaw(in_params_ptr);

        let put_fn: unsafe extern "system" fn(
            *mut c_void,
            PCWSTR,
            i32,
            *const RawVariant,
            i32,
        ) -> HRESULT = std::mem::transmute(*in_params.vtable().add(5));

        let mut b_var = RawVariant::default();
        b_var.vt = 17; // VT_UI1
        b_var.data.b_val = percent;
        let _ = put_fn(in_params.0, windows::core::w!("Brightness"), 0, &b_var, 0);

        let mut t_var = RawVariant::default();
        t_var.vt = 3; // VT_I4
        t_var.data.l_val = 0;
        let _ = put_fn(in_params.0, windows::core::w!("Timeout"), 0, &t_var, 0);

        let obj_path = BSTR::from(relpath_str.as_str());
        let method_name = BSTR::from("WmiSetBrightness");
        if exec_method(
            services.0,
            bstr_raw(&obj_path),
            bstr_raw(&method_name),
            0,
            std::ptr::null_mut(),
            in_params.0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
        .is_ok()
        {
            applied = true;
        }
    }
    applied
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Dxva2PhysicalMonitor {
    h_physical_monitor: usize,
    description: [u16; 128],
}

struct Dxva2Fns {
    get_num: unsafe extern "system" fn(HMONITOR, *mut u32) -> BOOL,
    get_mons: unsafe extern "system" fn(HMONITOR, u32, *mut Dxva2PhysicalMonitor) -> BOOL,
    destroy_mons: unsafe extern "system" fn(u32, *mut Dxva2PhysicalMonitor) -> BOOL,
    get_bright: unsafe extern "system" fn(usize, *mut u32, *mut u32, *mut u32) -> BOOL,
    set_bright: unsafe extern "system" fn(usize, u32) -> BOOL,
}

unsafe fn load_dxva2() -> Option<Dxva2Fns> {
    let module = LoadLibraryW(windows::core::w!("dxva2.dll")).ok()?;
    let get_num = GetProcAddress(module, windows::core::s!("GetNumberOfPhysicalMonitorsFromHMONITOR"))?;
    let get_mons = GetProcAddress(module, windows::core::s!("GetPhysicalMonitorsFromHMONITOR"))?;
    let destroy_mons = GetProcAddress(module, windows::core::s!("DestroyPhysicalMonitors"))?;
    let get_bright = GetProcAddress(module, windows::core::s!("GetMonitorBrightness"))?;
    let set_bright = GetProcAddress(module, windows::core::s!("SetMonitorBrightness"))?;
    Some(Dxva2Fns {
        get_num: std::mem::transmute(get_num),
        get_mons: std::mem::transmute(get_mons),
        destroy_mons: std::mem::transmute(destroy_mons),
        get_bright: std::mem::transmute(get_bright),
        set_bright: std::mem::transmute(set_bright),
    })
}

unsafe fn dxva2_read_brightness() -> Option<u8> {
    let fns = load_dxva2()?;
    struct Ctx<'a> {
        fns: &'a Dxva2Fns,
        found: Option<u8>,
    }
    unsafe extern "system" fn enum_proc(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut Ctx<'_>);
        if ctx.found.is_some() {
            return false.into();
        }
        let mut count = 0u32;
        if (ctx.fns.get_num)(hmonitor, &mut count).as_bool() && count > 0 {
            let mut mons = vec![std::mem::zeroed::<Dxva2PhysicalMonitor>(); count as usize];
            if (ctx.fns.get_mons)(hmonitor, count, mons.as_mut_ptr()).as_bool() {
                for mon in &mons {
                    if mon.h_physical_monitor != 0 {
                        let (mut min_b, mut cur_b, mut max_b) = (0u32, 0u32, 0u32);
                        if (ctx.fns.get_bright)(
                            mon.h_physical_monitor,
                            &mut min_b,
                            &mut cur_b,
                            &mut max_b,
                        )
                        .as_bool()
                            && max_b > min_b
                        {
                            let pct = (((cur_b.saturating_sub(min_b)) as f32
                                / (max_b - min_b) as f32)
                                * 100.0)
                                .round() as u8;
                            ctx.found = Some(clamp_brightness_percent(pct));
                            break;
                        }
                    }
                }
                let _ = (ctx.fns.destroy_mons)(count, mons.as_mut_ptr());
            }
        }
        ctx.found.is_none().into()
    }
    let mut ctx = Ctx {
        fns: &fns,
        found: None,
    };
    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(enum_proc),
        LPARAM(&mut ctx as *mut Ctx<'_> as isize),
    );
    ctx.found
}

unsafe fn dxva2_set_brightness(percent: u8) -> bool {
    let Some(fns) = load_dxva2() else {
        return false;
    };
    struct Ctx<'a> {
        fns: &'a Dxva2Fns,
        percent: u8,
        applied: bool,
    }
    unsafe extern "system" fn enum_proc(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut Ctx<'_>);
        let mut count = 0u32;
        if (ctx.fns.get_num)(hmonitor, &mut count).as_bool() && count > 0 {
            let mut mons = vec![std::mem::zeroed::<Dxva2PhysicalMonitor>(); count as usize];
            if (ctx.fns.get_mons)(hmonitor, count, mons.as_mut_ptr()).as_bool() {
                for mon in &mons {
                    if mon.h_physical_monitor != 0 {
                        if (ctx.fns.set_bright)(mon.h_physical_monitor, ctx.percent as u32)
                            .as_bool()
                        {
                            ctx.applied = true;
                        }
                    }
                }
                let _ = (ctx.fns.destroy_mons)(count, mons.as_mut_ptr());
            }
        }
        true.into()
    }
    let mut ctx = Ctx {
        fns: &fns,
        percent,
        applied: false,
    };
    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(enum_proc),
        LPARAM(&mut ctx as *mut Ctx<'_> as isize),
    );
    ctx.applied
}

/// Reads brightness using a persistent `IWbemServices` handle on the telemetry poller thread,
/// falling back to DXVA2 DDC/CI and reconnecting lazily only if the WMI query fails.
fn read_brightness_with_wmi(wmi_services: &mut Option<ComRaw>) -> u8 {
    let hw = unsafe {
        let mut reading = wmi_services
            .as_ref()
            .and_then(|svc| wmi_read_brightness_with_services(svc));
        if reading.is_none() {
            *wmi_services = connect_wmi_services();
            if let Some(svc) = wmi_services.as_ref() {
                reading = wmi_read_brightness_with_services(svc);
            }
        }
        reading.or_else(|| dxva2_read_brightness())
    };
    if let Some(pct) = hw {
        let clamped = clamp_brightness_percent(pct);
        CACHED_BRIGHTNESS.store(clamped, Ordering::Relaxed);
        clamped
    } else {
        CACHED_BRIGHTNESS.load(Ordering::Relaxed)
    }
}

/// Reads the current system display brightness (`0..=100`) via WMI (`WmiMonitorBrightness`),
/// falling back to DXVA2 DDC/CI and finally the last cached brightness value.
pub fn read_brightness() -> u8 {
    let _com = ThreadComGuard::enter();
    let hw = unsafe { wmi_read_brightness().or_else(|| dxva2_read_brightness()) };
    if let Some(pct) = hw {
        let clamped = clamp_brightness_percent(pct);
        CACHED_BRIGHTNESS.store(clamped, Ordering::Relaxed);
        clamped
    } else {
        CACHED_BRIGHTNESS.load(Ordering::Relaxed)
    }
}

/// Sets the system display brightness (`0..=100`) via WMI (`WmiMonitorBrightnessMethods`)
/// and DXVA2 DDC/CI, updating the cached fallback level even if no hardware panel responds.
pub fn set_brightness(percent: u8) -> std::result::Result<(), String> {
    let clamped = clamp_brightness_percent(percent);
    CACHED_BRIGHTNESS.store(clamped, Ordering::Relaxed);

    let _com = ThreadComGuard::enter();
    let wmi_ok = unsafe { wmi_set_brightness(clamped) };
    let dxva_ok = unsafe { dxva2_set_brightness(clamped) };
    if wmi_ok || dxva_ok {
        Ok(())
    } else {
        Err("No WMI or DDC/CI brightness-capable display detected".to_string())
    }
}

// ---------------------------------------------------------------------------
// Available Wi-Fi Network List (`WlanGetAvailableNetworkList`)
// ---------------------------------------------------------------------------

/// Merges duplicate SSIDs (keeping highest signal, ORing `connected`/`saved`) and sorts
/// connected first, then saved, then strongest signal, then SSID ascending.
pub fn merge_and_sort_wifi_networks(entries: Vec<WifiNetworkInfo>) -> Vec<WifiNetworkInfo> {
    let mut by_ssid: HashMap<String, WifiNetworkInfo> = HashMap::new();
    for mut entry in entries {
        let trimmed = entry.ssid.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        match by_ssid.get_mut(&trimmed) {
            Some(existing) => {
                existing.signal_percent = existing.signal_percent.max(entry.signal_percent);
                existing.connected |= entry.connected;
                existing.saved |= entry.saved;
                existing.secured |= entry.secured;
            }
            None => {
                entry.ssid = trimmed.clone();
                entry.signal_percent = entry.signal_percent.min(100);
                by_ssid.insert(trimmed, entry);
            }
        }
    }

    let mut list: Vec<WifiNetworkInfo> = by_ssid.into_values().collect();
    list.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.saved.cmp(&a.saved))
            .then(b.signal_percent.cmp(&a.signal_percent))
            .then_with(|| a.ssid.to_lowercase().cmp(&b.ssid.to_lowercase()))
    });
    list
}

/// Scans nearby infrastructure Wi-Fi networks via `WlanGetAvailableNetworkList`.
pub fn scan_available_wifi_networks() -> Vec<WifiNetworkInfo> {
    let Some(client) = WlanClient::open() else {
        return Vec::new();
    };
    let ifaces = client.interfaces();
    let target_guid = ifaces
        .iter()
        .find(|(_, state)| *state == wlan_interface_state_connected)
        .or_else(|| ifaces.first())
        .map(|(guid, _)| *guid);
    let Some(guid) = target_guid else {
        return Vec::new();
    };

    let mut list: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
    let rc = unsafe { WlanGetAvailableNetworkList(client.0, &guid, 0, None, &mut list) };
    if rc == ERROR_ACCESS_DENIED.0 || rc != 0 || list.is_null() {
        if let Some(ssid) = query_connected_wifi_ssid() {
            return vec![WifiNetworkInfo {
                ssid,
                signal_percent: 100,
                secured: true,
                connected: true,
                saved: true,
            }];
        }
        return Vec::new();
    }

    let mut collected = Vec::new();
    unsafe {
        if let Some(header) = list.as_ref() {
            let count = (header.dwNumberOfItems as usize).min(1024);
            if count > 0 {
                let entries: &[WLAN_AVAILABLE_NETWORK] =
                    std::slice::from_raw_parts(header.Network.as_ptr(), count);
                for entry in entries {
                    let bytes = ssid_bytes(&entry.dot11Ssid);
                    if bytes.is_empty() || entry.dot11BssType != dot11_BSS_type_infrastructure {
                        continue;
                    }
                    let ssid = String::from_utf8_lossy(bytes).into_owned();
                    collected.push(WifiNetworkInfo {
                        ssid,
                        signal_percent: entry.wlanSignalQuality.min(100) as u8,
                        secured: entry.bSecurityEnabled.as_bool(),
                        connected: (entry.dwFlags & WLAN_AVAILABLE_NETWORK_CONNECTED) != 0,
                        saved: (entry.dwFlags & WLAN_AVAILABLE_NETWORK_HAS_PROFILE) != 0,
                    });
                }
            }
        }
        WlanFreeMemory(list as *const _);
    }

    merge_and_sort_wifi_networks(collected)
}

// ---------------------------------------------------------------------------
// Live System Tray Icon Count
// ---------------------------------------------------------------------------

#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(
        hKey: isize,
        lpSubKey: PCWSTR,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut isize,
    ) -> i32;
    fn RegEnumKeyExW(
        hKey: isize,
        dwIndex: u32,
        lpName: PWSTR,
        lpcchName: *mut u32,
        lpReserved: *mut u32,
        lpClass: PWSTR,
        lpcchClass: *mut u32,
        lpftLastWriteTime: *mut c_void,
    ) -> i32;
    fn RegQueryValueExW(
        hKey: isize,
        lpValueName: PCWSTR,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> i32;
    fn RegCloseKey(hKey: isize) -> i32;
}

const HKEY_CURRENT_USER: isize = -2147483647isize; // 0x80000001
const REG_KEY_READ: u32 = 0x20019;
const REG_SZ_TYPE: u32 = 1;

unsafe fn reg_open(parent: isize, subkey: &str) -> Option<isize> {
    let wide: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();
    let mut out = 0isize;
    if RegOpenKeyExW(parent, PCWSTR(wide.as_ptr()), 0, REG_KEY_READ, &mut out) == 0 {
        Some(out)
    } else {
        None
    }
}

unsafe fn reg_string(hkey: isize, name: &str) -> Option<String> {
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut kind = 0u32;
    let mut size = 0u32;
    if RegQueryValueExW(
        hkey,
        PCWSTR(wide.as_ptr()),
        std::ptr::null_mut(),
        &mut kind,
        std::ptr::null_mut(),
        &mut size,
    ) != 0
        || kind != REG_SZ_TYPE
        || size == 0
        || size > 65536
    {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    if RegQueryValueExW(
        hkey,
        PCWSTR(wide.as_ptr()),
        std::ptr::null_mut(),
        &mut kind,
        buf.as_mut_ptr(),
        &mut size,
    ) != 0
    {
        return None;
    }
    let chars: Vec<u16> = buf[..size as usize]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&ch| ch != 0)
        .collect();
    String::from_utf16(&chars).ok()
}

const REG_DWORD_TYPE: u32 = 4;
const REG_BINARY_TYPE: u32 = 3;

unsafe fn reg_dword(hkey: isize, name: &str) -> Option<u32> {
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut kind = 0u32;
    let mut val = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    if RegQueryValueExW(
        hkey,
        PCWSTR(wide.as_ptr()),
        std::ptr::null_mut(),
        &mut kind,
        (&mut val as *mut u32).cast::<u8>(),
        &mut size,
    ) == 0
        && kind == REG_DWORD_TYPE
    {
        Some(val)
    } else {
        None
    }
}

unsafe fn reg_bytes(hkey: isize, name: &str) -> Option<Vec<u8>> {
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut kind = 0u32;
    let mut size = 0u32;
    if RegQueryValueExW(
        hkey,
        PCWSTR(wide.as_ptr()),
        std::ptr::null_mut(),
        &mut kind,
        std::ptr::null_mut(),
        &mut size,
    ) != 0
        || kind != REG_BINARY_TYPE
        || size == 0
        || size > 512 * 1024
    {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    if RegQueryValueExW(
        hkey,
        PCWSTR(wide.as_ptr()),
        std::ptr::null_mut(),
        &mut kind,
        buf.as_mut_ptr(),
        &mut size,
    ) != 0
    {
        return None;
    }
    buf.truncate(size as usize);
    Some(buf)
}

/// Active background System Tray icon discovered from Windows `NotifyIconSettings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTrayRecord {
    pub id: String,
    pub label: String,
    pub exe_path: String,
    pub hwnd: isize,
    pub pid: u32,
    pub launch_uri: Option<String>,
    pub icon_png: Option<Vec<u8>>,
}

/// Resolves Windows Known Folder GUID prefixes (e.g. `{6D809377-...}\Google\...`)
/// stored in `NotifyIconSettings\ExecutablePath` to full filesystem paths (`C:\Program Files\...`).
pub fn resolve_known_folder_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if !trimmed.starts_with('{') {
        return trimmed.to_string();
    }
    let Some(close_idx) = trimmed.find('}') else {
        return trimmed.to_string();
    };
    let guid_str = &trimmed[1..close_idx];
    let Ok(guid) = GUID::try_from(guid_str) else {
        return trimmed.to_string();
    };
    let tail = &trimmed[close_idx + 1..];
    unsafe {
        if let Ok(pwstr) = SHGetKnownFolderPath(&guid, KF_FLAG_DEFAULT, None) {
            if !pwstr.is_null() {
                let len = (0..32768).take_while(|&i| *pwstr.0.add(i) != 0).count();
                let base = String::from_utf16_lossy(std::slice::from_raw_parts(pwstr.0, len));
                CoTaskMemFree(Some(pwstr.0.cast()));
                return format!("{base}{tail}");
            }
        }
    }
    trimmed.to_string()
}

fn has_active_bluetooth_radio() -> bool {
    #[repr(C)]
    struct BluetoothFindRadioParams {
        dw_size: u32,
    }
    unsafe {
        let Ok(dll) = LoadLibraryW(windows::core::w!("BluetoothApis.dll")) else {
            return false;
        };
        let Some(find_first) = GetProcAddress(dll, windows::core::s!("BluetoothFindFirstRadio"))
        else {
            return false;
        };
        let Some(find_close) = GetProcAddress(dll, windows::core::s!("BluetoothFindRadioClose"))
        else {
            return false;
        };
        let find_first_fn: unsafe extern "system" fn(
            *const BluetoothFindRadioParams,
            *mut HANDLE,
        ) -> HANDLE = std::mem::transmute(find_first);
        let find_close_fn: unsafe extern "system" fn(HANDLE) -> BOOL =
            std::mem::transmute(find_close);
        let params = BluetoothFindRadioParams {
            dw_size: std::mem::size_of::<BluetoothFindRadioParams>() as u32,
        };
        let mut radio = HANDLE::default();
        let search = find_first_fn(&params, &mut radio);
        if search.0.is_null() {
            false
        } else {
            if !radio.0.is_null() {
                let _ = CloseHandle(radio);
            }
            let _ = find_close_fn(search);
            true
        }
    }
}

fn enumerate_running_process_exes() -> Vec<(u32, String)> {
    #[link(name = "kernel32")]
    extern "system" {
        fn K32EnumProcesses(lpid_process: *mut u32, cb: u32, lpcb_needed: *mut u32) -> i32;
    }

    let mut pids = vec![0u32; 512];
    let mut needed = 0u32;
    unsafe {
        if K32EnumProcesses(
            pids.as_mut_ptr(),
            (pids.len() * std::mem::size_of::<u32>()) as u32,
            &mut needed,
        ) == 0
        {
            return Vec::new();
        }
    }
    let count = (needed as usize / std::mem::size_of::<u32>()).min(pids.len());
    let mut result = Vec::with_capacity(count);
    for &pid in &pids[..count] {
        if pid == 0 {
            continue;
        }
        unsafe {
            if let Ok(proc_h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut path_buf = [0u16; 1024];
                let mut len = path_buf.len() as u32;
                if QueryFullProcessImageNameW(
                    proc_h,
                    PROCESS_NAME_WIN32,
                    PWSTR(path_buf.as_mut_ptr()),
                    &mut len,
                )
                .is_ok()
                {
                    let exe = String::from_utf16_lossy(&path_buf[..len as usize]);
                    result.push((pid, exe));
                }
                let _ = CloseHandle(proc_h);
            }
        }
    }
    result
}

/// Discovers live background & hidden system tray items from `Control Panel\NotifyIconSettings`
/// matched against running OS processes (`K32EnumProcesses` + Known Folder resolution) and active
/// Windows system tray items (such as Bluetooth Devices), preserving PNG `IconSnapshot` artwork.
pub fn collect_live_tray_records() -> Vec<LiveTrayRecord> {
    collect_live_tray_records_inner(true)
}

fn collect_live_tray_records_inner(include_icons: bool) -> Vec<LiveTrayRecord> {
    const BLUETOOTH_ICON_GUID: &str = "7820AE78-23E3-4229-82C1-E41CB67D5B9C";

    unsafe extern "system" fn collect_hwnd_exe(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(isize, u32, String)>);
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid != 0 {
            if let Ok(proc_h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut path_buf = [0u16; 1024];
                let mut len = path_buf.len() as u32;
                if QueryFullProcessImageNameW(
                    proc_h,
                    PROCESS_NAME_WIN32,
                    PWSTR(path_buf.as_mut_ptr()),
                    &mut len,
                )
                .is_ok()
                {
                    let exe = String::from_utf16_lossy(&path_buf[..len as usize]);
                    out.push((hwnd.0 as isize, pid, exe));
                }
                let _ = CloseHandle(proc_h);
            }
        }
        true.into()
    }

    unsafe {
        let Some(root) = reg_open(HKEY_CURRENT_USER, "Control Panel\\NotifyIconSettings") else {
            return Vec::new();
        };

        let mut windows_list: Vec<(isize, u32, String)> = Vec::new();
        let param = LPARAM(&mut windows_list as *mut Vec<(isize, u32, String)> as isize);
        let _ = EnumWindows(Some(collect_hwnd_exe), param);
        let mut prev = None;
        while let Ok(hwnd) = FindWindowExW(Some(HWND_MESSAGE), prev, PCWSTR::null(), PCWSTR::null())
        {
            let _ = collect_hwnd_exe(hwnd, param);
            prev = Some(hwnd);
        }

        let mut exe_to_pid: HashMap<String, u32> = HashMap::new();
        for (pid, exe) in enumerate_running_process_exes() {
            exe_to_pid.entry(exe.to_ascii_lowercase()).or_insert(pid);
        }
        let mut exe_to_hwnd: HashMap<String, isize> = HashMap::new();
        for (hwnd, pid, exe) in &windows_list {
            let key = exe.to_ascii_lowercase();
            exe_to_hwnd.entry(key.clone()).or_insert(*hwnd);
            exe_to_pid.entry(key).or_insert(*pid);
        }

        let bt_active = has_active_bluetooth_radio();
        let own_exe = std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().to_ascii_lowercase());
        let mut by_key: HashMap<String, LiveTrayRecord> = HashMap::new();

        for idx in 0..1024u32 {
            let mut name_buf = [0u16; 256];
            let mut name_len = name_buf.len() as u32;
            if RegEnumKeyExW(
                root,
                idx,
                PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
                std::ptr::null_mut(),
                PWSTR::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) != 0
            {
                break;
            }
            let subkey = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let Some(item_key) = reg_open(root, &subkey) else {
                continue;
            };
            let raw_exe = reg_string(item_key, "ExecutablePath");
            let icon_guid = reg_string(item_key, "IconGuid");
            let uid = reg_dword(item_key, "UID");
            let is_promoted = reg_dword(item_key, "IsPromoted");
            let initial_tooltip = reg_string(item_key, "InitialTooltip");
            let icon_png = if include_icons {
                reg_bytes(item_key, "IconSnapshot")
            } else {
                None
            };
            let _ = RegCloseKey(item_key);

            if is_promoted == Some(1) {
                continue;
            }
            let Some(raw_exe) = raw_exe else {
                continue;
            };
            if icon_guid.is_none() && uid.is_none() {
                continue;
            }

            let resolved_exe = resolve_known_folder_path(&raw_exe);
            let lower = resolved_exe.to_ascii_lowercase();
            let exe_name = Path::new(&resolved_exe)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("App");
            let exe_lower = exe_name.to_ascii_lowercase();

            if own_exe.as_deref() == Some(lower.as_str())
                || matches!(
                    exe_lower.as_str(),
                    "svchost.exe"
                        | "conhost.exe"
                        | "sihost.exe"
                        | "runtimebroker.exe"
                        | "shellexperiencehost.exe"
                        | "startmenuexperiencehost.exe"
                        | "searchhost.exe"
                        | "textinputhost.exe"
                )
            {
                continue;
            }

            if exe_lower == "explorer.exe" {
                let norm_guid = icon_guid
                    .as_deref()
                    .map(|g| g.trim_start_matches('{').trim_end_matches('}'));
                if norm_guid.is_some_and(|g| g.eq_ignore_ascii_case(BLUETOOTH_ICON_GUID)) && bt_active
                {
                    let key = format!("guid:{BLUETOOTH_ICON_GUID}");
                    let entry = LiveTrayRecord {
                        id: key.clone(),
                        label: "Bluetooth Devices".to_string(),
                        exe_path: resolved_exe,
                        hwnd: 0,
                        pid: 0,
                        launch_uri: Some("ms-settings:bluetooth".to_string()),
                        icon_png,
                    };
                    by_key.entry(key).or_insert(entry);
                }
                continue;
            }

            let Some(&pid) = exe_to_pid.get(&lower) else {
                continue;
            };

            let hwnd = exe_to_hwnd.get(&lower).copied().unwrap_or(0);
            let mut nid = NOTIFYICONIDENTIFIER {
                cbSize: std::mem::size_of::<NOTIFYICONIDENTIFIER>() as u32,
                ..Default::default()
            };
            let rect_confirmed = if let Some(ref guid_str) = icon_guid {
                GUID::try_from(guid_str.trim_start_matches('{').trim_end_matches('}'))
                    .ok()
                    .is_some_and(|guid| {
                        nid.guidItem = guid;
                        Shell_NotifyIconGetRect(&nid).is_ok()
                    })
            } else if let Some(u) = uid {
                hwnd != 0 && {
                    nid.hWnd = HWND(hwnd as *mut _);
                    nid.uID = u;
                    Shell_NotifyIconGetRect(&nid).is_ok()
                }
            } else {
                false
            };

            // CLI shells and regular web browsers share EXEs with user sessions even when not tray-hosted;
            // only keep them if Shell_NotifyIconGetRect actively confirms their tray icon.
            if matches!(
                exe_lower.as_str(),
                "powershell.exe"
                    | "pwsh.exe"
                    | "cmd.exe"
                    | "wt.exe"
                    | "windowsterminal.exe"
                    | "chrome.exe"
                    | "msedge.exe"
                    | "brave.exe"
                    | "firefox.exe"
            ) && !rect_confirmed
            {
                continue;
            }

            let clean_tip = initial_tooltip
                .as_deref()
                .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|t| !t.is_empty());
            let label = clean_tip.unwrap_or_else(|| friendly_app_name(exe_name));

            let candidate = LiveTrayRecord {
                id: lower.clone(),
                label,
                exe_path: resolved_exe,
                hwnd,
                pid,
                launch_uri: None,
                icon_png,
            };
            match by_key.get_mut(&lower) {
                Some(existing) => {
                    if existing.icon_png.is_none() && candidate.icon_png.is_some() {
                        *existing = candidate;
                    }
                }
                None => {
                    by_key.insert(lower, candidate);
                }
            }
        }

        let _ = RegCloseKey(root);
        let mut records: Vec<LiveTrayRecord> = by_key.into_values().collect();
        records.sort_by(|a, b| a.label.to_ascii_lowercase().cmp(&b.label.to_ascii_lowercase()));
        records
    }
}

/// Counts live background/hidden system tray icons without allocating `IconSnapshot` PNG bytes.
pub fn read_tray_count() -> u32 {
    collect_live_tray_records_inner(false).len() as u32
}

fn read_smtc_music(
    manager: &mut Option<GlobalSystemMediaTransportControlsSessionManager>,
) -> Option<MusicInfo> {
    let mgr = manager.as_ref()?;
    let Ok(sessions) = mgr.GetSessions() else {
        *manager = None;
        return None;
    };

    let mut fallback: Option<MusicInfo> = None;
    let count = sessions.Size().unwrap_or(0);
    for i in 0..count {
        let Ok(session) = sessions.GetAt(i) else {
            continue;
        };
        let playback_status = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|p| p.PlaybackStatus().ok());
        if playback_status == Some(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed)
        {
            continue;
        }
        let is_playing = playback_status
            == Some(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing);

        let Ok(props) = session
            .TryGetMediaPropertiesAsync()
            .and_then(|op| op.get())
        else {
            continue;
        };
        let title = props.Title().unwrap_or_default().to_string();
        if title.is_empty() {
            continue;
        }
        let artist = props.Artist().unwrap_or_default().to_string();
        let info = MusicInfo {
            title,
            artist,
            lyric_line: None,
            is_playing,
        };
        if is_playing {
            return Some(info);
        }
        if fallback.is_none() {
            fallback = Some(info);
        }
    }
    fallback
}

unsafe fn has_visible_owner(hwnd: HWND) -> bool {
    let Ok(owner) = GetWindow(hwnd, GW_OWNER) else {
        return false;
    };
    !owner.0.is_null() && IsWindowVisible(owner).as_bool()
}

unsafe fn query_process_image_path(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    let Ok(process_handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
        return None;
    };
    let mut path_buf = [0u16; 1024];
    let mut path_len = path_buf.len() as u32;
    let got_path = QueryFullProcessImageNameW(
        process_handle,
        PROCESS_NAME_WIN32,
        windows::core::PWSTR(path_buf.as_mut_ptr()),
        &mut path_len,
    )
    .is_ok();
    let _ = CloseHandle(process_handle);
    got_path.then(|| String::from_utf16_lossy(&path_buf[..path_len as usize]))
}

struct UwpChildSearch {
    host_pid: u32,
    core_match: Option<(HWND, String)>,
    fallback_match: Option<(HWND, String)>,
}

unsafe extern "system" fn enum_uwp_child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam.0 as *mut UwpChildSearch);
    let mut child_pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut child_pid));
    if child_pid == 0 || child_pid == state.host_pid {
        return true.into();
    }
    let Some(child_path) = query_process_image_path(child_pid) else {
        return true.into();
    };

    let mut class_buf = [0u8; 128];
    let class_len = GetClassNameA(hwnd, &mut class_buf);
    let class_name = std::str::from_utf8(&class_buf[..class_len.max(0) as usize]).unwrap_or("");
    if class_name == "Windows.UI.Core.CoreWindow" {
        state.core_match = Some((hwnd, child_path));
        return false.into();
    }
    if state.fallback_match.is_none() {
        state.fallback_match = Some((hwnd, child_path));
    }
    true.into()
}

unsafe fn resolve_uwp_child_target(host_hwnd: HWND, host_pid: u32) -> Option<(HWND, String)> {
    let mut search = UwpChildSearch {
        host_pid,
        core_match: None,
        fallback_match: None,
    };
    let _ = EnumChildWindows(
        Some(host_hwnd),
        Some(enum_uwp_child_proc),
        LPARAM(&mut search as *mut UwpChildSearch as isize),
    );
    search.core_match.or(search.fallback_match)
}

fn find_windowsapps_exe_from_registry(
    package_prefix: &str,
    exe_candidates: &[&str],
) -> Option<String> {
    const PACKAGES_KEY: &str =
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages";
    let root_key = unsafe { reg_open(HKEY_CURRENT_USER, PACKAGES_KEY) }?;
    let mut best_pkg: Option<String> = None;
    let mut best_path: Option<String> = None;
    let mut idx = 0u32;
    let mut name_buf = [0u16; 260];
    loop {
        let mut name_len = name_buf.len() as u32;
        let rc = unsafe {
            RegEnumKeyExW(
                root_key,
                idx,
                PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
                std::ptr::null_mut(),
                PWSTR::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if rc != 0 {
            break;
        }
        idx += 1;
        let subkey_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
        if !subkey_name.to_ascii_lowercase().starts_with(package_prefix) {
            continue;
        }
        if let Some(sub_key) = unsafe { reg_open(root_key, &subkey_name) } {
            let folder = unsafe { reg_string(sub_key, "PackageRootFolder") };
            unsafe {
                RegCloseKey(sub_key);
            }
            if let Some(root_folder) = folder {
                for exe in exe_candidates {
                    let candidate = Path::new(&root_folder).join(exe);
                    if candidate.is_file() {
                        if best_pkg.as_ref().is_none_or(|b| subkey_name > *b) {
                            best_pkg = Some(subkey_name.clone());
                            best_path = Some(candidate.to_string_lossy().into_owned());
                        }
                        break;
                    }
                }
            }
        }
    }
    unsafe {
        RegCloseKey(root_key);
    }
    best_path
}

fn find_windowsapps_exe(package_prefix: &str, exe_candidates: &[&str]) -> Option<String> {
    if let Some(reg_hit) = find_windowsapps_exe_from_registry(package_prefix, exe_candidates) {
        return Some(reg_hit);
    }
    let root = Path::new(r"C:\Program Files\WindowsApps");
    let entries = std::fs::read_dir(root).ok()?;
    let mut dirs: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|ent| ent.path()))
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.to_ascii_lowercase().starts_with(package_prefix))
        })
        .collect();
    dirs.sort();
    for dir in dirs.into_iter().rev() {
        for exe in exe_candidates {
            let candidate = dir.join(exe);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

fn uwp_resolved_cache() -> &'static std::sync::Mutex<HashMap<(isize, u32), String>> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<HashMap<(isize, u32), String>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn cache_uwp_resolved_exe(host_hwnd: isize, host_pid: u32, exe_path: &str) {
    if host_hwnd == 0 || host_pid == 0 || exe_path.is_empty() {
        return;
    }
    if let Ok(mut guard) = uwp_resolved_cache().lock() {
        if guard.len() >= 128 && !guard.contains_key(&(host_hwnd, host_pid)) {
            guard.clear();
        }
        guard.insert((host_hwnd, host_pid), exe_path.to_string());
    }
}

fn lookup_uwp_resolved_exe(host_hwnd: isize, host_pid: u32) -> Option<String> {
    uwp_resolved_cache()
        .lock()
        .ok()
        .and_then(|guard| guard.get(&(host_hwnd, host_pid)).cloned())
}

fn fallback_uwp_package_exe(title: &str) -> Option<String> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<HashMap<String, Option<String>>>> =
        std::sync::OnceLock::new();
    let key = title.trim().to_ascii_lowercase();
    if key.is_empty() {
        return None;
    }
    let map = CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    if let Ok(guard) = map.lock() {
        if let Some(cached) = guard.get(&key) {
            return cached.clone();
        }
    }

    let resolved = match key.as_str() {
        "calculator" | "kalkulator" => {
            find_windowsapps_exe("microsoft.windowscalculator_", &["CalculatorApp.exe"])
        }
        "settings" | "pengaturan" => {
            let sys = Path::new(r"C:\Windows\ImmersiveControlPanel\SystemSettings.exe");
            sys.is_file().then(|| sys.to_string_lossy().into_owned())
        }
        "photos" | "foto" => {
            find_windowsapps_exe("microsoft.windows.photos_", &["Photos.exe", "Microsoft.Photos.exe"])
        }
        "terminal" => {
            find_windowsapps_exe("microsoft.windowsterminal_", &["WindowsTerminal.exe"])
        }
        "clock" => find_windowsapps_exe("microsoft.windowsalarms_", &["Time.exe"]),
        "microsoft store" => {
            find_windowsapps_exe("microsoft.windowsstore_", &["WinStore.App.exe"])
        }
        _ => None,
    };

    if let Ok(mut guard) = map.lock() {
        if guard.len() >= 128 && !guard.contains_key(&key) {
            guard.clear();
        }
        guard.insert(key, resolved.clone());
    }
    resolved
}

/// Returns `true` when DWM cloaking means the window should be omitted from the Dock.
/// Minimized windows (`IsIconic`) on the current desktop (including suspended UWP apps with
/// `DWM_CLOAKED_APP = 1`) remain listed unless shell-cloaked (`DWM_CLOAKED_SHELL = 2`) for
/// another virtual desktop.
fn is_window_cloaked_excluded(cloaked: u32, is_minimized: bool) -> bool {
    const DWM_CLOAKED_SHELL: u32 = 0x0000_0002;
    if is_minimized {
        (cloaked & DWM_CLOAKED_SHELL) != 0
    } else {
        cloaked != 0
    }
}

/// Returns `true` when a non-minimized window has a degenerate or far-offscreen rect.
/// Win32 moves minimized (`IsIconic`) windows to `(-32000, -32000)`, so minimized windows
/// bypass this check and stay visible in the Dock.
fn is_window_rect_excluded(rect: RECT, is_minimized: bool) -> bool {
    if is_minimized {
        return false;
    }
    rect.right - rect.left <= 1
        || rect.bottom - rect.top <= 1
        || rect.left <= -9000
        || rect.top <= -9000
}

unsafe extern "system" fn enum_running_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<RunningWindow>);

    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }

    let is_minimized = IsIconic(hwnd).as_bool();

    let mut cloaked = 0u32;
    let size = std::mem::size_of::<u32>() as u32;
    if DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, size).is_ok()
        && is_window_cloaked_excluded(cloaked, is_minimized)
    {
        return true.into();
    }

    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_ok() && is_window_rect_excluded(rect, is_minimized) {
        return true.into();
    }

    let mut text = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut text);
    if len <= 0 {
        return true.into();
    }
    let title = String::from_utf16_lossy(&text[..len as usize]);

    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
        return true.into();
    }
    if (ex_style & WS_EX_APPWINDOW.0) == 0 && has_visible_owner(hwnd) {
        return true.into();
    }

    if matches!(
        title.as_str(),
        "Program Manager" | "Hime" | "Hime Island" | "Hime Dock"
    ) {
        return true.into();
    }

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    let Some(mut exe_path) = query_process_image_path(pid) else {
        return true.into();
    };
    let mut lower = exe_path.to_lowercase();

    let mut class_buf = [0u8; 256];
    let class_len = GetClassNameA(hwnd, &mut class_buf);
    let window_class = std::str::from_utf8(&class_buf[..class_len as usize]).unwrap_or("");

    let is_explorer_window = window_class == "CabinetWClass" || window_class == "ExploreWClass";
    if ((lower.contains("hime-island") || lower.contains("hime.exe")) && title != "Settings")
        || lower.contains("conhost.exe")
        || (lower.contains("explorer.exe") && !is_explorer_window)
        || lower.contains("shellexperiencehost.exe")
        || lower.contains("startmenuexperiencehost.exe")
        || lower.contains("searchhost.exe")
        || lower.contains("textinputhost.exe")
        || lower.contains("lockapp.exe")
        || lower.contains("widgets.exe")
        || window_class == "ApplicationFrameInputSinkWindow"
        || (window_class == "Windows.UI.Core.CoreWindow"
            && (lower.contains("windowsapps") || lower.contains("immersivecontrolpanel")))
        || (lower.contains("applicationframehost.exe")
            && window_class != "ApplicationFrameWindow")
    {
        return true.into();
    }

    let mut target_hwnd = hwnd;
    let is_uwp_frame =
        lower.contains("applicationframehost.exe") && window_class == "ApplicationFrameWindow";
    if is_uwp_frame {
        if let Some((child_hwnd, child_path)) = resolve_uwp_child_target(hwnd, pid) {
            target_hwnd = child_hwnd;
            exe_path = child_path;
            lower = exe_path.to_lowercase();
            if lower.contains("searchhost.exe")
                || lower.contains("shellexperiencehost.exe")
                || lower.contains("textinputhost.exe")
            {
                return true.into();
            }
            cache_uwp_resolved_exe(hwnd.0 as isize, pid, &exe_path);
        } else if let Some(cached_exe) = lookup_uwp_resolved_exe(hwnd.0 as isize, pid) {
            exe_path = cached_exe;
        } else if let Some(pkg_exe) = fallback_uwp_package_exe(&title) {
            exe_path = pkg_exe;
        }
    }

    let mut executable = Path::new(&exe_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app.exe")
        .to_string();
    let mut app_name = friendly_app_name(&executable);
    if is_uwp_frame && executable.eq_ignore_ascii_case("applicationframehost.exe") {
        let trimmed_title = title.trim();
        if !trimmed_title.is_empty() {
            app_name = trimmed_title.to_string();
            executable = format!("uwp:{}", trimmed_title.to_ascii_lowercase());
        }
    } else if app_name == "Windows App" && !title.trim().is_empty() {
        app_name = title.trim().to_string();
    }

    out.push(RunningWindow {
        hwnd: target_hwnd.0 as isize,
        title,
        app_name,
        executable,
        exe_path,
    });
    true.into()
}

/// Enumerates visible, non-cloaked top-level application windows for the Dock.
pub fn read_running_windows() -> Vec<RunningWindow> {
    let mut windows = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_running_windows_proc),
            LPARAM(&mut windows as *mut Vec<RunningWindow> as isize),
        );
    }
    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_power_status(ac: u8, flag: u8, percent: u8) -> SYSTEM_POWER_STATUS {
        SYSTEM_POWER_STATUS {
            ACLineStatus: ac,
            BatteryFlag: flag,
            BatteryLifePercent: percent,
            SystemStatusFlag: 0,
            BatteryLifeTime: u32::MAX,
            BatteryFullLifeTime: u32::MAX,
        }
    }

    #[test]
    fn parse_power_status_charging_and_clamped() {
        let charging = parse_system_power_status(&make_power_status(1, CHARGING_FLAG, 74));
        assert_eq!(
            charging,
            BatteryStatus {
                percent: 74,
                charging: true,
                present: true,
            }
        );

        let clamped = parse_system_power_status(&make_power_status(1, 1, 255));
        assert_eq!(clamped.percent, 100);
        assert!(clamped.present);
    }

    #[test]
    fn parse_power_status_no_battery() {
        let desktop = parse_system_power_status(&make_power_status(1, NO_SYSTEM_BATTERY, 255));
        assert!(!desktop.present);
        assert!(!desktop.charging);
        assert_eq!(desktop.percent, 0);
    }

    #[test]
    fn sanitize_audio_device_name_rejects_guids_and_empty() {
        assert_eq!(
            sanitize_audio_device_name(Some("{0.0.0.00000000}.{12345678-1234-1234-1234-1234567890ab}")),
            "Audio Output"
        );
        assert_eq!(sanitize_audio_device_name(Some("   ")), "Audio Output");
        assert_eq!(sanitize_audio_device_name(None), "Audio Output");
        assert_eq!(
            sanitize_audio_device_name(Some("  Speakers (Realtek High Definition Audio)  ")),
            "Speakers (Realtek High Definition Audio)"
        );
    }

    #[test]
    fn compute_kbps_calculates_kilobits_per_second() {
        // 125_000 bytes in 1000 ms = 1_000_000 bits/s = 1000 kbps
        let kbps = compute_kbps(1_000, 126_000, 1000);
        assert!((kbps - 1000.0).abs() < 0.01);
    }

    #[test]
    fn compute_kbps_handles_zero_or_wrap() {
        assert_eq!(compute_kbps(100, 500, 0), 0.0);
        assert_eq!(compute_kbps(500, 100, 1000), 0.0);
    }

    #[test]
    fn virtual_adapter_detection_filters_tunnels_and_vms() {
        assert!(is_virtual_adapter("Hyper-V Virtual Ethernet Adapter", "vEthernet (WSL)"));
        assert!(is_virtual_adapter("WireGuard Tunnel", "wg0"));
        assert!(!is_virtual_adapter("Realtek PCIe GbE Family Controller", "Ethernet"));
    }

    #[test]
    fn running_window_converts_to_dock_item() {
        let rw = RunningWindow {
            hwnd: 42,
            title: "Cargo.toml - Hime".to_string(),
            app_name: "Code".to_string(),
            executable: "Code.exe".to_string(),
            exe_path: "C:\\Program Files\\Microsoft VS Code\\Code.exe".to_string(),
        };
        let item = rw.to_dock_item();
        assert_eq!(item.id, "code.exe:42");
        assert_eq!(item.label, "Code");
        assert_eq!(item.executable, "Code.exe");
        assert_eq!(item.exe_path, rw.exe_path);
        assert_eq!(item.hwnds, vec![42]);
        assert!(item.running);
        assert!(!item.pinned);
    }

    #[test]
    fn group_running_windows_collapses_same_executable_with_badge_count() {
        let windows = vec![
            RunningWindow {
                hwnd: 101,
                title: "Tab 1".to_string(),
                app_name: "Chrome".to_string(),
                executable: "chrome.exe".to_string(),
                exe_path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
            },
            RunningWindow {
                hwnd: 102,
                title: "Tab 2".to_string(),
                app_name: "Chrome".to_string(),
                executable: "Chrome.exe".to_string(),
                exe_path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
            },
            RunningWindow {
                hwnd: 103,
                title: "Tab 3".to_string(),
                app_name: "Chrome".to_string(),
                executable: "CHROME.EXE".to_string(),
                exe_path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
            },
            RunningWindow {
                hwnd: 201,
                title: "Terminal".to_string(),
                app_name: "Terminal".to_string(),
                executable: "wt.exe".to_string(),
                exe_path: r"C:\Windows\System32\wt.exe".to_string(),
            },
        ];

        let grouped = group_running_windows(&windows);
        assert_eq!(grouped.len(), 2);

        assert_eq!(grouped[0].id, "chrome.exe");
        assert_eq!(grouped[0].hwnds, vec![101, 102, 103]);
        assert_eq!(grouped[0].badge_count, 3);
        assert!(grouped[0].running);

        assert_eq!(grouped[1].id, "wt.exe");
        assert_eq!(grouped[1].hwnds, vec![201]);
        assert_eq!(grouped[1].badge_count, 0);
        assert!(grouped[1].running);
    }

    #[test]
    fn local_clock_returns_valid_24h_range() {
        let (h, m) = read_local_clock();
        assert!(h < 24);
        assert!(m < 60);
    }

    #[test]
    fn telemetry_snapshot_default_is_consistent() {
        let snap = TelemetrySnapshot::default();
        assert_eq!(snap.network.kind, NetworkKind::Offline);
        assert_eq!(snap.brightness, 80);
        assert!(snap.available_wifi.is_empty());
        assert_eq!(snap.tray_count, 0);
        assert!(snap.music.is_none());
        assert!(snap.running_windows.is_empty());
    }

    #[test]
    fn clamp_volume_and_brightness_handle_bounds_and_nan() {
        assert_eq!(clamp_volume_scalar(-0.5), 0.0);
        assert_eq!(clamp_volume_scalar(1.8), 1.0);
        assert_eq!(clamp_volume_scalar(f32::NAN), 0.0);
        assert!((clamp_volume_scalar(0.42) - 0.42).abs() < 1e-5);

        assert_eq!(clamp_brightness_percent(0), 0);
        assert_eq!(clamp_brightness_percent(75), 75);
        assert_eq!(clamp_brightness_percent(250), 100);
        assert!(read_brightness() <= 100);
    }

    #[test]
    fn merge_and_sort_wifi_networks_dedupes_and_orders_correctly() {
        let input = vec![
            WifiNetworkInfo {
                ssid: "CafeGuest".to_string(),
                signal_percent: 45,
                secured: false,
                connected: false,
                saved: false,
            },
            WifiNetworkInfo {
                ssid: "HomeWifi".to_string(),
                signal_percent: 62,
                secured: true,
                connected: false,
                saved: true,
            },
            WifiNetworkInfo {
                ssid: "HomeWifi".to_string(),
                signal_percent: 88,
                secured: true,
                connected: true,
                saved: false,
            },
            WifiNetworkInfo {
                ssid: "   ".to_string(),
                signal_percent: 99,
                secured: true,
                connected: false,
                saved: false,
            },
        ];

        let sorted = merge_and_sort_wifi_networks(input);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].ssid, "HomeWifi");
        assert_eq!(sorted[0].signal_percent, 88);
        assert!(sorted[0].connected);
        assert!(sorted[0].saved);
        assert_eq!(sorted[1].ssid, "CafeGuest");
    }

    #[test]
    fn uwp_friendly_names_and_grouping_separate_apps() {
        assert_eq!(friendly_app_name("CalculatorApp.exe"), "Calculator");
        assert_eq!(friendly_app_name("SystemSettings.exe"), "Windows Settings");
        assert_eq!(friendly_app_name("Microsoft.Photos.exe"), "Photos");
        assert_eq!(friendly_app_name("WindowsTerminal.exe"), "Terminal");

        let windows = vec![
            RunningWindow {
                hwnd: 11,
                title: "Calculator".to_string(),
                app_name: friendly_app_name("CalculatorApp.exe"),
                executable: "CalculatorApp.exe".to_string(),
                exe_path: r"C:\Program Files\WindowsApps\Microsoft.WindowsCalculator\CalculatorApp.exe".to_string(),
            },
            RunningWindow {
                hwnd: 22,
                title: "Settings".to_string(),
                app_name: friendly_app_name("SystemSettings.exe"),
                executable: "SystemSettings.exe".to_string(),
                exe_path: r"C:\Windows\ImmersiveControlPanel\SystemSettings.exe".to_string(),
            },
        ];
        let grouped = group_running_windows(&windows);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].label, "Calculator");
        assert_eq!(grouped[0].badge_count, 0);
        assert_eq!(grouped[1].label, "Windows Settings");
        assert_eq!(grouped[1].badge_count, 0);

        if Path::new(r"C:\Windows\ImmersiveControlPanel\SystemSettings.exe").is_file() {
            assert!(fallback_uwp_package_exe("Settings")
                .is_some_and(|p| p.ends_with("SystemSettings.exe")));
        }
    }

    #[test]
    fn minimized_windows_bypass_offscreen_rect_and_app_cloak_filters() {
        let minimized_rect = RECT {
            left: -32000,
            top: -32000,
            right: -31840,
            bottom: -31972,
        };
        assert!(!is_window_rect_excluded(minimized_rect, true));
        assert!(is_window_rect_excluded(minimized_rect, false));

        let tiny_rect = RECT {
            left: 100,
            top: 100,
            right: 101,
            bottom: 101,
        };
        assert!(is_window_rect_excluded(tiny_rect, false));

        let normal_rect = RECT {
            left: 120,
            top: 80,
            right: 1120,
            bottom: 780,
        };
        assert!(!is_window_rect_excluded(normal_rect, false));

        // DWM_CLOAKED_APP (1): hidden when non-minimized, preserved when minimized UWP frame.
        assert!(is_window_cloaked_excluded(1, false));
        assert!(!is_window_cloaked_excluded(1, true));
        assert!(!is_window_cloaked_excluded(0, true));
        // DWM_CLOAKED_SHELL (2): window lives on another virtual desktop, always excluded.
        assert!(is_window_cloaked_excluded(2, false));
        assert!(is_window_cloaked_excluded(2, true));
        assert!(is_window_cloaked_excluded(3, true));
    }

    #[test]
    fn uwp_resolved_cache_and_registry_fallback_preserve_exe_path() {
        cache_uwp_resolved_exe(
            999_001,
            42_420,
            r"C:\Program Files\WindowsApps\Microsoft.WindowsCalculator\CalculatorApp.exe",
        );
        assert_eq!(
            lookup_uwp_resolved_exe(999_001, 42_420).as_deref(),
            Some(r"C:\Program Files\WindowsApps\Microsoft.WindowsCalculator\CalculatorApp.exe")
        );
        assert_eq!(lookup_uwp_resolved_exe(999_001, 99_999), None);

        if let Some(calc_exe) = fallback_uwp_package_exe("Calculator") {
            assert!(
                calc_exe.ends_with("CalculatorApp.exe"),
                "unexpected Calculator path: {calc_exe}"
            );
            assert!(Path::new(&calc_exe).is_file());
        }
    }

    #[test]
    fn resolve_known_folder_and_tray_friendly_names() {
        let sys_resolved =
            resolve_known_folder_path(r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\SecurityHealthSystray.exe");
        assert!(
            sys_resolved.to_ascii_lowercase().ends_with(r"\system32\securityhealthsystray.exe"),
            "unexpected known folder expansion: {sys_resolved}"
        );
        assert_eq!(
            resolve_known_folder_path(r"C:\Apps\Tool.exe"),
            r"C:\Apps\Tool.exe"
        );
        assert_eq!(friendly_app_name("GoogleDriveFS.exe"), "Google Drive");
        assert_eq!(friendly_app_name("DCv2.exe"), "MSI Center Pro");
    }
}
