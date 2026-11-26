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

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use windows::core::{BOOL, GUID, PCWSTR};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Networking::Connectivity::{NetworkConnectivityLevel, NetworkInformation};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetBestInterface, GetIfTable2, GetIpForwardTable2, MIB_IF_ROW2, MIB_IF_TABLE2,
    MIB_IPFORWARD_ROW2, MIB_IPFORWARD_TABLE2,
};
use windows::Win32::NetworkManagement::WiFi::{
    wlan_interface_state_connected, wlan_intf_opcode_current_connection, WlanCloseHandle,
    WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle, WlanQueryInterface, DOT11_SSID,
    WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
    WLAN_INTERFACE_STATE,
};
use windows::Win32::Networking::WinSock::{ADDRESS_FAMILY, AF_INET, AF_INET6};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    COINIT_MULTITHREADED,
};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameA, GetWindow, GetWindowLongW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, GW_OWNER, WS_EX_APPWINDOW,
    WS_EX_TOOLWINDOW,
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
        item.running = true;
        item
    }
}

/// Point-in-time snapshot of all OS telemetry consumed by the native shell surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub hours: u8,
    pub minutes: u8,
    pub network: NetworkStatus,
    pub battery: BatteryStatus,
    pub volume: VolumeStatus,
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

    while !stop.load(Ordering::Relaxed) {
        let (hours, minutes) = read_local_clock();
        let network = read_network_status(&mut prev_net);
        let battery = read_battery_status();
        let volume = read_volume_status(enumerator.as_ref());

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

    let device_name = unsafe {
        device.GetId().ok().map(|id| {
            let s = PCWSTR::from_raw(id.0).to_string().unwrap_or_default();
            CoTaskMemFree(Some(id.0 as *const _));
            s
        })
    }
    .filter(|s| !s.is_empty());

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

unsafe extern "system" fn enum_running_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<RunningWindow>);

    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }

    let mut cloaked = 0u32;
    let size = std::mem::size_of::<u32>() as u32;
    if DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, size).is_ok()
        && cloaked != 0
    {
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
    if pid == 0 {
        return true.into();
    }

    let Ok(process_handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
        return true.into();
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

    if !got_path {
        return true.into();
    }

    let exe_path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
    let lower = exe_path.to_lowercase();

    let mut class_buf = [0u8; 256];
    let class_len = GetClassNameA(hwnd, &mut class_buf);
    let window_class = std::str::from_utf8(&class_buf[..class_len as usize]).unwrap_or("");

    let is_explorer_window = window_class == "CabinetWClass" || window_class == "ExploreWClass";
    if ((lower.contains("hime-island") || lower.contains("hime.exe")) && title != "Settings")
        || lower.contains("conhost.exe")
        || (lower.contains("explorer.exe") && !is_explorer_window)
        || lower.contains("shellexperiencehost.exe")
        || lower.contains("searchhost.exe")
        || lower.contains("textinputhost.exe")
        || (lower.contains("applicationframehost.exe")
            && window_class != "ApplicationFrameWindow")
    {
        return true.into();
    }

    let executable = Path::new(&exe_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app.exe")
        .to_string();
    let app_name = executable.strip_suffix(".exe").unwrap_or(&executable).to_string();

    out.push(RunningWindow {
        hwnd: hwnd.0 as isize,
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
        assert!(item.running);
        assert!(!item.pinned);
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
        assert!(snap.music.is_none());
        assert!(snap.running_windows.is_empty());
    }
}
