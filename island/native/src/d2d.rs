//! Direct2D + DirectWrite + DirectComposition window & rendering surface.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use windows::core::{s, w, HRESULT, PCWSTR, Result};
use windows_numerics::Vector2 as D2D_POINT_2F;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_IGNORE, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BEZIER_SEGMENT, D2D1_COLOR_F,
    D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED,
    D2D1_FIGURE_END_OPEN, D2D1_PIXEL_FORMAT, D2D_RECT_F, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Bitmap, ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1Layer,
    ID2D1PathGeometry, ID2D1SolidColorBrush, ID2D1StrokeStyle,
    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_PROPERTIES, D2D1_CAP_STYLE_ROUND,
    D2D1_ELLIPSE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_LAYER_PARAMETERS, D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_SOFTWARE, D2D1_ROUNDED_RECT, D2D1_STROKE_STYLE_PROPERTIES1,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::{
    BeginPath, CombineRgn, CreateCompatibleDC, CreateDIBSection, CreateRoundRectRgn, DeleteDC,
    DeleteObject, EndPath, LineTo, MoveToEx, PathToRegion, PolyBezierTo, SelectObject,
    SetWindowRgn, ValidateRect, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HRGN,
    RGN_OR,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow, DrawIconEx, GetClassLongPtrW,
    GetIconInfo, GetWindowLongW, LoadCursorW, PostMessageW, RegisterClassExW, SendMessageTimeoutW,
    SetCursor, SetWindowLongW, SetWindowPos, ShowWindow, CS_HREDRAW, CS_VREDRAW, DI_NORMAL,
    GCLP_HICON, GCLP_HICONSM, GWL_EXSTYLE, HICON, HTCLIENT, HWND_TOPMOST, ICONINFO, ICON_BIG,
    ICON_SMALL, ICON_SMALL2, IDC_ARROW, SMTO_ABORTIFHUNG, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE, WM_APP, WM_DROPFILES, WM_GETICON,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_SETCURSOR, WNDCLASSEXW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::theme::{
    Color, ACCENT_CYAN, ACCENT_GLOW, FONT_FAMILY, STATUS_ALERT, STATUS_OFFLINE, STATUS_SUCCESS,
    STATUS_WARNING, TRANSPARENT,
};

const WINDOW_CLASS_NAME: PCWSTR = w!("HimeIslandNativeSurface");
static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();
static EVENT_TARGET_HWND: AtomicIsize = AtomicIsize::new(0);

/// Custom window message posted to the controller HWND when a surface queues input events.
pub const WM_SURFACE_EVENT: u32 = WM_APP + 2;

/// Input / drag-and-drop event forwarded from a [`NativeSurface`] window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceEventKind {
    MouseMove { x: f32, y: f32 },
    LeftButtonDown { x: f32, y: f32 },
    LeftButtonUp { x: f32, y: f32 },
    DropFiles { wparam: usize },
}

/// Event emitted by a [`NativeSurface`] window, tagged with its `HWND` (`isize`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceEvent {
    pub hwnd: isize,
    pub kind: SurfaceEventKind,
}

thread_local! {
    static SURFACE_EVENTS: RefCell<Vec<SurfaceEvent>> = const { RefCell::new(Vec::new()) };
    static SHARED_D2D_FACTORY: RefCell<Option<ID2D1Factory1>> = const { RefCell::new(None) };
    static SHARED_DWRITE_FACTORY: RefCell<Option<IDWriteFactory>> = const { RefCell::new(None) };
}

fn shared_d2d_factory() -> Result<ID2D1Factory1> {
    SHARED_D2D_FACTORY.with(|cell| {
        let mut slot = cell.borrow_mut();
        if let Some(factory) = slot.as_ref() {
            return Ok(factory.clone());
        }
        let factory: ID2D1Factory1 =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        *slot = Some(factory.clone());
        Ok(factory)
    })
}

fn shared_dwrite_factory() -> Result<IDWriteFactory> {
    SHARED_DWRITE_FACTORY.with(|cell| {
        let mut slot = cell.borrow_mut();
        if let Some(factory) = slot.as_ref() {
            return Ok(factory.clone());
        }
        let factory: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        *slot = Some(factory.clone());
        Ok(factory)
    })
}

/// Trims unused working-set pages from the current process back to the OS.
pub fn trim_working_set() {
    unsafe {
        let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

/// Registers the controller window that should be woken via [`WM_SURFACE_EVENT`].
pub fn set_surface_event_target(hwnd: HWND) {
    EVENT_TARGET_HWND.store(hwnd.0 as isize, Ordering::Relaxed);
}

/// Drains all queued [`SurfaceEvent`]s from the current thread.
pub fn drain_surface_events() -> Vec<SurfaceEvent> {
    SURFACE_EVENTS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// Axis-aligned rectangle in DIPs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn to_d2d(self) -> D2D_RECT_F {
        D2D_RECT_F {
            left: self.x,
            top: self.y,
            right: self.x + self.width,
            bottom: self.y + self.height,
        }
    }
}

/// Horizontal text alignment for `draw_text`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Leading,
    Center,
    Trailing,
}

/// Screen edge orientation for [`edge_contour_points`] and [`NativeSurface::draw_edge_surface`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Path operation produced by [`edge_contour_points`] (shared by Direct2D and GDI `PathToRegion`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContourOp {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CubicTo {
        cp1: (f32, f32),
        cp2: (f32, f32),
        end: (f32, f32),
    },
    Close,
}

/// Extracted 32-bit premultiplied BGRA application icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconRgba {
    pub width: u32,
    pub height: u32,
    pub bgra_premul: Vec<u8>,
}

/// Computes exact `edgeSurfaceContour` Bezier + rounded-corner path operations for `rect`.
pub fn edge_contour_points(
    rect: Rect,
    ear_w: f32,
    radius: f32,
    edge: EdgeSide,
    flares: bool,
) -> Vec<ContourOp> {
    let (lw, lh) = match edge {
        EdgeSide::Top | EdgeSide::Bottom => (rect.width.max(1.0), rect.height.max(1.0)),
        EdgeSide::Left | EdgeSide::Right => (rect.height.max(1.0), rect.width.max(1.0)),
    };

    let ear = if flares {
        ear_w.max(0.0).min(lw * 0.25)
    } else {
        0.0
    };
    let left = ear;
    let right = (lw - ear).max(left);
    let r = radius.max(0.0).min(lh * 0.5).min((right - left) * 0.5);
    let k = 0.552_284_8 * r;
    let c_off = ear * (10.0 / 13.0);

    let mut local_ops = Vec::with_capacity(10);
    if edge == EdgeSide::Bottom && !flares && r > 0.0 {
        local_ops.push(ContourOp::MoveTo(left, r));
        local_ops.push(ContourOp::CubicTo {
            cp1: (left, r - k),
            cp2: (left + r - k, 0.0),
            end: (left + r, 0.0),
        });
        local_ops.push(ContourOp::LineTo(right - r, 0.0));
        local_ops.push(ContourOp::CubicTo {
            cp1: (right - r + k, 0.0),
            cp2: (right, r - k),
            end: (right, r),
        });
        local_ops.push(ContourOp::LineTo(right, lh - r));
        local_ops.push(ContourOp::CubicTo {
            cp1: (right, lh - r + k),
            cp2: (right - r + k, lh),
            end: (right - r, lh),
        });
        local_ops.push(ContourOp::LineTo(left + r, lh));
        local_ops.push(ContourOp::CubicTo {
            cp1: (left + r - k, lh),
            cp2: (left, lh - r + k),
            end: (left, lh - r),
        });
        local_ops.push(ContourOp::Close);
    } else {
        if flares && ear > 0.0 {
            local_ops.push(ContourOp::MoveTo(0.0, 0.0));
            local_ops.push(ContourOp::CubicTo {
                cp1: (c_off, 0.0),
                cp2: (ear, ear - c_off),
                end: (ear, ear),
            });
        } else {
            local_ops.push(ContourOp::MoveTo(left, 0.0));
        }

        local_ops.push(ContourOp::LineTo(left, lh - r));
        if r > 0.0 {
            local_ops.push(ContourOp::CubicTo {
                cp1: (left, lh - r + k),
                cp2: (left + r - k, lh),
                end: (left + r, lh),
            });
        }
        local_ops.push(ContourOp::LineTo(right - r, lh));
        if r > 0.0 {
            local_ops.push(ContourOp::CubicTo {
                cp1: (right - r + k, lh),
                cp2: (right, lh - r + k),
                end: (right, lh - r),
            });
        }
        if flares && ear > 0.0 {
            local_ops.push(ContourOp::LineTo(right, ear));
            local_ops.push(ContourOp::CubicTo {
                cp1: (right, ear - c_off),
                cp2: (lw - c_off, 0.0),
                end: (lw, 0.0),
            });
        } else {
            local_ops.push(ContourOp::LineTo(right, 0.0));
        }
        local_ops.push(ContourOp::Close);
    }

    let map_pt = |(lx, ly): (f32, f32)| -> (f32, f32) {
        match edge {
            EdgeSide::Top => (rect.x + lx, rect.y + ly),
            EdgeSide::Bottom => (rect.x + lx, rect.y + (lh - ly)),
            EdgeSide::Left => (rect.x + ly, rect.y + lx),
            EdgeSide::Right => (rect.x + (lh - ly), rect.y + lx),
        }
    };

    local_ops
        .into_iter()
        .map(|op| match op {
            ContourOp::MoveTo(x, y) => {
                let (mx, my) = map_pt((x, y));
                ContourOp::MoveTo(mx, my)
            }
            ContourOp::LineTo(x, y) => {
                let (mx, my) = map_pt((x, y));
                ContourOp::LineTo(mx, my)
            }
            ContourOp::CubicTo { cp1, cp2, end } => ContourOp::CubicTo {
                cp1: map_pt(cp1),
                cp2: map_pt(cp2),
                end: map_pt(end),
            },
            ContourOp::Close => ContourOp::Close,
        })
        .collect()
}

/// Extracts a 32-bit premultiplied BGRA icon from a Win32 `HWND` or executable path.
pub fn extract_window_or_exe_icon(hwnd: isize, exe_path: &str, size: u32) -> Option<IconRgba> {
    let target_size = size.clamp(16, 128);
    unsafe {
        let mut owned_icon = false;
        let mut hicon = HICON::default();

        if hwnd != 0 {
            let wnd = HWND(hwnd as *mut std::ffi::c_void);
            for icon_kind in [ICON_BIG, ICON_SMALL2, ICON_SMALL] {
                let mut res: usize = 0;
                let ok = SendMessageTimeoutW(
                    wnd,
                    WM_GETICON,
                    WPARAM(icon_kind as usize),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG,
                    50,
                    Some(&mut res),
                );
                if ok.0 != 0 && res != 0 {
                    hicon = HICON(res as *mut std::ffi::c_void);
                    break;
                }
            }
            if hicon.0.is_null() {
                let gcl = GetClassLongPtrW(wnd, GCLP_HICON);
                if gcl != 0 {
                    hicon = HICON(gcl as *mut std::ffi::c_void);
                } else {
                    let gcl_sm = GetClassLongPtrW(wnd, GCLP_HICONSM);
                    if gcl_sm != 0 {
                        hicon = HICON(gcl_sm as *mut std::ffi::c_void);
                    }
                }
            }
        }

        if hicon.0.is_null() && !exe_path.is_empty() {
            let wide_path = to_wide_null(exe_path);
            let mut large_icon = HICON::default();
            let count = ExtractIconExW(
                PCWSTR(wide_path.as_ptr()),
                0,
                Some(&mut large_icon),
                None,
                1,
            );
            if count > 0 && !large_icon.0.is_null() {
                hicon = large_icon;
                owned_icon = true;
            }
        }

        if hicon.0.is_null() {
            return None;
        }

        let extracted = icon_to_bgra_premul(hicon, target_size);
        if owned_icon {
            let _ = DestroyIcon(hicon);
        }
        extracted
    }
}

unsafe fn icon_to_bgra_premul(hicon: HICON, size: u32) -> Option<IconRgba> {
    let mut info = ICONINFO::default();
    if GetIconInfo(hicon, &mut info).is_err() {
        return None;
    }
    if !info.hbmColor.0.is_null() {
        let _ = DeleteObject(info.hbmColor.into());
    }
    if !info.hbmMask.0.is_null() {
        let _ = DeleteObject(info.hbmMask.into());
    }

    let mem_dc = CreateCompatibleDC(None);
    if mem_dc.0.is_null() {
        return None;
    }

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size as i32,
            biHeight: -(size as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let Ok(dib) = CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
        let _ = DeleteDC(mem_dc);
        return None;
    };
    let old = SelectObject(mem_dc, dib.into());
    let byte_len = (size * size * 4) as usize;
    std::ptr::write_bytes(bits as *mut u8, 0, byte_len);

    let drawn = DrawIconEx(
        mem_dc,
        0,
        0,
        hicon,
        size as i32,
        size as i32,
        0,
        None,
        DI_NORMAL,
    )
    .is_ok();

    let mut out = if drawn && !bits.is_null() {
        std::slice::from_raw_parts(bits as *const u8, byte_len).to_vec()
    } else {
        Vec::new()
    };

    let _ = SelectObject(mem_dc, old);
    let _ = DeleteObject(dib.into());
    let _ = DeleteDC(mem_dc);

    if out.is_empty() {
        return None;
    }

    let has_alpha = out.chunks_exact(4).any(|px| px[3] > 0);
    if !has_alpha {
        let mut any_rgb = false;
        for px in out.chunks_exact_mut(4) {
            if px[0] != 0 || px[1] != 0 || px[2] != 0 {
                px[3] = 255;
                any_rgb = true;
            }
        }
        if !any_rgb {
            return None;
        }
    }

    Some(IconRgba {
        width: size,
        height: size,
        bgra_premul: out,
    })
}

/// Hime mascot eye expression states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HimeExpression {
    Netral,
    Senang,
    Fokus,
    Khawatir,
    Offline,
    Waspada,
}

impl Default for HimeExpression {
    fn default() -> Self {
        Self::Netral
    }
}

const MAX_BITMAP_CACHE_ENTRIES: usize = 64;
const MAX_BRUSH_CACHE_ENTRIES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegionKey {
    Rounded {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        diam: i32,
    },
    Edge(u64),
    Multi(u64),
}

#[derive(Default)]
struct BitmapCache {
    entries: HashMap<(u32, u32, u64), ID2D1Bitmap>,
    order: VecDeque<(u32, u32, u64)>,
}

impl BitmapCache {
    fn get(&self, key: &(u32, u32, u64)) -> Option<ID2D1Bitmap> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: (u32, u32, u64), bitmap: ID2D1Bitmap) {
        if self.entries.contains_key(&key) {
            return;
        }
        while self.order.len() >= MAX_BITMAP_CACHE_ENTRIES {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
            }
        }
        self.order.push_back(key);
        self.entries.insert(key, bitmap);
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
    }
}

/// Borderless topmost window backed by Direct2D `ID2D1HwndRenderTarget` (software rasterizer)
/// + DirectWrite + `SetWindowRgn` clipping.
pub struct NativeSurface {
    hwnd: HWND,
    width: u32,
    height: u32,
    dpi: f32,
    d2d_factory: ID2D1Factory1,
    dwrite_factory: IDWriteFactory,
    d2d_target: ID2D1HwndRenderTarget,
    brush_cache: RefCell<HashMap<u32, ID2D1SolidColorBrush>>,
    text_format_cache: RefCell<HashMap<(u32, u8, bool), IDWriteTextFormat>>,
    bitmap_cache: RefCell<BitmapCache>,
    cached_layer: RefCell<Option<ID2D1Layer>>,
    round_cap_stroke: RefCell<Option<ID2D1StrokeStyle>>,
    last_region_key: RefCell<Option<RegionKey>>,
}

impl NativeSurface {
    /// Creates a new borderless topmost Direct2D HWND render target window.
    pub fn new(title: &str, x: i32, y: i32, width: u32, height: u32) -> Result<Self> {
        let width = width.max(1);
        let height = height.max(1);

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let hinstance: HINSTANCE = GetModuleHandleW(None)?.into();
            register_window_class(hinstance);

            let wide_title = to_wide_null(title);
            let ex_style = WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
            let hwnd = CreateWindowExW(
                ex_style,
                WINDOW_CLASS_NAME,
                PCWSTR(wide_title.as_ptr()),
                WS_POPUP,
                x,
                y,
                width as i32,
                height as i32,
                None,
                None,
                Some(hinstance),
                None,
            )?;

            let dpi = get_window_dpi(hwnd);

            let d2d_factory = shared_d2d_factory()?;
            let dwrite_factory = shared_dwrite_factory()?;

            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: dpi,
                dpiY: dpi,
                ..Default::default()
            };
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };
            let d2d_target = d2d_factory.CreateHwndRenderTarget(&rt_props, &hwnd_props)?;

            Ok(Self {
                hwnd,
                width,
                height,
                dpi,
                d2d_factory,
                dwrite_factory,
                d2d_target,
                brush_cache: RefCell::new(HashMap::new()),
                text_format_cache: RefCell::new(HashMap::new()),
                bitmap_cache: RefCell::new(BitmapCache::default()),
                cached_layer: RefCell::new(None),
                round_cap_stroke: RefCell::new(None),
                last_region_key: RefCell::new(None),
            })
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    /// Returns the logical surface size in DIPs adjusted for the window's DPI.
    pub fn dip_size(&self) -> (f32, f32) {
        let scale = (self.dpi / 96.0).max(0.5);
        (self.width as f32 / scale, self.height as f32 / scale)
    }

    pub fn d2d_factory(&self) -> &ID2D1Factory1 {
        &self.d2d_factory
    }

    pub fn dwrite_factory(&self) -> &IDWriteFactory {
        &self.dwrite_factory
    }

    pub fn render_target(&self) -> &ID2D1HwndRenderTarget {
        &self.d2d_target
    }

    /// Shows the surface without stealing keyboard focus and ensures topmost Z-order.
    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    /// Hides the surface window.
    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Toggles `WS_EX_TRANSPARENT` so pointer events pass through when idle/non-interactive.
    pub fn set_click_through(&self, enabled: bool) {
        unsafe {
            let style = GetWindowLongW(self.hwnd, GWL_EXSTYLE);
            let transparent = WS_EX_TRANSPARENT.0 as i32;
            let new_style = if enabled {
                style | transparent
            } else {
                style & !transparent
            };
            if new_style != style {
                SetWindowLongW(self.hwnd, GWL_EXSTYLE, new_style);
            }
        }
    }

    /// Moves and resizes the surface window and its backing DXGI swap chain.
    pub fn set_bounds(&mut self, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
        let width = width.max(1);
        let height = height.max(1);
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                width as i32,
                height as i32,
                SWP_NOACTIVATE,
            );
        }
        if width != self.width || height != self.height {
            self.resize_swap_chain(width, height)?;
        }
        Ok(())
    }

    /// Resizes only the window position without reallocating the swap chain.
    pub fn set_position(&self, x: i32, y: i32) {
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    /// Clips the window to a rounded rectangle in DIPs so areas outside the pill are transparent.
    pub fn set_rounded_region(&self, rect: Rect, radius: f32) {
        let scale = (self.dpi / 96.0).max(0.5);
        let left = (rect.x * scale).round() as i32;
        let top = (rect.y * scale).round() as i32;
        let right = ((rect.x + rect.width) * scale).round() as i32 + 1;
        let bottom = ((rect.y + rect.height) * scale).round() as i32 + 1;
        let diam = ((radius * 2.0) * scale).round().max(0.0) as i32;
        let key = RegionKey::Rounded {
            left,
            top,
            right,
            bottom,
            diam,
        };
        if *self.last_region_key.borrow() == Some(key) {
            return;
        }
        unsafe {
            let rgn = CreateRoundRectRgn(left, top, right, bottom, diam, diam);
            if !rgn.is_invalid() {
                // On success, the system owns `rgn` and deletes it automatically.
                if SetWindowRgn(self.hwnd, Some(rgn), true) != 0 {
                    *self.last_region_key.borrow_mut() = Some(key);
                } else {
                    let _ = DeleteObject(rgn.into());
                }
            }
        }
    }

    /// Clips the window to the union of one or more rounded rectangles in DIPs, keeping
    /// the surface region cache in sync across transitions (e.g. expanded Dock <-> collapsed strip).
    pub fn set_rounded_regions(&self, rects: &[(Rect, f32)]) {
        if rects.is_empty() {
            return;
        }
        if rects.len() == 1 {
            self.set_rounded_region(rects[0].0, rects[0].1);
            return;
        }

        let scale = (self.dpi / 96.0).max(0.5);
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut coords: Vec<(i32, i32, i32, i32, i32)> = Vec::with_capacity(rects.len());
        for &(r, rad) in rects {
            let left = (r.x * scale).round() as i32;
            let top = (r.y * scale).round() as i32;
            let right = ((r.x + r.width) * scale).round() as i32 + 1;
            let bottom = ((r.y + r.height) * scale).round() as i32 + 1;
            let diam = ((rad * 2.0) * scale).round().max(0.0) as i32;
            for v in [left, top, right, bottom, diam] {
                hash ^= v as u64;
                hash = hash.wrapping_mul(0x1000_0000_01b3);
            }
            coords.push((left, top, right, bottom, diam));
        }

        let key = RegionKey::Multi(hash);
        if *self.last_region_key.borrow() == Some(key) {
            return;
        }

        unsafe {
            let mut combined: Option<HRGN> = None;
            for (left, top, right, bottom, diam) in coords {
                let next = CreateRoundRectRgn(left, top, right, bottom, diam, diam);
                if next.is_invalid() {
                    continue;
                }
                match combined {
                    None => combined = Some(next),
                    Some(acc) => {
                        let _ = CombineRgn(Some(acc), Some(acc), Some(next), RGN_OR);
                        let _ = DeleteObject(next.into());
                    }
                }
            }
            if let Some(acc) = combined {
                if SetWindowRgn(self.hwnd, Some(acc), true) != 0 {
                    *self.last_region_key.borrow_mut() = Some(key);
                } else {
                    let _ = DeleteObject(acc.into());
                }
            }
        }
    }

    /// Begins a Direct2D frame and clears the HWND render target.
    pub fn begin_draw(&self) {
        unsafe {
            self.d2d_target.BeginDraw();
            self.d2d_target.Clear(Some(&TRANSPARENT.to_d2d()));
        }
    }

    /// Ends the Direct2D frame and presents directly to the HWND.
    pub fn end_draw_and_present(&self) -> Result<()> {
        unsafe {
            self.d2d_target.EndDraw(None, None)?;
            Ok(())
        }
    }

    /// Fills a rounded rectangle with a solid color.
    pub fn fill_rounded_rect(&self, rect: Rect, radius: f32, color: Color) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let rr = D2D1_ROUNDED_RECT {
                rect: rect.to_d2d(),
                radiusX: radius,
                radiusY: radius,
            };
            self.d2d_target.FillRoundedRectangle(&rr, &brush);
            Ok(())
        }
    }

    /// Strokes a rounded rectangle outline (e.g. subtle glassmorphic border).
    pub fn stroke_rounded_rect(
        &self,
        rect: Rect,
        radius: f32,
        color: Color,
        stroke_width: f32,
    ) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let rr = D2D1_ROUNDED_RECT {
                rect: rect.to_d2d(),
                radiusX: radius,
                radiusY: radius,
            };
            self.d2d_target
                .DrawRoundedRectangle(&rr, &brush, stroke_width, None);
            Ok(())
        }
    }

    /// Draws UTF-8 text inside `rect` using DirectWrite (semi-bold).
    pub fn draw_text(
        &self,
        text: &str,
        rect: Rect,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> Result<()> {
        self.draw_text_weighted(text, rect, font_size, color, align, false)
    }

    /// Draws bold UTF-8 text inside `rect` using DirectWrite.
    pub fn draw_text_bold(
        &self,
        text: &str,
        rect: Rect,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> Result<()> {
        self.draw_text_weighted(text, rect, font_size, color, align, true)
    }

    fn draw_text_weighted(
        &self,
        text: &str,
        rect: Rect,
        font_size: f32,
        color: Color,
        align: TextAlign,
        bold: bool,
    ) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let format = self.create_text_format_weighted(font_size, align, bold)?;
            let wide: Vec<u16> = text.encode_utf16().collect();
            let d2d_rect = rect.to_d2d();
            self.d2d_target.DrawText(
                &wide,
                &format,
                &d2d_rect,
                &brush,
                Default::default(),
                Default::default(),
            );
            Ok(())
        }
    }

    /// Draws a straight line between `(x1, y1)` and `(x2, y2)` with round caps.
    pub fn draw_line(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
        stroke_width: f32,
    ) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let round_cap = self.create_round_cap_stroke()?;
            self.d2d_target.DrawLine(
                pt(x1, y1),
                pt(x2, y2),
                &brush,
                stroke_width.max(0.5),
                Some(&round_cap),
            );
            Ok(())
        }
    }

    /// Draws a filled or stroked circle centered at `(cx, cy)`.
    pub fn draw_circle(
        &self,
        cx: f32,
        cy: f32,
        radius: f32,
        color: Color,
        filled: bool,
        stroke_width: f32,
    ) -> Result<()> {
        if filled {
            self.fill_circle(cx, cy, radius, color)
        } else {
            self.stroke_circle(cx, cy, radius, color, stroke_width)
        }
    }

    /// Fills a circle centered at `(cx, cy)`.
    pub fn fill_circle(&self, cx: f32, cy: f32, radius: f32, color: Color) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let ellipse = D2D1_ELLIPSE {
                point: pt(cx, cy),
                radiusX: radius.max(0.5),
                radiusY: radius.max(0.5),
            };
            self.d2d_target.FillEllipse(&ellipse, &brush);
            Ok(())
        }
    }

    /// Strokes a circle outline centered at `(cx, cy)`.
    pub fn stroke_circle(
        &self,
        cx: f32,
        cy: f32,
        radius: f32,
        color: Color,
        stroke_width: f32,
    ) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let ellipse = D2D1_ELLIPSE {
                point: pt(cx, cy),
                radiusX: radius.max(0.5),
                radiusY: radius.max(0.5),
            };
            self.d2d_target
                .DrawEllipse(&ellipse, &brush, stroke_width.max(0.5), None);
            Ok(())
        }
    }

    /// Draws inverse-rounded "ears" on the top-left and top-right corners of `rect`
    /// so a top-docked Dynamic Island blends smoothly into the monitor's top bezel.
    pub fn draw_edge_surface_ears(&self, rect: Rect, ear_radius: f32, color: Color) -> Result<()> {
        if ear_radius <= 0.0 {
            return Ok(());
        }
        unsafe {
            let brush = self.create_brush(color)?;
            let r = ear_radius.min(rect.height * 0.5);

            // Left concave ear
            let left_geo = self.build_ear_geometry(rect.x, rect.y, r, true)?;
            self.d2d_target.FillGeometry(&left_geo, &brush, None);

            // Right concave ear
            let right_geo = self.build_ear_geometry(rect.x + rect.width, rect.y, r, false)?;
            self.d2d_target.FillGeometry(&right_geo, &brush, None);

            Ok(())
        }
    }

    /// Draws Hime's expressive eyes centered at `(center_x, center_y)`.
    /// `blink_progress` in `0.0..=1.0` smoothly closes the eyes (`0.0` = open, `1.0` = closed).
    pub fn draw_hime_eyes(
        &self,
        center_x: f32,
        center_y: f32,
        scale: f32,
        expression: HimeExpression,
        blink_progress: f32,
    ) -> Result<()> {
        let s = scale.max(0.25);
        let blink = blink_progress.clamp(0.0, 1.0);
        let eye_spacing = 12.0 * s;
        let left_x = center_x - eye_spacing * 0.5;
        let right_x = center_x + eye_spacing * 0.5;

        let (eye_color, glow_color) = match expression {
            HimeExpression::Netral | HimeExpression::Fokus => (ACCENT_CYAN, ACCENT_GLOW),
            HimeExpression::Senang => (STATUS_SUCCESS, STATUS_SUCCESS.with_alpha(0.25)),
            HimeExpression::Khawatir => (STATUS_WARNING, STATUS_WARNING.with_alpha(0.25)),
            HimeExpression::Offline => (STATUS_OFFLINE, STATUS_OFFLINE.with_alpha(0.15)),
            HimeExpression::Waspada => (STATUS_ALERT, STATUS_ALERT.with_alpha(0.32)),
        };

        unsafe {
            let brush = self.create_brush(eye_color)?;
            let glow_brush = self.create_brush(glow_color)?;
            let round_cap = self.create_round_cap_stroke()?;

            // Closed-eye fast path for full blink or Offline expression
            if blink > 0.9 || expression == HimeExpression::Offline {
                let half_w = 3.2 * s;
                for ex in [left_x, right_x] {
                    self.d2d_target.DrawLine(
                        pt(ex - half_w, center_y),
                        pt(ex + half_w, center_y),
                        &brush,
                        1.8 * s,
                        Some(&round_cap),
                    );
                }
                return Ok(());
            }

            let open_factor = 1.0 - blink;
            match expression {
                HimeExpression::Netral => {
                    for ex in [left_x, right_x] {
                        self.d2d_target.FillEllipse(
                            &D2D1_ELLIPSE {
                                point: pt(ex, center_y),
                                radiusX: 3.4 * s,
                                radiusY: (4.4 * s * open_factor).max(0.8),
                            },
                            &glow_brush,
                        );
                        self.d2d_target.FillEllipse(
                            &D2D1_ELLIPSE {
                                point: pt(ex, center_y),
                                radiusX: 2.2 * s,
                                radiusY: (3.2 * s * open_factor).max(0.6),
                            },
                            &brush,
                        );
                    }
                }
                HimeExpression::Senang => {
                    // Upward happy arcs (^ ^)
                    for ex in [left_x, right_x] {
                        let arc = self.build_quadratic_arc(
                            pt(ex - 3.2 * s, center_y + 1.2 * s),
                            pt(ex, center_y - 3.2 * s * open_factor),
                            pt(ex + 3.2 * s, center_y + 1.2 * s),
                        )?;
                        self.d2d_target
                            .DrawGeometry(&arc, &brush, 2.0 * s, Some(&round_cap));
                    }
                }
                HimeExpression::Fokus => {
                    // Narrowed determined horizontal pills
                    for ex in [left_x, right_x] {
                        let rr = D2D1_ROUNDED_RECT {
                            rect: D2D_RECT_F {
                                left: ex - 3.2 * s,
                                top: center_y - 1.5 * s * open_factor,
                                right: ex + 3.2 * s,
                                bottom: center_y + 1.5 * s * open_factor,
                            },
                            radiusX: 1.4 * s,
                            radiusY: 1.4 * s,
                        };
                        self.d2d_target.FillRoundedRectangle(&rr, &brush);
                    }
                }
                HimeExpression::Khawatir => {
                    // Tilted worried arcs (/ \)
                    let left_arc = self.build_quadratic_arc(
                        pt(left_x - 3.0 * s, center_y + 1.6 * s),
                        pt(left_x + 0.5 * s, center_y - 1.2 * s * open_factor),
                        pt(left_x + 2.8 * s, center_y - 1.8 * s * open_factor),
                    )?;
                    let right_arc = self.build_quadratic_arc(
                        pt(right_x - 2.8 * s, center_y - 1.8 * s * open_factor),
                        pt(right_x - 0.5 * s, center_y - 1.2 * s * open_factor),
                        pt(right_x + 3.0 * s, center_y + 1.6 * s),
                    )?;
                    self.d2d_target
                        .DrawGeometry(&left_arc, &brush, 2.0 * s, Some(&round_cap));
                    self.d2d_target
                        .DrawGeometry(&right_arc, &brush, 2.0 * s, Some(&round_cap));
                }
                HimeExpression::Waspada => {
                    // Wide alert eyes with outer ring
                    for ex in [left_x, right_x] {
                        let outer = D2D1_ELLIPSE {
                            point: pt(ex, center_y),
                            radiusX: 3.4 * s,
                            radiusY: (3.8 * s * open_factor).max(0.8),
                        };
                        self.d2d_target.FillEllipse(&outer, &glow_brush);
                        self.d2d_target.DrawEllipse(&outer, &brush, 1.5 * s, None);
                        self.d2d_target.FillEllipse(
                            &D2D1_ELLIPSE {
                                point: pt(ex, center_y),
                                radiusX: 1.4 * s,
                                radiusY: (1.6 * s * open_factor).max(0.5),
                            },
                            &brush,
                        );
                    }
                }
                HimeExpression::Offline => unreachable!(),
            }
            Ok(())
        }
    }

    /// Draws the exact `EdgeSurface` contour (`ID2D1PathGeometry`) with optional 1px border.
    pub fn draw_edge_surface(
        &self,
        rect: Rect,
        ear_w: f32,
        radius: f32,
        edge: EdgeSide,
        flares: bool,
        fill: Color,
        border_color: Option<Color>,
    ) -> Result<()> {
        let ops = edge_contour_points(rect, ear_w, radius, edge, flares);
        unsafe {
            let geo = self.build_contour_geometry(&ops)?;
            let fill_brush = self.create_brush(fill)?;
            self.d2d_target.FillGeometry(&geo, &fill_brush, None);
            if let Some(bc) = border_color {
                let border_brush = self.create_brush(bc)?;
                self.d2d_target.DrawGeometry(&geo, &border_brush, 1.0, None);
            }
            Ok(())
        }
    }

    /// Clips the window via GDI `BeginPath` + `PolyBezierTo` + `PathToRegion` + `SetWindowRgn`
    /// to match the exact `EdgeSurface` contour (including concave top/side ears).
    pub fn set_edge_surface_region(
        &self,
        rect: Rect,
        ear_w: f32,
        radius: f32,
        edge: EdgeSide,
        flares: bool,
    ) {
        let scale = (self.dpi / 96.0).max(0.5);
        let key = RegionKey::Edge(edge_region_hash(scale, rect, ear_w, radius, edge, flares));
        if *self.last_region_key.borrow() == Some(key) {
            return;
        }
        let ops = edge_contour_points(rect, ear_w, radius, edge, flares);
        let to_px = |x: f32, y: f32| POINT {
            x: (x * scale).round() as i32,
            y: (y * scale).round() as i32,
        };
        unsafe {
            let hdc = CreateCompatibleDC(None);
            if hdc.0.is_null() {
                return;
            }
            if BeginPath(hdc).as_bool() {
                for op in &ops {
                    match *op {
                        ContourOp::MoveTo(x, y) => {
                            let p = to_px(x, y);
                            let _ = MoveToEx(hdc, p.x, p.y, None);
                        }
                        ContourOp::LineTo(x, y) => {
                            let p = to_px(x, y);
                            let _ = LineTo(hdc, p.x, p.y);
                        }
                        ContourOp::CubicTo { cp1, cp2, end } => {
                            let pts = [
                                to_px(cp1.0, cp1.1),
                                to_px(cp2.0, cp2.1),
                                to_px(end.0, end.1),
                            ];
                            let _ = PolyBezierTo(hdc, &pts);
                        }
                        ContourOp::Close => {}
                    }
                }
                if EndPath(hdc).as_bool() {
                    let rgn = PathToRegion(hdc);
                    if !rgn.is_invalid() {
                        if SetWindowRgn(self.hwnd, Some(rgn), true) != 0 {
                            *self.last_region_key.borrow_mut() = Some(key);
                        } else {
                            let _ = DeleteObject(rgn.into());
                        }
                    }
                }
            }
            let _ = DeleteDC(hdc);
        }
    }

    /// Draws a premultiplied BGRA bitmap inside `rect` with optional rounded corner clipping.
    pub fn draw_bgra_bitmap(
        &self,
        rect: Rect,
        width: u32,
        height: u32,
        bgra_premul: &[u8],
        corner_radius: f32,
    ) -> Result<()> {
        let byte_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))
            .unwrap_or(0);
        if width == 0 || height == 0 || byte_len == 0 || bgra_premul.len() < byte_len {
            return Ok(());
        }
        unsafe {
            let key = (width, height, fast_hash_bgra(&bgra_premul[..byte_len]));
            let cached_bitmap = self.bitmap_cache.borrow().get(&key);
            let bitmap = if let Some(cached) = cached_bitmap {
                cached
            } else {
                let props = D2D1_BITMAP_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                };
                let created = self.d2d_target.CreateBitmap(
                    D2D_SIZE_U { width, height },
                    Some(bgra_premul.as_ptr() as *const _),
                    width * 4,
                    &props,
                )?;
                self.bitmap_cache.borrow_mut().insert(key, created.clone());
                created
            };
            let dest = rect.to_d2d();
            if corner_radius > 0.0 {
                let rr = D2D1_ROUNDED_RECT {
                    rect: dest,
                    radiusX: corner_radius,
                    radiusY: corner_radius,
                };
                let clip_geo = self.d2d_factory.CreateRoundedRectangleGeometry(&rr)?;
                let layer = {
                    let existing = self.cached_layer.borrow().clone();
                    if let Some(layer) = existing {
                        layer
                    } else {
                        let created = self.d2d_target.CreateLayer(None)?;
                        *self.cached_layer.borrow_mut() = Some(created.clone());
                        created
                    }
                };
                let mut layer_params = D2D1_LAYER_PARAMETERS {
                    contentBounds: dest,
                    geometricMask: std::mem::ManuallyDrop::new(Some(clip_geo.into())),
                    maskAntialiasMode: Default::default(),
                    maskTransform: windows_numerics::Matrix3x2::identity(),
                    opacity: 1.0,
                    opacityBrush: std::mem::ManuallyDrop::new(None),
                    layerOptions: Default::default(),
                };
                self.d2d_target.PushLayer(&layer_params, &layer);
                self.d2d_target.DrawBitmap(
                    &bitmap,
                    Some(&dest),
                    1.0,
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                    None,
                );
                self.d2d_target.PopLayer();
                std::mem::ManuallyDrop::drop(&mut layer_params.geometricMask);
                std::mem::ManuallyDrop::drop(&mut layer_params.opacityBrush);
            } else {
                self.d2d_target.DrawBitmap(
                    &bitmap,
                    Some(&dest),
                    1.0,
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                    None,
                );
            }
            Ok(())
        }
    }

    /// Draws a crisp vector Wi-Fi icon centered at `(cx, cy)`.
    pub fn draw_wifi_icon(
        &self,
        cx: f32,
        cy: f32,
        size: f32,
        connected: bool,
        off: bool,
        color: Color,
    ) -> Result<()> {
        let s = size.max(8.0);
        let stroke_w = (s * 0.12).max(1.2);
        let base_y = cy + s * 0.34;
        unsafe {
            let brush = self.create_brush(color)?;
            let dim_brush = self.create_brush(color.with_alpha(color.a * 0.3))?;
            let round_cap = self.create_round_cap_stroke()?;

            let active_brush = if connected && !off { &brush } else { &dim_brush };

            // Center dot
            self.d2d_target.FillEllipse(
                &D2D1_ELLIPSE {
                    point: pt(cx, base_y),
                    radiusX: stroke_w * 0.75,
                    radiusY: stroke_w * 0.75,
                },
                active_brush,
            );

            // Two concentric Wi-Fi arcs
            for factor in [0.36_f32, 0.66_f32] {
                let r = s * factor;
                let arc = self.build_quadratic_arc(
                    pt(cx - r * 0.72, base_y - r * 0.52),
                    pt(cx, base_y - r * 1.15),
                    pt(cx + r * 0.72, base_y - r * 0.52),
                )?;
                self.d2d_target
                    .DrawGeometry(&arc, active_brush, stroke_w, Some(&round_cap));
            }

            if off {
                self.d2d_target.DrawLine(
                    pt(cx - s * 0.38, cy + s * 0.34),
                    pt(cx + s * 0.38, cy - s * 0.34),
                    &brush,
                    stroke_w,
                    Some(&round_cap),
                );
            }
            Ok(())
        }
    }

    /// Draws a crisp vector battery icon inside `(x, y, w, h)` with optional charging bolt.
    pub fn draw_battery_icon(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        percent: u8,
        charging: bool,
    ) -> Result<()> {
        let bw = (w - 2.5).max(8.0);
        let bh = h.max(6.0);
        let pct = (percent.min(100) as f32) / 100.0;
        let fill_color = if charging {
            STATUS_SUCCESS
        } else if percent <= 20 {
            STATUS_WARNING
        } else {
            Color::from_rgb_u8(0xFF, 0xFF, 0xFF)
        };

        unsafe {
            let border_brush = self.create_brush(Color::from_rgba_u8(255, 255, 255, 160))?;
            let fill_brush = self.create_brush(fill_color)?;

            let body = D2D1_ROUNDED_RECT {
                rect: Rect::new(x, y, bw, bh).to_d2d(),
                radiusX: 2.5,
                radiusY: 2.5,
            };
            self.d2d_target.DrawRoundedRectangle(&body, &border_brush, 1.2, None);

            // Positive terminal nub
            let nub_h = (bh * 0.45).max(3.0);
            let nub = D2D1_ROUNDED_RECT {
                rect: Rect::new(x + bw + 0.5, y + (bh - nub_h) * 0.5, 2.0, nub_h).to_d2d(),
                radiusX: 0.8,
                radiusY: 0.8,
            };
            self.d2d_target.FillRoundedRectangle(&nub, &border_brush);

            // Inner charge bar
            let pad = 2.0;
            let max_fill_w = (bw - pad * 2.0).max(1.0);
            let fill_w = (max_fill_w * pct).max(if percent > 0 { 2.0 } else { 0.0 });
            if fill_w > 0.0 {
                let inner = D2D1_ROUNDED_RECT {
                    rect: Rect::new(x + pad, y + pad, fill_w, (bh - pad * 2.0).max(2.0)).to_d2d(),
                    radiusX: 1.2,
                    radiusY: 1.2,
                };
                self.d2d_target.FillRoundedRectangle(&inner, &fill_brush);
            }
            Ok(())
        }
    }

    /// Draws an equalizer/music visualizer of vertical pill bars.
    pub fn draw_music_visualizer(
        &self,
        x: f32,
        y: f32,
        h: f32,
        bars: &[f32],
        is_playing: bool,
    ) -> Result<()> {
        if bars.is_empty() {
            return Ok(());
        }
        let bar_w = 2.5_f32;
        let gap = 2.0_f32;
        let color = if is_playing { ACCENT_CYAN } else { STATUS_OFFLINE };
        unsafe {
            let brush = self.create_brush(color)?;
            for (i, &level) in bars.iter().enumerate() {
                let norm = if is_playing {
                    level.clamp(0.18, 1.0)
                } else {
                    0.22
                };
                let bh = (h * norm).max(3.0);
                let bx = x + (i as f32) * (bar_w + gap);
                let by = y + (h - bh) * 0.5;
                let rr = D2D1_ROUNDED_RECT {
                    rect: Rect::new(bx, by, bar_w, bh).to_d2d(),
                    radiusX: bar_w * 0.5,
                    radiusY: bar_w * 0.5,
                };
                self.d2d_target.FillRoundedRectangle(&rr, &brush);
            }
            Ok(())
        }
    }

    /// Draws a crisp sun icon (brightness) centered at `(cx, cy)`.
    pub fn draw_sun_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let r = (size * 0.24).max(2.0);
        let sw = (size * 0.1).max(1.1);
        self.stroke_circle(cx, cy, r, color, sw)?;
        let inner = r + size * 0.12;
        let outer = size * 0.48;
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::FRAC_PI_4;
            let (sin_a, cos_a) = angle.sin_cos();
            self.draw_line(
                cx + cos_a * inner,
                cy + sin_a * inner,
                cx + cos_a * outer,
                cy + sin_a * outer,
                color,
                sw,
            )?;
        }
        Ok(())
    }

    /// Draws a crisp speaker/volume icon centered at `(cx, cy)`.
    pub fn draw_volume_icon(
        &self,
        cx: f32,
        cy: f32,
        size: f32,
        muted: bool,
        level: f32,
        color: Color,
    ) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.11).max(1.1);
        unsafe {
            let brush = self.create_brush(color)?;
            let round_cap = self.create_round_cap_stroke()?;

            // Speaker cone polygon
            let cone = self.d2d_factory.CreatePathGeometry()?;
            let sink = cone.Open()?;
            sink.BeginFigure(pt(cx - s * 0.38, cy - s * 0.16), D2D1_FIGURE_BEGIN_FILLED);
            sink.AddLine(pt(cx - s * 0.18, cy - s * 0.16));
            sink.AddLine(pt(cx + s * 0.04, cy - s * 0.36));
            sink.AddLine(pt(cx + s * 0.04, cy + s * 0.36));
            sink.AddLine(pt(cx - s * 0.18, cy + s * 0.16));
            sink.AddLine(pt(cx - s * 0.38, cy + s * 0.16));
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            self.d2d_target.FillGeometry(&cone, &brush, None);

            if muted || level <= 0.01 {
                self.d2d_target.DrawLine(
                    pt(cx + s * 0.18, cy - s * 0.18),
                    pt(cx + s * 0.44, cy + s * 0.18),
                    &brush,
                    sw,
                    Some(&round_cap),
                );
                self.d2d_target.DrawLine(
                    pt(cx + s * 0.44, cy - s * 0.18),
                    pt(cx + s * 0.18, cy + s * 0.18),
                    &brush,
                    sw,
                    Some(&round_cap),
                );
            } else {
                let arc1 = self.build_quadratic_arc(
                    pt(cx + s * 0.18, cy - s * 0.18),
                    pt(cx + s * 0.32, cy),
                    pt(cx + s * 0.18, cy + s * 0.18),
                )?;
                self.d2d_target
                    .DrawGeometry(&arc1, &brush, sw, Some(&round_cap));
                if level > 0.45 {
                    let arc2 = self.build_quadratic_arc(
                        pt(cx + s * 0.28, cy - s * 0.30),
                        pt(cx + s * 0.48, cy),
                        pt(cx + s * 0.28, cy + s * 0.30),
                    )?;
                    self.d2d_target
                        .DrawGeometry(&arc2, &brush, sw, Some(&round_cap));
                }
            }
            Ok(())
        }
    }

    /// Draws a crisp Bluetooth rune icon centered at `(cx, cy)`.
    pub fn draw_bluetooth_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.11).max(1.1);
        let w = s * 0.26;
        let h = s * 0.42;
        self.draw_line(cx, cy - h, cx, cy + h, color, sw)?;
        self.draw_line(cx, cy - h, cx + w, cy - h * 0.45, color, sw)?;
        self.draw_line(cx + w, cy - h * 0.45, cx - w, cy + h * 0.45, color, sw)?;
        self.draw_line(cx, cy + h, cx + w, cy + h * 0.45, color, sw)?;
        self.draw_line(cx + w, cy + h * 0.45, cx - w, cy - h * 0.45, color, sw)?;
        Ok(())
    }

    /// Draws a crisp calendar icon centered at `(cx, cy)`.
    pub fn draw_calendar_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.11).max(1.1);
        let rect = Rect::new(cx - s * 0.4, cy - s * 0.34, s * 0.8, s * 0.72);
        self.stroke_rounded_rect(rect, 2.0, color, sw)?;
        self.draw_line(rect.x, rect.y + s * 0.24, rect.x + rect.width, rect.y + s * 0.24, color, sw)?;
        self.draw_line(cx - s * 0.18, rect.y - s * 0.1, cx - s * 0.18, rect.y + s * 0.08, color, sw)?;
        self.draw_line(cx + s * 0.18, rect.y - s * 0.1, cx + s * 0.18, rect.y + s * 0.08, color, sw)?;
        Ok(())
    }

    /// Draws a `<` or `>` chevron icon centered at `(cx, cy)`.
    pub fn draw_chevron_icon(
        &self,
        cx: f32,
        cy: f32,
        size: f32,
        dir_right: bool,
        color: Color,
    ) -> Result<()> {
        let s = size.max(6.0);
        let sw = (s * 0.15).max(1.2);
        let dx = if dir_right { s * 0.22 } else { -s * 0.22 };
        self.draw_line(cx - dx, cy - s * 0.32, cx + dx, cy, color, sw)?;
        self.draw_line(cx + dx, cy, cx - dx, cy + s * 0.32, color, sw)?;
        Ok(())
    }

    /// Draws a play triangle or pause double-bar icon centered at `(cx, cy)`.
    pub fn draw_play_pause_icon(
        &self,
        cx: f32,
        cy: f32,
        size: f32,
        is_playing: bool,
        color: Color,
    ) -> Result<()> {
        let s = size.max(8.0);
        if is_playing {
            let bar_w = (s * 0.22).max(2.0);
            let bar_h = s * 0.76;
            self.fill_rounded_rect(
                Rect::new(cx - bar_w * 1.4, cy - bar_h * 0.5, bar_w, bar_h),
                1.0,
                color,
            )?;
            self.fill_rounded_rect(
                Rect::new(cx + bar_w * 0.4, cy - bar_h * 0.5, bar_w, bar_h),
                1.0,
                color,
            )?;
            return Ok(());
        }
        unsafe {
            let brush = self.create_brush(color)?;
            let geo = self.d2d_factory.CreatePathGeometry()?;
            let sink = geo.Open()?;
            sink.BeginFigure(pt(cx - s * 0.28, cy - s * 0.38), D2D1_FIGURE_BEGIN_FILLED);
            sink.AddLine(pt(cx + s * 0.38, cy));
            sink.AddLine(pt(cx - s * 0.28, cy + s * 0.38));
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            self.d2d_target.FillGeometry(&geo, &brush, None);
            Ok(())
        }
    }

    /// Draws a skip-next (`>>|`) or skip-previous (`|<<`) icon centered at `(cx, cy)`.
    pub fn draw_skip_icon(
        &self,
        cx: f32,
        cy: f32,
        size: f32,
        next: bool,
        color: Color,
    ) -> Result<()> {
        let s = size.max(8.0);
        let dir = if next { 1.0_f32 } else { -1.0_f32 };
        let bar_w = (s * 0.14).max(1.6);
        let bar_x = if next { cx + s * 0.24 } else { cx - s * 0.24 - bar_w };
        self.fill_rounded_rect(Rect::new(bar_x, cy - s * 0.32, bar_w, s * 0.64), 0.8, color)?;
        unsafe {
            let brush = self.create_brush(color)?;
            let geo = self.d2d_factory.CreatePathGeometry()?;
            let sink = geo.Open()?;
            sink.BeginFigure(pt(cx - dir * s * 0.32, cy - s * 0.32), D2D1_FIGURE_BEGIN_FILLED);
            sink.AddLine(pt(cx + dir * s * 0.18, cy));
            sink.AddLine(pt(cx - dir * s * 0.32, cy + s * 0.32));
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            self.d2d_target.FillGeometry(&geo, &brush, None);
            Ok(())
        }
    }

    /// Draws a compact keyboard icon centered at `(cx, cy)`.
    pub fn draw_keyboard_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.1).max(1.0);
        let rect = Rect::new(cx - s * 0.45, cy - s * 0.28, s * 0.9, s * 0.56);
        self.stroke_rounded_rect(rect, 2.0, color, sw)?;
        for dx in [-0.25_f32, 0.0, 0.25] {
            self.fill_circle(cx + s * dx, cy - s * 0.08, 0.9, color)?;
        }
        self.draw_line(cx - s * 0.22, cy + s * 0.12, cx + s * 0.22, cy + s * 0.12, color, sw)?;
        Ok(())
    }

    /// Draws a music note icon centered at `(cx, cy)`.
    pub fn draw_music_note_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.12).max(1.2);
        self.fill_circle(cx - s * 0.14, cy + s * 0.22, s * 0.16, color)?;
        self.draw_line(cx - s * 0.01, cy + s * 0.22, cx - s * 0.01, cy - s * 0.32, color, sw)?;
        self.draw_line(cx - s * 0.01, cy - s * 0.32, cx + s * 0.26, cy - s * 0.22, color, sw)?;
        Ok(())
    }

    /// Draws a compact settings gear icon centered at `(cx, cy)`.
    pub fn draw_settings_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.12).max(1.1);
        self.stroke_circle(cx, cy, s * 0.22, color, sw)?;
        for i in 0..6 {
            let a = (i as f32) * (std::f32::consts::TAU / 6.0);
            let (sa, ca) = a.sin_cos();
            self.draw_line(cx + ca * s * 0.28, cy + sa * s * 0.28, cx + ca * s * 0.42, cy + sa * s * 0.42, color, sw)?;
        }
        Ok(())
    }

    /// Draws a file-shelf inbox tray icon centered at `(cx, cy)`.
    pub fn draw_shelf_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.11).max(1.0);
        let rect = Rect::new(cx - s * 0.4, cy - s * 0.32, s * 0.8, s * 0.64);
        self.stroke_rounded_rect(rect, 2.0, color, sw)?;
        self.draw_line(rect.x, cy + s * 0.06, rect.x + rect.width, cy + s * 0.06, color, sw)?;
        Ok(())
    }

    /// Draws a 4-point sparkle icon for Hime AI centered at `(cx, cy)`.
    pub fn draw_sparkles_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.12).max(1.1);
        self.draw_line(cx, cy - s * 0.42, cx, cy + s * 0.42, color, sw)?;
        self.draw_line(cx - s * 0.42, cy, cx + s * 0.42, cy, color, sw)?;
        self.fill_circle(cx, cy, s * 0.14, color)?;
        Ok(())
    }

    /// Draws a notification bell icon centered at `(cx, cy)`.
    pub fn draw_bell_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.11).max(1.0);
        self.stroke_circle(cx, cy - s * 0.05, s * 0.25, color, sw)?;
        self.draw_line(cx - s * 0.34, cy + s * 0.22, cx + s * 0.34, cy + s * 0.22, color, sw)?;
        self.fill_circle(cx, cy + s * 0.34, s * 0.08, color)?;
        Ok(())
    }

    /// Draws a 2x2 / dock pill grid icon centered at `(cx, cy)`.
    pub fn draw_dock_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.1).max(1.0);
        let rect = Rect::new(cx - s * 0.42, cy - s * 0.32, s * 0.84, s * 0.64);
        self.stroke_rounded_rect(rect, 2.2, color, sw)?;
        self.draw_line(cx - s * 0.22, cy + s * 0.14, cx + s * 0.22, cy + s * 0.14, color, sw * 1.3)?;
        Ok(())
    }

    unsafe fn create_round_cap_stroke(&self) -> Result<ID2D1StrokeStyle> {
        if let Some(cached) = self.round_cap_stroke.borrow().as_ref() {
            return Ok(cached.clone());
        }
        let stroke_props = D2D1_STROKE_STYLE_PROPERTIES1 {
            startCap: D2D1_CAP_STYLE_ROUND,
            endCap: D2D1_CAP_STYLE_ROUND,
            ..Default::default()
        };
        let style: ID2D1StrokeStyle = self.d2d_factory.CreateStrokeStyle(&stroke_props, None)?.into();
        *self.round_cap_stroke.borrow_mut() = Some(style.clone());
        Ok(style)
    }

    unsafe fn build_contour_geometry(&self, ops: &[ContourOp]) -> Result<ID2D1PathGeometry> {
        let geo = self.d2d_factory.CreatePathGeometry()?;
        let sink = geo.Open()?;
        let mut started = false;
        for op in ops {
            match *op {
                ContourOp::MoveTo(x, y) => {
                    sink.BeginFigure(pt(x, y), D2D1_FIGURE_BEGIN_FILLED);
                    started = true;
                }
                ContourOp::LineTo(x, y) => {
                    sink.AddLine(pt(x, y));
                }
                ContourOp::CubicTo { cp1, cp2, end } => {
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(cp1.0, cp1.1),
                        point2: pt(cp2.0, cp2.1),
                        point3: pt(end.0, end.1),
                    });
                }
                ContourOp::Close => {
                    if started {
                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        started = false;
                    }
                }
            }
        }
        if started {
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        }
        sink.Close()?;
        Ok(geo.into())
    }

    unsafe fn create_brush(&self, color: Color) -> Result<ID2D1SolidColorBrush> {
        let key = pack_color_rgba(color);
        if let Some(cached) = self.brush_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }
        let d2d_color: D2D1_COLOR_F = color.to_d2d();
        let brush = self.d2d_target.CreateSolidColorBrush(&d2d_color, None)?;
        let mut cache = self.brush_cache.borrow_mut();
        if cache.len() >= MAX_BRUSH_CACHE_ENTRIES {
            cache.clear();
        }
        cache.insert(key, brush.clone());
        Ok(brush)
    }

    unsafe fn create_text_format_weighted(
        &self,
        font_size: f32,
        align: TextAlign,
        bold: bool,
    ) -> Result<IDWriteTextFormat> {
        let clamped_size = font_size.max(1.0);
        let font_size_tenths = (clamped_size * 10.0).round() as u32;
        let align_key = match align {
            TextAlign::Leading => 0,
            TextAlign::Center => 1,
            TextAlign::Trailing => 2,
        };
        let key = (font_size_tenths, align_key, bold);
        if let Some(cached) = self.text_format_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }

        let family = to_wide_null(FONT_FAMILY);
        let locale = to_wide_null("en-US");
        let weight = if bold {
            DWRITE_FONT_WEIGHT_BOLD
        } else {
            DWRITE_FONT_WEIGHT_SEMI_BOLD
        };
        let format = self.dwrite_factory.CreateTextFormat(
            PCWSTR(family.as_ptr()),
            None,
            weight,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            clamped_size,
            PCWSTR(locale.as_ptr()),
        )?;
        let dwrite_align = match align {
            TextAlign::Leading => DWRITE_TEXT_ALIGNMENT_LEADING,
            TextAlign::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
            TextAlign::Trailing => DWRITE_TEXT_ALIGNMENT_TRAILING,
        };
        format.SetTextAlignment(dwrite_align)?;
        format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
        self.text_format_cache
            .borrow_mut()
            .insert(key, format.clone());
        Ok(format)
    }

    unsafe fn build_ear_geometry(
        &self,
        corner_x: f32,
        top_y: f32,
        r: f32,
        is_left: bool,
    ) -> Result<ID2D1PathGeometry> {
        let geo = self.d2d_factory.CreatePathGeometry()?;
        let sink = geo.Open()?;
        let dir = if is_left { -1.0 } else { 1.0 };
        let start = pt(corner_x, top_y);
        sink.BeginFigure(start, D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(pt(corner_x + dir * r, top_y));
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: pt(corner_x + dir * r * 0.45, top_y),
            point2: pt(corner_x, top_y + r * 0.45),
            point3: pt(corner_x, top_y + r),
        });
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        Ok(geo.into())
    }

    unsafe fn build_quadratic_arc(
        &self,
        p0: D2D_POINT_2F,
        ctrl: D2D_POINT_2F,
        p2: D2D_POINT_2F,
    ) -> Result<ID2D1PathGeometry> {
        let geo = self.d2d_factory.CreatePathGeometry()?;
        let sink = geo.Open()?;
        sink.BeginFigure(p0, D2D1_FIGURE_BEGIN_HOLLOW);
        // Elevate quadratic Bezier to cubic Bezier for Direct2D.
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: D2D_POINT_2F {
                X: p0.X + (2.0 / 3.0) * (ctrl.X - p0.X),
                Y: p0.Y + (2.0 / 3.0) * (ctrl.Y - p0.Y),
            },
            point2: D2D_POINT_2F {
                X: p2.X + (2.0 / 3.0) * (ctrl.X - p2.X),
                Y: p2.Y + (2.0 / 3.0) * (ctrl.Y - p2.Y),
            },
            point3: p2,
        });
        sink.EndFigure(D2D1_FIGURE_END_OPEN);
        sink.Close()?;
        Ok(geo.into())
    }

    fn resize_swap_chain(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            self.brush_cache.borrow_mut().clear();
            self.bitmap_cache.borrow_mut().clear();
            self.cached_layer.borrow_mut().take();
            self.last_region_key.borrow_mut().take();
            self.d2d_target.Resize(&D2D_SIZE_U { width, height })?;
            self.width = width;
            self.height = height;
            self.dpi = get_window_dpi(self.hwnd);
            self.d2d_target.SetDpi(self.dpi, self.dpi);
            Ok(())
        }
    }
}

impl Drop for NativeSurface {
    fn drop(&mut self) {
        self.brush_cache.borrow_mut().clear();
        self.bitmap_cache.borrow_mut().clear();
        self.cached_layer.borrow_mut().take();
        unsafe {
            if !self.hwnd.0.is_null() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

fn register_window_class(hinstance: HINSTANCE) {
    CLASS_REGISTERED.get_or_init(|| unsafe {
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(surface_wnd_proc),
            hInstance: hinstance,
            hCursor: cursor,
            lpszClassName: WINDOW_CLASS_NAME,
            ..Default::default()
        };
        let _ = RegisterClassExW(&wc);
    });
}

unsafe extern "system" fn surface_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SETCURSOR && ((lparam.0 & 0xFFFF) as u32) == HTCLIENT {
        if let Ok(arrow) = LoadCursorW(None, IDC_ARROW) {
            let _ = SetCursor(Some(arrow));
        }
        return LRESULT(1);
    }

    let kind = match msg {
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            Some(SurfaceEventKind::MouseMove { x, y })
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            Some(SurfaceEventKind::LeftButtonDown { x, y })
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            Some(SurfaceEventKind::LeftButtonUp { x, y })
        }
        WM_DROPFILES => Some(SurfaceEventKind::DropFiles { wparam: wparam.0 }),
        _ => None,
    };

    if let Some(kind) = kind {
        SURFACE_EVENTS.with(|cell| {
            cell.borrow_mut().push(SurfaceEvent {
                hwnd: hwnd.0 as isize,
                kind,
            });
        });
        let target = EVENT_TARGET_HWND.load(Ordering::Relaxed);
        if target != 0 {
            let _ = PostMessageW(
                Some(HWND(target as *mut std::ffi::c_void)),
                WM_SURFACE_EVENT,
                WPARAM(0),
                LPARAM(0),
            );
        }
        return LRESULT(0);
    }

    if msg == WM_PAINT {
        let _ = ValidateRect(Some(hwnd), None);
        return LRESULT(0);
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

#[inline]
const fn pt(x: f32, y: f32) -> D2D_POINT_2F {
    D2D_POINT_2F { X: x, Y: y }
}

#[inline]
fn pack_color_rgba(color: Color) -> u32 {
    let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

#[inline]
fn fast_hash_bgra(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let word = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        hash ^= word;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    for &b in chunks.remainder() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

#[inline]
fn edge_region_hash(
    scale: f32,
    rect: Rect,
    ear_w: f32,
    radius: f32,
    edge: EdgeSide,
    flares: bool,
) -> u64 {
    let norm = |v: f32| if v == 0.0 { 0.0_f32 } else { v };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let vals = [
        norm(scale).to_bits(),
        norm(rect.x).to_bits(),
        norm(rect.y).to_bits(),
        norm(rect.width).to_bits(),
        norm(rect.height).to_bits(),
        norm(ear_w).to_bits(),
        norm(radius).to_bits(),
        match edge {
            EdgeSide::Top => 0,
            EdgeSide::Bottom => 1,
            EdgeSide::Left => 2,
            EdgeSide::Right => 3,
        },
        u32::from(flares),
    ];
    for v in vals {
        hash ^= v as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

fn get_window_dpi(hwnd: HWND) -> f32 {
    unsafe {
        if let Ok(user32) = GetModuleHandleW(w!("user32.dll")) {
            if let Some(proc_addr) = GetProcAddress(user32, s!("GetDpiForWindow")) {
                let func: unsafe extern "system" fn(HWND) -> u32 = std::mem::transmute(proc_addr);
                let dpi = func(hwnd);
                if dpi >= 48 {
                    return dpi as f32;
                }
            }
        }
    }
    96.0
}

fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Converts a Win32 HRESULT into a `windows::core::Result<()>`.
#[allow(dead_code)]
fn check_hr(hr: HRESULT) -> Result<()> {
    hr.ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme;

    #[test]
    fn hime_expression_serde_roundtrip() {
        let variants = [
            HimeExpression::Netral,
            HimeExpression::Senang,
            HimeExpression::Fokus,
            HimeExpression::Khawatir,
            HimeExpression::Offline,
            HimeExpression::Waspada,
        ];
        for expr in variants {
            let json = serde_json::to_string(&expr).unwrap();
            let parsed: HimeExpression = serde_json::from_str(&json).unwrap();
            assert_eq!(expr, parsed);
        }
        assert_eq!(theme::CORNER_RADIUS, 18.0);
    }

    #[test]
    fn native_surface_uses_hwnd_render_target_and_renders() {
        let mut surface = match NativeSurface::new("HimeTestSurface", -2000, -2000, 160, 48) {
            Ok(s) => s,
            Err(_) => return,
        };
        let ex_style = unsafe { GetWindowLongW(surface.hwnd(), GWL_EXSTYLE) } as u32;
        assert_ne!(ex_style & WS_EX_TOPMOST.0, 0);
        assert_ne!(ex_style & WS_EX_TOOLWINDOW.0, 0);
        assert_ne!(ex_style & WS_EX_NOACTIVATE.0, 0);

        surface.set_click_through(true);
        let ct_style = unsafe { GetWindowLongW(surface.hwnd(), GWL_EXSTYLE) } as u32;
        assert_ne!(ct_style & WS_EX_TRANSPARENT.0, 0);

        surface.set_click_through(false);
        let nct_style = unsafe { GetWindowLongW(surface.hwnd(), GWL_EXSTYLE) } as u32;
        assert_eq!(nct_style & WS_EX_TRANSPARENT.0, 0);

        surface.set_bounds(-2000, -2000, 540, 200).expect("set_bounds should resize target");
        assert_eq!(surface.size(), (540, 200));

        let notch = crate::notch::NotchView::default();
        surface.begin_draw();
        notch.render(&surface).expect("NotchView::render should succeed");
        surface.end_draw_and_present().expect("notch end_draw_and_present");

        let mut dock = crate::dock::DockBar::new();
        dock.add_item(crate::dock::DockItem::new("term", "Terminal", "wt.exe", true));
        surface.begin_draw();
        dock.render(&surface).expect("DockBar::render should succeed");
        surface.end_draw_and_present().expect("dock end_draw_and_present");

        surface.begin_draw();
        surface.draw_line(10.0, 10.0, 40.0, 40.0, crate::theme::TEXT_PRIMARY, 1.5).unwrap();
        surface.fill_circle(60.0, 24.0, 8.0, crate::theme::ACCENT_CYAN).unwrap();
        surface.stroke_circle(84.0, 24.0, 8.0, crate::theme::TEXT_PRIMARY, 1.2).unwrap();
        surface.draw_circle(108.0, 24.0, 8.0, crate::theme::TEXT_PRIMARY, false, 1.0).unwrap();
        surface.draw_sun_icon(132.0, 24.0, 14.0, crate::theme::TEXT_PRIMARY).unwrap();
        surface.draw_volume_icon(156.0, 24.0, 14.0, false, 0.8, crate::theme::TEXT_PRIMARY).unwrap();
        surface.draw_bluetooth_icon(180.0, 24.0, 14.0, crate::theme::TEXT_PRIMARY).unwrap();
        surface.draw_calendar_icon(204.0, 24.0, 14.0, crate::theme::TEXT_PRIMARY).unwrap();
        surface.draw_text_bold("16:50", Rect::new(220.0, 10.0, 60.0, 28.0), 14.0, crate::theme::TEXT_PRIMARY, TextAlign::Center).unwrap();
        surface.end_draw_and_present().expect("vector icon helpers");
    }

    #[test]
    fn edge_contour_points_match_top_flares_and_transforms() {
        let ops = edge_contour_points(Rect::new(0.0, 0.0, 320.0, 38.0), 13.0, 18.0, EdgeSide::Top, true);
        assert_eq!(ops.first(), Some(&ContourOp::MoveTo(0.0, 0.0)));
        assert_eq!(
            ops.get(1),
            Some(&ContourOp::CubicTo {
                cp1: (10.0, 0.0),
                cp2: (13.0, 3.0),
                end: (13.0, 13.0),
            })
        );
        assert_eq!(ops.last(), Some(&ContourOp::Close));

        let left_ops =
            edge_contour_points(Rect::new(0.0, 0.0, 54.0, 220.0), 13.0, 18.0, EdgeSide::Left, true);
        assert_eq!(left_ops.first(), Some(&ContourOp::MoveTo(0.0, 0.0)));
        assert_eq!(
            left_ops.get(1),
            Some(&ContourOp::CubicTo {
                cp1: (0.0, 10.0),
                cp2: (3.0, 13.0),
                end: (13.0, 13.0),
            })
        );
    }

    #[test]
    fn extract_window_or_exe_icon_reads_system_executable() {
        let icon = extract_window_or_exe_icon(0, r"C:\Windows\explorer.exe", 32)
            .or_else(|| extract_window_or_exe_icon(0, r"C:\Windows\System32\notepad.exe", 32));
        if let Some(ic) = icon {
            assert_eq!(ic.width, 32);
            assert_eq!(ic.height, 32);
            assert_eq!(ic.bgra_premul.len(), 32 * 32 * 4);
            assert!(ic.bgra_premul.chunks_exact(4).any(|px| px[3] > 0));
        }
    }

    #[test]
    fn caches_factories_brushes_formats_bitmaps_layers_and_regions() {
        let mut s1 = match NativeSurface::new("HimeCacheTest1", -2000, -2000, 120, 48) {
            Ok(s) => s,
            Err(_) => return,
        };
        let s2 = match NativeSurface::new("HimeCacheTest2", -2000, -2000, 120, 48) {
            Ok(s) => s,
            Err(_) => return,
        };
        assert_eq!(s1.d2d_factory(), s2.d2d_factory());
        assert_eq!(s1.dwrite_factory(), s2.dwrite_factory());

        let px = vec![0x80_u8; 16 * 16 * 4];
        s1.begin_draw();
        s1.draw_edge_surface(
            Rect::new(0.0, 0.0, 120.0, 48.0),
            13.0,
            18.0,
            EdgeSide::Top,
            true,
            theme::NOTCH_BG_SOLID,
            Some(theme::GLASS_BORDER),
        )
        .unwrap();
        s1.draw_text("Hello", Rect::new(8.0, 8.0, 80.0, 20.0), 13.0, theme::TEXT_PRIMARY, TextAlign::Leading)
            .unwrap();
        s1.draw_text("World", Rect::new(8.0, 24.0, 80.0, 20.0), 13.0, theme::TEXT_PRIMARY, TextAlign::Leading)
            .unwrap();
        s1.draw_bgra_bitmap(Rect::new(90.0, 8.0, 16.0, 16.0), 16, 16, &px, 4.0)
            .unwrap();
        s1.draw_bgra_bitmap(Rect::new(90.0, 26.0, 16.0, 16.0), 16, 16, &px, 4.0)
            .unwrap();
        s1.end_draw_and_present().unwrap();

        assert_eq!(s1.brush_cache.borrow().len(), 3);
        assert_eq!(s1.text_format_cache.borrow().len(), 1);
        assert_eq!(s1.bitmap_cache.borrow().entries.len(), 1);
        assert!(s1.cached_layer.borrow().is_some());

        s1.set_rounded_region(Rect::new(0.0, 0.0, 120.0, 48.0), 18.0);
        let first_rgn = *s1.last_region_key.borrow();
        assert!(matches!(first_rgn, Some(RegionKey::Rounded { .. })));
        s1.set_rounded_region(Rect::new(0.0, 0.0, 120.0, 48.0), 18.0);
        assert_eq!(*s1.last_region_key.borrow(), first_rgn);

        s1.set_edge_surface_region(Rect::new(0.0, 0.0, 120.0, 48.0), 13.0, 18.0, EdgeSide::Top, true);
        assert!(matches!(*s1.last_region_key.borrow(), Some(RegionKey::Edge(_))));

        // Resize clears device-dependent caches and region key.
        s1.set_bounds(-2000, -2000, 180, 60).unwrap();
        assert_eq!(s1.brush_cache.borrow().len(), 0);
        assert_eq!(s1.bitmap_cache.borrow().entries.len(), 0);
        assert!(s1.cached_layer.borrow().is_none());
        assert!(s1.last_region_key.borrow().is_none());

        // Bitmap cache stays bounded at 64 entries.
        s1.begin_draw();
        for i in 0..70_u32 {
            let mut img = vec![0x40_u8; 4 * 4 * 4];
            img[0] = (i & 0xFF) as u8;
            s1.draw_bgra_bitmap(Rect::new(0.0, 0.0, 4.0, 4.0), 4, 4, &img, 0.0)
                .unwrap();
        }
        s1.end_draw_and_present().unwrap();
        assert_eq!(s1.bitmap_cache.borrow().entries.len(), MAX_BITMAP_CACHE_ENTRIES);

        trim_working_set();
    }
}
