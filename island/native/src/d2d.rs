//! Direct2D + DirectWrite + Layered 32-bit Per-Pixel Alpha (`UpdateLayeredWindow`) surface
//! with real Liquid Glass backdrop Gaussian blur (`frost`), SDF bevel refraction, chromatic
//! edge dispersion, adaptive Light/Dark Glass contrast, and Windows WIC UWP icon extraction.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use windows::core::{s, w, GUID, HRESULT, Interface, PCWSTR, Result};
use windows_numerics::Vector2 as D2D_POINT_2F;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BEZIER_SEGMENT, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
    D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D1_PIXEL_FORMAT,
    D2D_RECT_F, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Bitmap, ID2D1DCRenderTarget, ID2D1Factory1, ID2D1Layer,
    ID2D1PathGeometry, ID2D1SolidColorBrush, ID2D1StrokeStyle,
    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_PROPERTIES, D2D1_CAP_STYLE_ROUND,
    D2D1_ELLIPSE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_LAYER_PARAMETERS, D2D1_LINE_JOIN_ROUND,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_SOFTWARE, D2D1_ROUNDED_RECT,
    D2D1_STROKE_STYLE_PROPERTIES1,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateRectRgn, DeleteDC, DeleteObject, SelectObject,
    SetWindowRgn, ValidateRect, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow, DrawIconEx, GetClassLongPtrW,
    GetIconInfo, GetWindowLongW, LoadCursorW, PostMessageW, RegisterClassExW, SendMessageTimeoutW,
    SetCursor, SetWindowLongW, SetWindowPos, ShowWindow, UpdateLayeredWindow, CS_HREDRAW,
    CS_VREDRAW, DI_NORMAL, GCLP_HICON, GCLP_HICONSM, GWL_EXSTYLE, HICON, HTCLIENT, HWND_TOPMOST,
    ICONINFO, ICON_BIG, ICON_SMALL, ICON_SMALL2, IDC_ARROW, SMTO_ABORTIFHUNG, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA,
    WM_APP, WM_DROPFILES, WM_GETICON, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT,
    WM_SETCURSOR, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::theme::{
    AdaptivePalette, Color, ColorThemeMode, ThemePolarity, FONT_FAMILY, STATUS_LIVE_AMBER,
    STATUS_LIVE_GREEN, STATUS_LIVE_RED, STATUS_MUTED, SYSTEM_BLUE, SYSTEM_BLUE_TINT, TRANSPARENT,
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
        ear_w.max(0.0).min(lw * 0.25).min((lh * 0.42).max(0.0))
    } else {
        0.0
    };
    let left = ear;
    let right = (lw - ear).max(left);
    let r = radius
        .max(0.0)
        .min((lh - ear).max(0.0))
        .min(lh * 0.5)
        .min((right - left) * 0.5);
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
        }

        // For UWP apps (WindowsApps / AppxManifest / Assets), CoreWindow's GCLP_HICON is the
        // generic IDI_APPLICATION page icon; resolve the real package PNG before class HICON.
        if hicon.0.is_null() && is_uwp_package_path(exe_path) {
            if let Some(uwp_icon) = extract_uwp_package_icon(exe_path, target_size) {
                return Some(uwp_icon);
            }
        }

        if hicon.0.is_null() && hwnd != 0 {
            let wnd = HWND(hwnd as *mut std::ffi::c_void);
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
            if !exe_path.is_empty() {
                return extract_uwp_package_icon(exe_path, target_size);
            }
            return None;
        }

        let extracted = icon_to_bgra_premul(hicon, target_size);
        if owned_icon {
            let _ = DestroyIcon(hicon);
        }
        extracted.or_else(|| {
            if !exe_path.is_empty() {
                extract_uwp_package_icon(exe_path, target_size)
            } else {
                None
            }
        })
    }
}

/// Decodes raw encoded image bytes (e.g. PNG from Windows `IconSnapshot`) via WIC
/// into `target_w x target_h` 32-bit premultiplied BGRA pixels.
pub fn decode_image_bytes_bgra_premul(
    bytes: &[u8],
    target_w: u32,
    target_h: u32,
) -> Option<IconRgba> {
    if bytes.is_empty() || target_w == 0 || target_h == 0 {
        return None;
    }
    let len_u32 = u32::try_from(bytes.len()).ok()?;
    #[link(name = "shlwapi")]
    extern "system" {
        fn SHCreateMemStream(p_init: *const u8, cb_init: u32) -> *mut std::ffi::c_void;
    }

    decode_via_wic(target_w, target_h, |factory_ptr, f_vtbl| unsafe {
        let stream_ptr = SHCreateMemStream(bytes.as_ptr(), len_u32);
        if stream_ptr.is_null() {
            return None;
        }
        let stream = WicComPtr(stream_ptr);
        // IWICImagingFactory::CreateDecoderFromStream (slot 4)
        let create_decoder_stream: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *mut std::ffi::c_void,
            *const GUID,
            u32,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT = std::mem::transmute(*f_vtbl.add(4));
        let mut decoder_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        if create_decoder_stream(factory_ptr, stream.0, std::ptr::null(), 0, &mut decoder_ptr)
            .is_err()
            || decoder_ptr.is_null()
        {
            return None;
        }
        Some(decoder_ptr)
    })
}

/// Decodes an image file (PNG/BMP/JPG) via Windows Imaging Component (`IWICImagingFactory`)
/// into `target_w x target_h` 32-bit premultiplied BGRA pixels.
pub fn decode_image_bgra_premul(
    path: &Path,
    target_w: u32,
    target_h: u32,
) -> Option<IconRgba> {
    if target_w == 0 || target_h == 0 || !path.exists() {
        return None;
    }
    let path_str = path.to_str()?;
    let wide_path = to_wide_null(path_str);
    const GENERIC_READ: u32 = 0x8000_0000;
    const WIC_DECODE_METADATA_CACHE_ON_DEMAND: u32 = 0;

    decode_via_wic(target_w, target_h, |factory_ptr, f_vtbl| unsafe {
        // IWICImagingFactory::CreateDecoderFromFilename (slot 3)
        let create_decoder: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            PCWSTR,
            *const GUID,
            u32,
            u32,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT = std::mem::transmute(*f_vtbl.add(3));
        let mut decoder_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        if create_decoder(
            factory_ptr,
            PCWSTR(wide_path.as_ptr()),
            std::ptr::null(),
            GENERIC_READ,
            WIC_DECODE_METADATA_CACHE_ON_DEMAND,
            &mut decoder_ptr,
        )
        .is_err()
            || decoder_ptr.is_null()
        {
            return None;
        }
        Some(decoder_ptr)
    })
}

struct WicComPtr(*mut std::ffi::c_void);
impl Drop for WicComPtr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let vtbl = *(self.0 as *const *const usize);
                let release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32 =
                    std::mem::transmute(*vtbl.add(2));
                release(self.0);
            }
        }
    }
}

fn decode_via_wic<F>(target_w: u32, target_h: u32, make_decoder: F) -> Option<IconRgba>
where
    F: FnOnce(*mut std::ffi::c_void, *const usize) -> Option<*mut std::ffi::c_void>,
{
    // Verified against wincodec.h
    const CLSID_WIC_IMAGING_FACTORY: GUID = GUID::from_u128(0xcacaf262_9370_4615_a13b_9f5539da4c0a);
    const IID_IWIC_IMAGING_FACTORY: GUID = GUID::from_u128(0xec5ec8a9_c395_4314_9c77_54d7a935ff70);
    const GUID_WIC_32BPP_PBGRA: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc910);
    const WIC_DITHER_NONE: u32 = 0;
    const WIC_PALETTE_CUSTOM: u32 = 0;
    const WIC_INTERP_FANT: u32 = 3;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let factory_unk: windows::core::IUnknown =
            CoCreateInstance(&CLSID_WIC_IMAGING_FACTORY, None, CLSCTX_ALL).ok()?;
        let mut wic_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        if factory_unk.query(&IID_IWIC_IMAGING_FACTORY, &mut wic_ptr).is_err() || wic_ptr.is_null()
        {
            return None;
        }
        let factory = WicComPtr(wic_ptr);
        let f_vtbl = *(factory.0 as *const *const usize);

        let decoder_ptr = make_decoder(factory.0, f_vtbl)?;
        let decoder = WicComPtr(decoder_ptr);
        let d_vtbl = *(decoder.0 as *const *const usize);

        // IWICBitmapDecoder::GetFrame (slot 13)
        let get_frame: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            u32,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT = std::mem::transmute(*d_vtbl.add(13));
        let mut frame_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        if get_frame(decoder.0, 0, &mut frame_ptr).is_err() || frame_ptr.is_null() {
            return None;
        }
        let frame = WicComPtr(frame_ptr);

        // IWICImagingFactory::CreateBitmapScaler (slot 11)
        let create_scaler: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT = std::mem::transmute(*f_vtbl.add(11));
        let mut scaler_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        if create_scaler(factory.0, &mut scaler_ptr).is_err() || scaler_ptr.is_null() {
            return None;
        }
        let scaler = WicComPtr(scaler_ptr);
        let s_vtbl = *(scaler.0 as *const *const usize);

        // IWICBitmapScaler::Initialize (slot 8)
        let scaler_init: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *mut std::ffi::c_void,
            u32,
            u32,
            u32,
        ) -> HRESULT = std::mem::transmute(*s_vtbl.add(8));
        if scaler_init(scaler.0, frame.0, target_w, target_h, WIC_INTERP_FANT).is_err() {
            return None;
        }

        // IWICImagingFactory::CreateFormatConverter (slot 10)
        let create_conv: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT = std::mem::transmute(*f_vtbl.add(10));
        let mut conv_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        if create_conv(factory.0, &mut conv_ptr).is_err() || conv_ptr.is_null() {
            return None;
        }
        let conv = WicComPtr(conv_ptr);
        let c_vtbl = *(conv.0 as *const *const usize);

        // IWICFormatConverter::Initialize (slot 8)
        let conv_init: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *mut std::ffi::c_void,
            *const GUID,
            u32,
            *mut std::ffi::c_void,
            f64,
            u32,
        ) -> HRESULT = std::mem::transmute(*c_vtbl.add(8));
        if conv_init(
            conv.0,
            scaler.0,
            &GUID_WIC_32BPP_PBGRA,
            WIC_DITHER_NONE,
            std::ptr::null_mut(),
            0.0,
            WIC_PALETTE_CUSTOM,
        )
        .is_err()
        {
            return None;
        }

        // IWICBitmapSource::CopyPixels (slot 7)
        let copy_pixels: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *const std::ffi::c_void,
            u32,
            u32,
            *mut u8,
        ) -> HRESULT = std::mem::transmute(*c_vtbl.add(7));
        let stride = target_w.checked_mul(4)?;
        let byte_len = (stride as usize).checked_mul(target_h as usize)?;
        let mut bgra = vec![0u8; byte_len];
        if copy_pixels(
            conv.0,
            std::ptr::null(),
            stride,
            byte_len as u32,
            bgra.as_mut_ptr(),
        )
        .is_err()
        {
            return None;
        }

        Some(IconRgba {
            width: target_w,
            height: target_h,
            bgra_premul: bgra,
        })
    }
}

fn find_best_uwp_asset_png(exe_dir: &Path) -> Option<PathBuf> {
    let assets_dir = exe_dir.join("Assets");
    let entries = std::fs::read_dir(&assets_dir).ok()?;
    let mut best: Option<(i32, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        let is_png = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("png"));
        if !is_png {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        let mut score: i32 = if lower.contains("square44x44logo") {
            400
        } else if lower.contains("applist") {
            360
        } else if lower.contains("medtile") || lower.contains("square150x150logo") {
            280
        } else if lower.contains("storelogo") {
            220
        } else if lower.contains("logo") {
            140
        } else {
            continue;
        };

        if lower.contains("unplated") {
            score += 120;
        }
        if lower.contains("targetsize-48") || lower.contains("targetsize-64") {
            score += 90;
        } else if lower.contains("scale-200") {
            score += 80;
        } else if lower.contains("scale-150") {
            score += 60;
        } else if lower.contains("scale-400") {
            score += 50;
        } else if lower.contains("scale-100") {
            score += 30;
        }
        if lower.contains("altform-") && !lower.contains("altform-unplated") {
            score -= 60;
        }
        if lower.contains("contrast-white") || lower.contains("contrast-black") {
            score -= 150;
        }
        if lower.contains("wide") || lower.contains("splash") {
            score -= 300;
        }

        if best.as_ref().is_none_or(|(best_s, _)| score > *best_s) {
            best = Some((score, path));
        }
    }
    best.map(|(_, p)| p)
}

fn is_uwp_package_path(exe_path: &str) -> bool {
    if exe_path.is_empty() {
        return false;
    }
    let lower_path = exe_path.to_ascii_lowercase();
    if lower_path.contains("windowsapps") {
        return true;
    }
    if let Some(exe_dir) = Path::new(exe_path).parent() {
        return exe_dir.join("AppxManifest.xml").exists() || exe_dir.join("Assets").is_dir();
    }
    false
}

fn extract_uwp_package_icon(exe_path: &str, target_size: u32) -> Option<IconRgba> {
    if !is_uwp_package_path(exe_path) {
        return None;
    }
    let exe_dir = Path::new(exe_path).parent()?;
    let png_path = find_best_uwp_asset_png(exe_dir)?;
    let size = target_size.clamp(32, 64);
    decode_image_bgra_premul(&png_path, size, size)
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

/// Apple Liquid Glass material variant for [`LiquidGlassStyle`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiquidGlassVariant {
    /// Adaptive obsidian smoked glass (~77% alpha) for panels, menus, settings, and text-heavy surfaces.
    Regular,
    /// Lighter clear smoked glass (~62% alpha) for floating HUD pills and media overlays.
    Clear,
    /// Elevated inner card / control surface (~82% alpha).
    Elevated,
    /// Stained-glass tint (`Glass.regular.tint(color)`), reserved for primary actions & active tabs.
    Tinted(Color),
}

/// Configuration for [`NativeSurface::draw_liquid_glass_panel`] and [`NativeSurface::draw_liquid_glass_edge_surface`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidGlassStyle {
    pub variant: LiquidGlassVariant,
    pub corner_radius: f32,
    pub shadow_blur: f32,
    pub shadow_offset_y: f32,
    pub shadow_opacity: f32,
    pub hovered: bool,
    pub pressed: bool,
}

impl LiquidGlassStyle {
    /// Regular obsidian Liquid Glass panel with soft outer shadow.
    pub const fn regular(corner_radius: f32) -> Self {
        Self {
            variant: LiquidGlassVariant::Regular,
            corner_radius,
            shadow_blur: 10.0,
            shadow_offset_y: 3.0,
            shadow_opacity: 0.28,
            hovered: false,
            pressed: false,
        }
    }

    /// Clear translucent Liquid Glass pill with light shadow.
    pub const fn clear(corner_radius: f32) -> Self {
        Self {
            variant: LiquidGlassVariant::Clear,
            corner_radius,
            shadow_blur: 8.0,
            shadow_offset_y: 2.0,
            shadow_opacity: 0.20,
            hovered: false,
            pressed: false,
        }
    }

    /// Elevated inner card surface (no heavy outer shadow).
    pub const fn elevated(corner_radius: f32) -> Self {
        Self {
            variant: LiquidGlassVariant::Elevated,
            corner_radius,
            shadow_blur: 0.0,
            shadow_offset_y: 0.0,
            shadow_opacity: 0.0,
            hovered: false,
            pressed: false,
        }
    }

    /// Tinted Liquid Glass pill reserved for primary actions or selected states.
    pub const fn tinted(color: Color, corner_radius: f32) -> Self {
        Self {
            variant: LiquidGlassVariant::Tinted(color),
            corner_radius,
            shadow_blur: 6.0,
            shadow_offset_y: 2.0,
            shadow_opacity: 0.18,
            hovered: false,
            pressed: false,
        }
    }

    /// Compact interactive Liquid Glass button preset.
    pub const fn button(corner_radius: f32, hovered: bool, pressed: bool) -> Self {
        Self {
            variant: LiquidGlassVariant::Elevated,
            corner_radius,
            shadow_blur: 4.0,
            shadow_offset_y: 1.0,
            shadow_opacity: 0.14,
            hovered,
            pressed,
        }
    }

    /// Overrides the outer drop shadow parameters.
    pub const fn with_shadow(mut self, blur: f32, offset_y: f32, opacity: f32) -> Self {
        self.shadow_blur = blur;
        self.shadow_offset_y = offset_y;
        self.shadow_opacity = opacity;
        self
    }

    /// Disables the outer drop shadow (e.g. for inner cards or tight clipped surfaces).
    pub const fn without_shadow(mut self) -> Self {
        self.shadow_blur = 0.0;
        self.shadow_offset_y = 0.0;
        self.shadow_opacity = 0.0;
        self
    }
}

/// Crisp SF-Symbols-style vector glyphs rendered with uniform rounded Direct2D strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfSymbol {
    CloseX,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    SettingsSliders,
    Desktops,
    TrayGrid,
    VolumeHigh,
    VolumeMuted,
    BrightnessSun,
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Plus,
    Minus,
    Trash,
    Check,
    Wifi,
    WifiOff,
    FolderDrop,
    Search,
    Sparkle,
    Pin,
    Calendar,
    FocusTimer,
    ShortcutKeys,
    Gear,
    MusicNote,
    Bell,
    Bluetooth,
    Copy,
    SendArrow,
    ShieldCheck,
    ListChecklist,
    Terminal,
    Document,
    Globe,
    Refresh,
    StopCircle,
    PrivateMask,
}

/// Cached 32-bit premultiplied BGRA DIB section backing an `UpdateLayeredWindow` surface.
struct BackingDib {
    hdc: HDC,
    hbitmap: HBITMAP,
    old_obj: HGDIOBJ,
    #[cfg(test)]
    bits: *mut u8,
    #[cfg(test)]
    width: u32,
    #[cfg(test)]
    height: u32,
}

impl BackingDib {
    unsafe fn new(width: u32, height: u32) -> Result<Self> {
        let width = width.max(1);
        let height = height.max(1);
        let hdc = CreateCompatibleDC(None);
        if hdc.0.is_null() {
            return Err(windows::core::Error::from_win32());
        }

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbitmap = match CreateDIBSection(Some(hdc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(hbmp) => hbmp,
            Err(err) => {
                let _ = DeleteDC(hdc);
                return Err(err);
            }
        };
        let old_obj = SelectObject(hdc, hbitmap.into());
        let byte_len = (width as usize) * (height as usize) * 4;
        if !bits.is_null() {
            std::ptr::write_bytes(bits as *mut u8, 0, byte_len);
        }

        Ok(Self {
            hdc,
            hbitmap,
            old_obj,
            #[cfg(test)]
            bits: bits as *mut u8,
            #[cfg(test)]
            width,
            #[cfg(test)]
            height,
        })
    }

    #[cfg(test)]
    unsafe fn pixel_bgra(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height || self.bits.is_null() {
            return None;
        }
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        let ptr = self.bits.add(idx);
        Some([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)])
    }
}

impl Drop for BackingDib {
    fn drop(&mut self) {
        unsafe {
            if !self.hdc.0.is_null() {
                if !self.old_obj.0.is_null() {
                    let _ = SelectObject(self.hdc, self.old_obj);
                }
                if !self.hbitmap.0.is_null() {
                    let _ = DeleteObject(self.hbitmap.into());
                }
                let _ = DeleteDC(self.hdc);
            }
        }
    }
}

/// Per-surface desktop/window backdrop snapshot for real Liquid Glass Gaussian blur (`frost`),
/// bevel refraction, and automatic Light/Dark Glass contrast adaptation.
#[derive(Debug, Clone)]
pub struct BackdropBuffer {
    pub width: u32,
    pub height: u32,
    pub bgra_premul: Vec<u8>,
    pub avg_luminance: f32,
}

impl BackdropBuffer {
    /// Builds a [`BackdropBuffer`] from 32-bit premultiplied BGRA pixels and computes its
    /// average perceptual WCAG luminance (`0.0..=1.0`).
    pub fn from_bgra_premul(width: u32, height: u32, bgra_premul: Vec<u8>) -> Option<Self> {
        let byte_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))?;
        if width == 0 || height == 0 || bgra_premul.len() < byte_len {
            return None;
        }
        let avg_luminance = compute_avg_luminance(width, height, &bgra_premul[..byte_len]);
        Some(Self {
            width,
            height,
            bgra_premul: bgra_premul[..byte_len].to_vec(),
            avg_luminance,
        })
    }
}

struct BackdropState {
    raw: BackdropBuffer,
    blurred_w: u32,
    blurred_h: u32,
    blurred_bgra: Vec<u8>,
    generation: u64,
}

impl BackdropState {
    fn new(raw: BackdropBuffer, generation: u64) -> Self {
        let (blurred_w, blurred_h, blurred_bgra) =
            blur_bgra_half_res(raw.width, raw.height, &raw.bgra_premul);
        Self {
            raw,
            blurred_w,
            blurred_h,
            blurred_bgra,
            generation,
        }
    }
}

const MAX_GLASS_BITMAP_CACHE: usize = 24;

#[derive(Default)]
struct GlassBitmapCache {
    entries: HashMap<u64, ID2D1Bitmap>,
    order: VecDeque<u64>,
}

impl GlassBitmapCache {
    fn get(&self, key: u64) -> Option<ID2D1Bitmap> {
        self.entries.get(&key).cloned()
    }

    fn insert(&mut self, key: u64, bmp: ID2D1Bitmap) {
        if self.entries.contains_key(&key) {
            return;
        }
        while self.order.len() >= MAX_GLASS_BITMAP_CACHE {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
            }
        }
        self.order.push_back(key);
        self.entries.insert(key, bmp);
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
    }
}

/// Borderless topmost `WS_EX_LAYERED` window backed by Direct2D `ID2D1DCRenderTarget`,
/// a cached 32-bit premultiplied BGRA DIB section, and `UpdateLayeredWindow(ULW_ALPHA)`.
pub struct NativeSurface {
    hwnd: HWND,
    width: u32,
    height: u32,
    dpi: f32,
    d2d_factory: ID2D1Factory1,
    dwrite_factory: IDWriteFactory,
    d2d_target: ID2D1DCRenderTarget,
    dib: RefCell<BackingDib>,
    brush_cache: RefCell<HashMap<u32, ID2D1SolidColorBrush>>,
    text_format_cache: RefCell<HashMap<(u32, u8, bool), IDWriteTextFormat>>,
    bitmap_cache: RefCell<BitmapCache>,
    glass_bitmap_cache: RefCell<GlassBitmapCache>,
    backdrop: RefCell<Option<BackdropState>>,
    backdrop_gen: Cell<u64>,
    color_theme_mode: Cell<ColorThemeMode>,
    cached_layer: RefCell<Option<ID2D1Layer>>,
    round_cap_stroke: RefCell<Option<ID2D1StrokeStyle>>,
    last_region_key: RefCell<Option<RegionKey>>,
    region_is_empty: RefCell<bool>,
}

impl NativeSurface {
    /// Creates a new borderless topmost layered Direct2D surface window with true per-pixel alpha.
    pub fn new(title: &str, x: i32, y: i32, width: u32, height: u32) -> Result<Self> {
        let width = width.max(1);
        let height = height.max(1);

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let hinstance: HINSTANCE = GetModuleHandleW(None)?.into();
            register_window_class(hinstance);

            let wide_title = to_wide_null(title);
            let ex_style = WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED;
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
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: dpi,
                dpiY: dpi,
                ..Default::default()
            };
            let d2d_target = d2d_factory.CreateDCRenderTarget(&rt_props)?;
            let dib = BackingDib::new(width, height)?;

            Ok(Self {
                hwnd,
                width,
                height,
                dpi,
                d2d_factory,
                dwrite_factory,
                d2d_target,
                dib: RefCell::new(dib),
                brush_cache: RefCell::new(HashMap::new()),
                text_format_cache: RefCell::new(HashMap::new()),
                bitmap_cache: RefCell::new(BitmapCache::default()),
                glass_bitmap_cache: RefCell::new(GlassBitmapCache::default()),
                backdrop: RefCell::new(None),
                backdrop_gen: Cell::new(0),
                color_theme_mode: Cell::new(ColorThemeMode::Auto),
                cached_layer: RefCell::new(None),
                round_cap_stroke: RefCell::new(None),
                last_region_key: RefCell::new(None),
                region_is_empty: RefCell::new(false),
            })
        }
    }

    /// Sets the user color theme preference (`Auto`, `Dark`, or `Light`).
    pub fn set_color_theme_mode(&self, mode: ColorThemeMode) {
        if self.color_theme_mode.get() != mode {
            self.color_theme_mode.set(mode);
            self.glass_bitmap_cache.borrow_mut().clear();
        }
    }

    /// Returns the active [`ColorThemeMode`].
    pub fn color_theme_mode(&self) -> ColorThemeMode {
        self.color_theme_mode.get()
    }

    /// Supplies a captured 32-bit premultiplied BGRA backdrop buffer behind this surface for
    /// real Gaussian `frost` blur, bevel refraction, and adaptive Light/Dark Glass contrast.
    pub fn set_backdrop_pixels(&self, width: u32, height: u32, bgra_premul: &[u8]) {
        let Some(buf) = BackdropBuffer::from_bgra_premul(width, height, bgra_premul.to_vec()) else {
            return;
        };
        self.set_backdrop(Some(buf));
    }

    /// Replaces or clears the surface's [`BackdropBuffer`].
    pub fn set_backdrop(&self, backdrop: Option<BackdropBuffer>) {
        let next_gen = self.backdrop_gen.get().wrapping_add(1);
        self.backdrop_gen.set(next_gen);
        self.glass_bitmap_cache.borrow_mut().clear();
        *self.backdrop.borrow_mut() = backdrop.map(|raw| BackdropState::new(raw, next_gen));
    }

    /// Clears any cached backdrop buffer (reverts to headless/flat adaptive glass fallback).
    pub fn clear_backdrop(&self) {
        self.set_backdrop(None);
    }

    /// Returns the average perceptual luminance (`0.0..=1.0`) of the current backdrop, if set.
    pub fn backdrop_luminance(&self) -> Option<f32> {
        self.backdrop.borrow().as_ref().map(|b| b.raw.avg_luminance)
    }

    /// Resolves the effective [`ThemePolarity`] (`LightGlass` vs `DarkGlass`) from
    /// [`ColorThemeMode`] and the current backdrop luminance.
    pub fn theme_polarity(&self) -> ThemePolarity {
        self.palette().polarity
    }

    /// Returns the resolved [`AdaptivePalette`] for this surface's current polarity.
    pub fn palette(&self) -> AdaptivePalette {
        AdaptivePalette::resolve(self.color_theme_mode.get(), self.backdrop_luminance())
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

    pub fn render_target(&self) -> &ID2D1DCRenderTarget {
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

    /// Moves and resizes the surface window and its cached 32-bit DIB backing store.
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

    /// Resizes only the window position without reallocating the backing DIB.
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

    /// Updates hit-test/empty-hide state cleanly while letting `ULW_ALPHA` handle
    /// anti-aliased edges and soft outer drop shadows with zero black border artifacts.
    pub fn set_rounded_region(&self, rect: Rect, radius: f32) {
        let scale = (self.dpi / 96.0).max(0.5);
        let is_empty = rect.width <= 0.0 || rect.height <= 0.0;
        let (left, top, right, bottom, diam) = if is_empty {
            (0, 0, 0, 0, 0)
        } else {
            (
                (rect.x * scale).round() as i32,
                (rect.y * scale).round() as i32,
                ((rect.x + rect.width) * scale).round() as i32 + 1,
                ((rect.y + rect.height) * scale).round() as i32 + 1,
                ((radius * 2.0) * scale).round().max(0.0) as i32,
            )
        };
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
        self.apply_layered_region_state(is_empty, key);
    }

    /// Preserves region-cache API compatibility for multi-rect surfaces while avoiding GDI
    /// `SetWindowRgn` clipping so `ULW_ALPHA` renders clean soft shadows and magnified icons.
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
        let mut all_empty = true;
        for &(r, rad) in rects {
            if r.width > 0.0 && r.height > 0.0 {
                all_empty = false;
            }
            let left = (r.x * scale).round() as i32;
            let top = (r.y * scale).round() as i32;
            let right = ((r.x + r.width) * scale).round() as i32 + 1;
            let bottom = ((r.y + r.height) * scale).round() as i32 + 1;
            let diam = ((rad * 2.0) * scale).round().max(0.0) as i32;
            for v in [left, top, right, bottom, diam] {
                hash ^= v as u64;
                hash = hash.wrapping_mul(0x1000_0000_01b3);
            }
        }

        let key = RegionKey::Multi(hash);
        if *self.last_region_key.borrow() == Some(key) {
            return;
        }
        self.apply_layered_region_state(all_empty, key);
    }

    /// Binds the cached 32-bit DIB section, begins a Direct2D frame, and clears to transparent alpha.
    pub fn begin_draw(&self) {
        unsafe {
            let hdc = self.dib.borrow().hdc;
            let rect = RECT {
                left: 0,
                top: 0,
                right: self.width as i32,
                bottom: self.height as i32,
            };
            let _ = self.d2d_target.BindDC(hdc, &rect);
            self.d2d_target.BeginDraw();
            self.d2d_target.Clear(Some(&TRANSPARENT.to_d2d()));
        }
    }

    /// Ends the Direct2D frame and composites the premultiplied 32-bit DIB via `UpdateLayeredWindow`.
    pub fn end_draw_and_present(&self) -> Result<()> {
        unsafe {
            self.d2d_target.EndDraw(None, None)?;
            let hdc = self.dib.borrow().hdc;
            let size = SIZE {
                cx: self.width as i32,
                cy: self.height as i32,
            };
            let pt_src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            UpdateLayeredWindow(
                self.hwnd,
                None,
                None,
                Some(&size),
                Some(hdc),
                Some(&pt_src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )?;
            Ok(())
        }
    }

    fn apply_layered_region_state(&self, is_empty: bool, key: RegionKey) {
        unsafe {
            if is_empty {
                if !*self.region_is_empty.borrow() {
                    let empty_rgn = CreateRectRgn(0, 0, 0, 0);
                    if !empty_rgn.is_invalid() {
                        if SetWindowRgn(self.hwnd, Some(empty_rgn), true) != 0 {
                            *self.region_is_empty.borrow_mut() = true;
                        } else {
                            let _ = DeleteObject(empty_rgn.into());
                        }
                    }
                }
            } else if *self.region_is_empty.borrow() {
                let _ = SetWindowRgn(self.hwnd, None, false);
                *self.region_is_empty.borrow_mut() = false;
            }
        }
        *self.last_region_key.borrow_mut() = Some(key);
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
            HimeExpression::Netral | HimeExpression::Fokus => (SYSTEM_BLUE, SYSTEM_BLUE_TINT),
            HimeExpression::Senang => (STATUS_LIVE_GREEN, STATUS_LIVE_GREEN.with_alpha(0.25)),
            HimeExpression::Khawatir => (STATUS_LIVE_AMBER, STATUS_LIVE_AMBER.with_alpha(0.25)),
            HimeExpression::Offline => (STATUS_MUTED, STATUS_MUTED.with_alpha(0.15)),
            HimeExpression::Waspada => (STATUS_LIVE_RED, STATUS_LIVE_RED.with_alpha(0.32)),
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

    /// Draws an Apple Liquid Glass `EdgeSurface` contour (concave ears + real blurred/refracted
    /// backdrop with edge bevel dispersion + crisp 1px optical rim, or clean flat adaptive fallback).
    pub fn draw_liquid_glass_edge_surface(
        &self,
        rect: Rect,
        ear_w: f32,
        radius: f32,
        edge: EdgeSide,
        flares: bool,
        style: LiquidGlassStyle,
    ) -> Result<()> {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return Ok(());
        }
        unsafe {
            if style.shadow_opacity > 0.0 && style.shadow_blur > 0.0 {
                self.draw_edge_contour_shadow(
                    rect,
                    ear_w,
                    radius,
                    edge,
                    flares,
                    style.shadow_blur,
                    style.shadow_offset_y,
                    style.shadow_opacity,
                )?;
            }

            if self.try_draw_refracted_glass(
                rect,
                style,
                Some((ear_w, radius, edge, flares)),
            )? {
                return Ok(());
            }

            // Headless / no-backdrop fallback: clean flat adaptive glass + 1px optical rim (no diagonal gradient).
            let palette = self.palette();
            let (base_fill, border_col) = resolve_fallback_glass_colors(&palette, style);
            let ops = edge_contour_points(rect, ear_w, radius, edge, flares);
            let geo = self.build_contour_geometry(&ops)?;
            let fill_brush = self.create_brush(base_fill)?;
            self.d2d_target.FillGeometry(&geo, &fill_brush, None);
            let border_brush = self.create_brush(border_col)?;
            self.d2d_target.DrawGeometry(&geo, &border_brush, 1.0, None);
            Ok(())
        }
    }

    /// Draws an edge-aware soft outer drop shadow following the exact `edge_contour_points`
    /// silhouette, keeping the screen-bezel edge flush so no rounded corners protrude beyond
    /// flared ears on any `EdgeSide`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_edge_contour_shadow(
        &self,
        rect: Rect,
        ear_w: f32,
        radius: f32,
        edge: EdgeSide,
        flares: bool,
        blur: f32,
        offset_y: f32,
        opacity: f32,
    ) -> Result<()> {
        if rect.width <= 0.0 || rect.height <= 0.0 || blur <= 0.0 || opacity <= 0.0 {
            return Ok(());
        }
        if edge == EdgeSide::Bottom && !flares {
            return self.draw_soft_shadow(rect, radius, blur, offset_y, opacity);
        }
        let steps = 5_usize;
        let max_blur = blur.min(24.0);
        let base_alpha = opacity.clamp(0.0, 0.65);
        unsafe {
            for i in 0..steps {
                let frac = 1.0 - (i as f32 / steps as f32);
                let expand = max_blur * frac;
                let falloff = (-(frac * 1.55) * (frac * 1.55)).exp();
                let step_alpha = (base_alpha * falloff / (steps as f32 * 0.58)).clamp(0.004, 0.28);
                let brush = self.create_brush(Color::rgba(0.0, 0.0, 0.0, step_alpha))?;
                let sr = match edge {
                    EdgeSide::Top => Rect::new(
                        rect.x - expand * 0.75,
                        rect.y,
                        rect.width + expand * 1.5,
                        rect.height + expand + offset_y.max(0.0) * 0.55,
                    ),
                    EdgeSide::Bottom => {
                        let bottom = rect.y + rect.height;
                        let up = expand + offset_y.abs() * 0.25;
                        Rect::new(
                            rect.x - expand * 0.75,
                            rect.y - up,
                            rect.width + expand * 1.5,
                            bottom - (rect.y - up),
                        )
                    }
                    EdgeSide::Left => Rect::new(
                        rect.x,
                        rect.y - expand * 0.75 + offset_y * 0.25,
                        rect.width + expand,
                        rect.height + expand * 1.5,
                    ),
                    EdgeSide::Right => Rect::new(
                        rect.x - expand,
                        rect.y - expand * 0.75 + offset_y * 0.25,
                        rect.width + expand,
                        rect.height + expand * 1.5,
                    ),
                };
                let ops = edge_contour_points(sr, ear_w, (radius + expand * 0.65).max(0.0), edge, flares);
                let geo = self.build_contour_geometry(&ops)?;
                self.d2d_target.FillGeometry(&geo, &brush, None);
            }
            Ok(())
        }
    }

    /// Draws an anti-aliased soft outer drop shadow behind `rect` using concentric
    /// premultiplied alpha rings (clean on `WS_EX_LAYERED` + `ULW_ALPHA` with zero black frames).
    pub fn draw_soft_shadow(
        &self,
        rect: Rect,
        radius: f32,
        blur: f32,
        offset_y: f32,
        opacity: f32,
    ) -> Result<()> {
        if rect.width <= 0.0 || rect.height <= 0.0 || blur <= 0.0 || opacity <= 0.0 {
            return Ok(());
        }
        let steps = 5_usize;
        let max_blur = blur.min(24.0);
        let base_alpha = opacity.clamp(0.0, 0.65);
        unsafe {
            for i in 0..steps {
                let frac = 1.0 - (i as f32 / steps as f32);
                let expand = max_blur * frac;
                let falloff = (-(frac * 1.55) * (frac * 1.55)).exp();
                let step_alpha = (base_alpha * falloff / (steps as f32 * 0.58)).clamp(0.004, 0.28);
                let brush = self.create_brush(Color::rgba(0.0, 0.0, 0.0, step_alpha))?;
                let sr = Rect::new(
                    rect.x - expand,
                    rect.y + offset_y - expand * 0.65,
                    rect.width + expand * 2.0,
                    rect.height + expand * 2.0,
                );
                let r = (radius + expand * 0.65).max(0.0);
                let rr = D2D1_ROUNDED_RECT {
                    rect: sr.to_d2d(),
                    radiusX: r,
                    radiusY: r,
                };
                self.d2d_target.FillRoundedRectangle(&rr, &brush);
            }
            Ok(())
        }
    }

    /// Draws an Apple Liquid Glass rounded panel (`Regular`, `Clear`, `Elevated`, or `Tinted`):
    /// soft outer drop shadow -> real blurred+refracted backdrop with subtle RGB edge dispersion
    /// and 1px optical rim (or clean flat adaptive frosted fill when no backdrop is present).
    pub fn draw_liquid_glass_panel(&self, rect: Rect, style: LiquidGlassStyle) -> Result<()> {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return Ok(());
        }
        let radius = style
            .corner_radius
            .max(0.0)
            .min(rect.width * 0.5)
            .min(rect.height * 0.5);

        unsafe {
            if style.shadow_opacity > 0.0 && style.shadow_blur > 0.0 {
                self.draw_soft_shadow(
                    rect,
                    radius,
                    style.shadow_blur,
                    style.shadow_offset_y,
                    style.shadow_opacity,
                )?;
            }

            if self.try_draw_refracted_glass(rect, style, None)? {
                return Ok(());
            }

            // Headless / no-backdrop fallback: clean flat adaptive translucency + 1px optical rim.
            let palette = self.palette();
            let (base_fill, border_col) = resolve_fallback_glass_colors(&palette, style);
            let rr = D2D1_ROUNDED_RECT {
                rect: rect.to_d2d(),
                radiusX: radius,
                radiusY: radius,
            };
            let fill_brush = self.create_brush(base_fill)?;
            self.d2d_target.FillRoundedRectangle(&rr, &fill_brush);

            // Subtle 1px optical inner highlight along inner edge
            if rect.width > 6.0 && rect.height > 6.0 {
                let inner_rr = D2D1_ROUNDED_RECT {
                    rect: Rect::new(
                        rect.x + 1.25,
                        rect.y + 1.25,
                        rect.width - 2.5,
                        rect.height - 2.5,
                    )
                    .to_d2d(),
                    radiusX: (radius - 1.25).max(0.0),
                    radiusY: (radius - 1.25).max(0.0),
                };
                let rim_alpha = if palette.polarity == ThemePolarity::LightGlass {
                    0.50
                } else {
                    0.12
                };
                let inner_brush = self.create_brush(Color::rgba(1.0, 1.0, 1.0, rim_alpha))?;
                self.d2d_target
                    .DrawRoundedRectangle(&inner_rr, &inner_brush, 0.85, None);
            }

            // Crisp 1px outer hairline border
            let border_rr = D2D1_ROUNDED_RECT {
                rect: Rect::new(
                    rect.x + 0.5,
                    rect.y + 0.5,
                    (rect.width - 1.0).max(0.5),
                    (rect.height - 1.0).max(0.5),
                )
                .to_d2d(),
                radiusX: (radius - 0.5).max(0.0),
                radiusY: (radius - 0.5).max(0.0),
            };
            let border_brush = self.create_brush(border_col)?;
            self.d2d_target
                .DrawRoundedRectangle(&border_rr, &border_brush, 1.0, None);
            Ok(())
        }
    }

    /// Draws a compact Liquid Glass button/pill with adaptive hover/press response and optional tint.
    pub fn draw_liquid_glass_button(
        &self,
        rect: Rect,
        corner_radius: f32,
        tint: Option<Color>,
        hovered: bool,
        pressed: bool,
    ) -> Result<()> {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return Ok(());
        }
        let palette = self.palette();
        let radius = corner_radius
            .max(0.0)
            .min(rect.width * 0.5)
            .min(rect.height * 0.5);
        let fill = match tint {
            Some(c) => {
                let lift = if pressed { 0.14 } else if hovered { 0.08 } else { 0.0 };
                c.with_alpha((c.a.clamp(0.24, 0.88) + lift).clamp(0.24, 0.94))
            }
            None => {
                if pressed {
                    palette.control_active
                } else if hovered {
                    palette.control_hover
                } else {
                    palette.control_fill
                }
            }
        };
        let border = match tint {
            Some(c) => c.lerp(Color::from_rgb_u8(0xFF, 0xFF, 0xFF), 0.40).with_alpha(0.45),
            None => palette.hairline_border,
        };
        self.fill_rounded_rect(rect, radius, fill)?;
        self.stroke_rounded_rect(
            Rect::new(
                rect.x + 0.5,
                rect.y + 0.5,
                (rect.width - 1.0).max(0.5),
                (rect.height - 1.0).max(0.5),
            ),
            (radius - 0.5).max(0.0),
            border,
            1.0,
        )?;
        Ok(())
    }

    /// Draws a frameless icon button container: draws nothing (0 fill, 0 border) at rest,
    /// and a soft borderless translucent pill wash on hover, press, or active selection.
    pub fn draw_icon_button(
        &self,
        rect: Rect,
        corner_radius: f32,
        hovered: bool,
        pressed: bool,
        active: bool,
    ) -> Result<()> {
        if (!hovered && !pressed && !active) || rect.width <= 0.0 || rect.height <= 0.0 {
            return Ok(());
        }
        let palette = self.palette();
        let radius = corner_radius
            .max(0.0)
            .min(rect.width * 0.5)
            .min(rect.height * 0.5);
        let fill = if pressed || active {
            palette.control_active
        } else {
            palette.control_hover
        };
        self.fill_rounded_rect(rect, radius, fill)
    }

    unsafe fn try_draw_refracted_glass(
        &self,
        rect: Rect,
        style: LiquidGlassStyle,
        edge_params: Option<(f32, f32, EdgeSide, bool)>,
    ) -> Result<bool> {
        let backdrop_ref = self.backdrop.borrow();
        let Some(bd) = backdrop_ref.as_ref() else {
            return Ok(false);
        };

        let scale = (self.dpi / 96.0).max(0.5);
        let px_x = (rect.x * scale).floor();
        let px_y = (rect.y * scale).floor();
        let px_w = (rect.width * scale).ceil().max(1.0) as u32;
        let px_h = (rect.height * scale).ceil().max(1.0) as u32;
        if px_w == 0 || px_h == 0 {
            return Ok(true);
        }

        // Rasterize interior blur & bevel refraction at half-resolution (matching `blurred_bgra`)
        // for <0.3ms 60fps hover transitions, then seal with a crisp 1px D2D vector outline.
        let gw = (px_w / 2).max(1);
        let gh = (px_h / 2).max(1);

        let palette = AdaptivePalette::resolve(
            self.color_theme_mode.get(),
            Some(bd.raw.avg_luminance),
        );
        let polarity = palette.polarity;
        let cache_key = glass_cache_key(
            bd.generation,
            polarity,
            px_x as i32,
            px_y as i32,
            gw,
            gh,
            style,
            edge_params,
        );

        let cached_bmp = self.glass_bitmap_cache.borrow().get(cache_key);
        let bitmap = if let Some(cached) = cached_bmp {
            cached
        } else {
            let pixels = rasterize_liquid_glass(
                bd,
                self.width,
                self.height,
                px_x,
                px_y,
                px_w,
                px_h,
                gw,
                gh,
                scale,
                style,
                polarity,
                edge_params,
            );
            let props = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: self.dpi,
                dpiY: self.dpi,
            };
            let created = self.d2d_target.CreateBitmap(
                D2D_SIZE_U {
                    width: gw,
                    height: gh,
                },
                Some(pixels.as_ptr() as *const _),
                gw * 4,
                &props,
            )?;
            self.glass_bitmap_cache
                .borrow_mut()
                .insert(cache_key, created.clone());
            created
        };

        drop(backdrop_ref);

        let dest = rect.to_d2d();
        let r = style
            .corner_radius
            .max(0.0)
            .min(rect.width * 0.5)
            .min(rect.height * 0.5);

        if edge_params.is_none() && r > 0.0 {
            let clip_rr = D2D1_ROUNDED_RECT {
                rect: dest,
                radiusX: r,
                radiusY: r,
            };
            let clip_geo = self.d2d_factory.CreateRoundedRectangleGeometry(&clip_rr)?;
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

        // Crisp 1px D2D vector optical rim/outline at native DPI
        let border_col = match style.variant {
            LiquidGlassVariant::Tinted(tint) => tint
                .lerp(Color::from_rgb_u8(0xFF, 0xFF, 0xFF), 0.45)
                .with_alpha(0.45),
            _ => palette.hairline_border,
        };
        let border_brush = self.create_brush(border_col)?;
        if let Some((ear_w, er, edge, flares)) = edge_params {
            let ops = edge_contour_points(rect, ear_w, er, edge, flares);
            let geo = self.build_contour_geometry(&ops)?;
            self.d2d_target.DrawGeometry(&geo, &border_brush, 1.0, None);
        } else {
            let border_rr = D2D1_ROUNDED_RECT {
                rect: Rect::new(
                    rect.x + 0.5,
                    rect.y + 0.5,
                    (rect.width - 1.0).max(0.5),
                    (rect.height - 1.0).max(0.5),
                )
                .to_d2d(),
                radiusX: (r - 0.5).max(0.0),
                radiusY: (r - 0.5).max(0.0),
            };
            self.d2d_target
                .DrawRoundedRectangle(&border_rr, &border_brush, 1.0, None);
        }
        Ok(true)
    }

    /// Updates hit-test/empty-hide state for `EdgeSurface` windows cleanly while letting
    /// `ULW_ALPHA` preserve anti-aliased concave ears and soft drop shadows.
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
        let is_empty = rect.width <= 0.0 || rect.height <= 0.0;
        self.apply_layered_region_state(is_empty, key);
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
            STATUS_LIVE_GREEN
        } else if percent <= 20 {
            STATUS_LIVE_AMBER
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
        let color = if is_playing { SYSTEM_BLUE } else { STATUS_MUTED };
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

    /// Draws an SF-Symbols-style vector glyph inside `rect` with uniform rounded strokes.
    /// Pass `stroke_width <= 0.0` to compute optical stroke weight automatically from `rect`.
    pub fn draw_sf_symbol(
        &self,
        rect: Rect,
        symbol: SfSymbol,
        color: Color,
        stroke_width: f32,
    ) -> Result<()> {
        let s = rect.width.min(rect.height).max(8.0);
        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;
        let sw = if stroke_width > 0.0 {
            stroke_width.max(0.8)
        } else {
            (s * 0.11).clamp(1.2, 2.4)
        };

        match symbol {
            SfSymbol::CloseX => {
                let u = s * 0.25;
                self.draw_line(cx - u, cy - u, cx + u, cy + u, color, sw)?;
                self.draw_line(cx + u, cy - u, cx - u, cy + u, color, sw)?;
            }
            SfSymbol::ChevronLeft => {
                self.draw_polyline(
                    &[
                        (cx + s * 0.12, cy - s * 0.26),
                        (cx - s * 0.14, cy),
                        (cx + s * 0.12, cy + s * 0.26),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::ChevronRight => {
                self.draw_polyline(
                    &[
                        (cx - s * 0.12, cy - s * 0.26),
                        (cx + s * 0.14, cy),
                        (cx - s * 0.12, cy + s * 0.26),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::ChevronDown => {
                self.draw_polyline(
                    &[
                        (cx - s * 0.26, cy - s * 0.10),
                        (cx, cy + s * 0.16),
                        (cx + s * 0.26, cy - s * 0.10),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::ChevronUp => {
                self.draw_polyline(
                    &[
                        (cx - s * 0.26, cy + s * 0.10),
                        (cx, cy - s * 0.16),
                        (cx + s * 0.26, cy + s * 0.10),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::SettingsSliders => {
                let rows = [(-0.24_f32, -0.14_f32), (0.0, 0.14), (0.24, -0.06)];
                for (dy, kx) in rows {
                    let y = cy + s * dy;
                    let knob_x = cx + s * kx;
                    self.draw_line(cx - s * 0.34, y, knob_x - s * 0.08, y, color, sw)?;
                    self.draw_line(knob_x + s * 0.08, y, cx + s * 0.34, y, color, sw)?;
                    self.stroke_circle(knob_x, y, s * 0.075, color, sw)?;
                }
            }
            SfSymbol::Desktops => {
                // Overlapping desktop windows (`macwindow.on.rectangle`)
                self.stroke_rounded_rect(
                    Rect::new(cx - s * 0.12, cy - s * 0.28, s * 0.48, s * 0.38),
                    (s * 0.07).max(1.2),
                    color.with_alpha_mul(0.75),
                    sw,
                )?;
                self.stroke_rounded_rect(
                    Rect::new(cx - s * 0.36, cy - s * 0.08, s * 0.50, s * 0.40),
                    (s * 0.07).max(1.2),
                    color,
                    sw,
                )?;
                self.draw_line(
                    cx - s * 0.24,
                    cy + s * 0.04,
                    cx + s * 0.02,
                    cy + s * 0.04,
                    color,
                    sw * 0.85,
                )?;
            }
            SfSymbol::TrayGrid => {
                let cell = s * 0.25;
                let off = s * 0.165;
                let r = (s * 0.06).max(1.2);
                for dx in [-off, off] {
                    for dy in [-off, off] {
                        self.stroke_rounded_rect(
                            Rect::new(cx + dx - cell * 0.5, cy + dy - cell * 0.5, cell, cell),
                            r,
                            color,
                            sw,
                        )?;
                    }
                }
            }
            SfSymbol::VolumeHigh => {
                self.draw_volume_icon(cx, cy, s, false, 0.85, color)?;
            }
            SfSymbol::VolumeMuted => {
                self.draw_volume_icon(cx, cy, s, true, 0.0, color)?;
            }
            SfSymbol::BrightnessSun => {
                self.draw_sun_icon(cx, cy, s, color)?;
            }
            SfSymbol::Play => {
                self.draw_play_pause_icon(cx, cy, s, false, color)?;
            }
            SfSymbol::Pause => {
                self.draw_play_pause_icon(cx, cy, s, true, color)?;
            }
            SfSymbol::SkipBack => {
                self.draw_skip_icon(cx, cy, s, false, color)?;
            }
            SfSymbol::SkipForward => {
                self.draw_skip_icon(cx, cy, s, true, color)?;
            }
            SfSymbol::Plus => {
                let u = s * 0.28;
                self.draw_line(cx - u, cy, cx + u, cy, color, sw)?;
                self.draw_line(cx, cy - u, cx, cy + u, color, sw)?;
            }
            SfSymbol::Minus => {
                let u = s * 0.28;
                self.draw_line(cx - u, cy, cx + u, cy, color, sw)?;
            }
            SfSymbol::Trash => {
                // Lid bar + top handle
                self.draw_line(cx - s * 0.30, cy - s * 0.22, cx + s * 0.30, cy - s * 0.22, color, sw)?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.12, cy - s * 0.22),
                        (cx - s * 0.08, cy - s * 0.34),
                        (cx + s * 0.08, cy - s * 0.34),
                        (cx + s * 0.12, cy - s * 0.22),
                    ],
                    false,
                    color,
                    sw,
                )?;
                // Bin body
                self.draw_polyline(
                    &[
                        (cx - s * 0.22, cy - s * 0.20),
                        (cx - s * 0.17, cy + s * 0.32),
                        (cx + s * 0.17, cy + s * 0.32),
                        (cx + s * 0.22, cy - s * 0.20),
                    ],
                    false,
                    color,
                    sw,
                )?;
                self.draw_line(cx - s * 0.07, cy - s * 0.08, cx - s * 0.06, cy + s * 0.20, color, sw * 0.85)?;
                self.draw_line(cx + s * 0.07, cy - s * 0.08, cx + s * 0.06, cy + s * 0.20, color, sw * 0.85)?;
            }
            SfSymbol::Check => {
                self.draw_polyline(
                    &[
                        (cx - s * 0.28, cy + s * 0.02),
                        (cx - s * 0.06, cy + s * 0.24),
                        (cx + s * 0.30, cy - s * 0.22),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::Wifi => {
                self.draw_wifi_icon(cx, cy, s, true, false, color)?;
            }
            SfSymbol::WifiOff => {
                self.draw_wifi_icon(cx, cy, s, false, true, color)?;
            }
            SfSymbol::FolderDrop => {
                // Folder silhouette + downward drop arrow
                self.draw_polyline(
                    &[
                        (cx - s * 0.36, cy - s * 0.18),
                        (cx - s * 0.36, cy - s * 0.30),
                        (cx - s * 0.10, cy - s * 0.30),
                        (cx - s * 0.02, cy - s * 0.20),
                        (cx + s * 0.36, cy - s * 0.20),
                        (cx + s * 0.36, cy + s * 0.28),
                        (cx - s * 0.36, cy + s * 0.28),
                    ],
                    true,
                    color,
                    sw,
                )?;
                // Down arrow inside folder
                self.draw_line(cx, cy - s * 0.06, cx, cy + s * 0.18, color, sw)?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.12, cy + s * 0.07),
                        (cx, cy + s * 0.19),
                        (cx + s * 0.12, cy + s * 0.07),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::Search => {
                let r = s * 0.22;
                let gx = cx - s * 0.07;
                let gy = cy - s * 0.07;
                self.stroke_circle(gx, gy, r, color, sw)?;
                let edge = r * std::f32::consts::FRAC_1_SQRT_2;
                self.draw_line(gx + edge, gy + edge, cx + s * 0.31, cy + s * 0.31, color, sw)?;
            }
            SfSymbol::Sparkle => {
                self.draw_sparkles_icon(cx, cy, s, color)?;
            }
            SfSymbol::Pin => {
                self.draw_line(cx - s * 0.20, cy - s * 0.32, cx + s * 0.20, cy - s * 0.32, color, sw)?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.12, cy - s * 0.32),
                        (cx - s * 0.14, cy + s * 0.02),
                        (cx - s * 0.24, cy + s * 0.10),
                        (cx + s * 0.24, cy + s * 0.10),
                        (cx + s * 0.14, cy + s * 0.02),
                        (cx + s * 0.12, cy - s * 0.32),
                    ],
                    false,
                    color,
                    sw,
                )?;
                self.draw_line(cx, cy + s * 0.10, cx, cy + s * 0.36, color, sw)?;
            }
            SfSymbol::Calendar => {
                self.draw_calendar_icon(cx, cy, s, color)?;
            }
            SfSymbol::FocusTimer => {
                let r = s * 0.29;
                let ty = cy + s * 0.04;
                self.stroke_circle(cx, ty, r, color, sw)?;
                self.draw_line(cx - s * 0.09, cy - s * 0.34, cx + s * 0.09, cy - s * 0.34, color, sw)?;
                self.draw_line(cx, ty, cx, ty - r * 0.56, color, sw)?;
                self.draw_line(cx, ty, cx + r * 0.45, ty + r * 0.20, color, sw)?;
            }
            SfSymbol::ShortcutKeys => {
                // Windows-native 4-pane shortcut keycap (`[Win]`)
                self.stroke_rounded_rect(
                    Rect::new(cx - s * 0.36, cy - s * 0.32, s * 0.72, s * 0.64),
                    (s * 0.12).max(2.0),
                    color,
                    sw,
                )?;
                let pane = s * 0.15;
                let off = s * 0.105;
                let pr = (s * 0.03).max(0.8);
                for dx in [-off, off] {
                    for dy in [-off, off] {
                        self.fill_rounded_rect(
                            Rect::new(cx + dx - pane * 0.5, cy + dy - pane * 0.5, pane, pane),
                            pr,
                            color,
                        )?;
                    }
                }
            }
            SfSymbol::Gear => {
                self.draw_settings_icon(cx, cy, s, color)?;
            }
            SfSymbol::MusicNote => {
                self.draw_music_note_icon(cx, cy, s, color)?;
            }
            SfSymbol::Bell => {
                self.draw_bell_icon(cx, cy, s, color)?;
            }
            SfSymbol::Bluetooth => {
                self.draw_bluetooth_icon(cx, cy, s, color)?;
            }
            SfSymbol::Copy => {
                let r = (s * 0.06).max(1.2);
                self.stroke_rounded_rect(
                    Rect::new(cx - s * 0.28, cy - s * 0.28, s * 0.42, s * 0.46),
                    r,
                    color.with_alpha_mul(0.72),
                    sw,
                )?;
                self.stroke_rounded_rect(
                    Rect::new(cx - s * 0.14, cy - s * 0.14, s * 0.42, s * 0.46),
                    r,
                    color,
                    sw,
                )?;
            }
            SfSymbol::SendArrow => {
                self.draw_line(cx, cy + s * 0.28, cx, cy - s * 0.26, color, sw)?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.22, cy - s * 0.06),
                        (cx, cy - s * 0.28),
                        (cx + s * 0.22, cy - s * 0.06),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::ShieldCheck => {
                self.draw_polyline(
                    &[
                        (cx, cy - s * 0.34),
                        (cx + s * 0.28, cy - s * 0.22),
                        (cx + s * 0.24, cy + s * 0.10),
                        (cx, cy + s * 0.34),
                        (cx - s * 0.24, cy + s * 0.10),
                        (cx - s * 0.28, cy - s * 0.22),
                    ],
                    true,
                    color,
                    sw,
                )?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.12, cy + s * 0.02),
                        (cx - s * 0.02, cy + s * 0.12),
                        (cx + s * 0.15, cy - s * 0.08),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::ListChecklist => {
                for dy in [-0.20_f32, 0.02, 0.24] {
                    let y = cy + s * dy;
                    self.fill_circle(cx - s * 0.24, y, (sw * 0.55).max(1.0), color)?;
                    self.draw_line(cx - s * 0.08, y, cx + s * 0.30, y, color, sw)?;
                }
            }
            SfSymbol::Terminal => {
                self.stroke_rounded_rect(
                    Rect::new(cx - s * 0.36, cy - s * 0.28, s * 0.72, s * 0.56),
                    (s * 0.08).max(1.5),
                    color,
                    sw,
                )?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.20, cy - s * 0.10),
                        (cx - s * 0.06, cy + s * 0.02),
                        (cx - s * 0.20, cy + s * 0.14),
                    ],
                    false,
                    color,
                    sw,
                )?;
                self.draw_line(cx + s * 0.02, cy + s * 0.14, cx + s * 0.20, cy + s * 0.14, color, sw)?;
            }
            SfSymbol::Document => {
                self.draw_polyline(
                    &[
                        (cx - s * 0.24, cy - s * 0.34),
                        (cx + s * 0.10, cy - s * 0.34),
                        (cx + s * 0.24, cy - s * 0.18),
                        (cx + s * 0.24, cy + s * 0.34),
                        (cx - s * 0.24, cy + s * 0.34),
                    ],
                    true,
                    color,
                    sw,
                )?;
                self.draw_line(cx - s * 0.12, cy - s * 0.04, cx + s * 0.12, cy - s * 0.04, color, sw * 0.85)?;
                self.draw_line(cx - s * 0.12, cy + s * 0.12, cx + s * 0.12, cy + s * 0.12, color, sw * 0.85)?;
            }
            SfSymbol::Globe => {
                let r = s * 0.33;
                self.stroke_circle(cx, cy, r, color, sw)?;
                self.draw_line(cx - r, cy, cx + r, cy, color, sw * 0.85)?;
                self.draw_line(cx, cy - r, cx, cy + r, color, sw * 0.85)?;
                unsafe {
                    let brush = self.create_brush(color)?;
                    self.d2d_target.DrawEllipse(
                        &D2D1_ELLIPSE {
                            point: pt(cx, cy),
                            radiusX: r * 0.48,
                            radiusY: r,
                        },
                        &brush,
                        sw * 0.85,
                        None,
                    );
                }
            }
            SfSymbol::Refresh => {
                let r = s * 0.28;
                unsafe {
                    let brush = self.create_brush(color)?;
                    let round_cap = self.create_round_cap_stroke()?;
                    let arc = self.build_quadratic_arc(
                        pt(cx + r * 0.85, cy - r * 0.45),
                        pt(cx - r * 0.8, cy - r * 1.15),
                        pt(cx - r * 0.85, cy + r * 0.45),
                    )?;
                    self.d2d_target
                        .DrawGeometry(&arc, &brush, sw, Some(&round_cap));
                    let arc2 = self.build_quadratic_arc(
                        pt(cx - r * 0.85, cy + r * 0.45),
                        pt(cx + r * 0.4, cy + r * 1.15),
                        pt(cx + r * 0.90, cy + r * 0.25),
                    )?;
                    self.d2d_target
                        .DrawGeometry(&arc2, &brush, sw, Some(&round_cap));
                }
                self.draw_polyline(
                    &[
                        (cx + s * 0.14, cy - s * 0.26),
                        (cx + s * 0.27, cy - s * 0.12),
                        (cx + s * 0.12, cy - s * 0.04),
                    ],
                    false,
                    color,
                    sw,
                )?;
            }
            SfSymbol::StopCircle => {
                let r = s * 0.34;
                self.stroke_circle(cx, cy, r, color, sw)?;
                let d = r * 0.68;
                self.draw_line(cx - d, cy - d, cx + d, cy + d, color, sw)?;
            }
            SfSymbol::PrivateMask => {
                // Minimal monochrome private-browsing glasses/hat glyph
                self.draw_line(cx - s * 0.38, cy - s * 0.08, cx + s * 0.38, cy - s * 0.08, color, sw)?;
                self.draw_polyline(
                    &[
                        (cx - s * 0.22, cy - s * 0.08),
                        (cx - s * 0.15, cy - s * 0.32),
                        (cx + s * 0.15, cy - s * 0.32),
                        (cx + s * 0.22, cy - s * 0.08),
                    ],
                    false,
                    color,
                    sw,
                )?;
                let lr = s * 0.13;
                self.stroke_circle(cx - s * 0.18, cy + s * 0.16, lr, color, sw)?;
                self.stroke_circle(cx + s * 0.18, cy + s * 0.16, lr, color, sw)?;
                self.draw_line(cx - s * 0.05, cy + s * 0.16, cx + s * 0.05, cy + s * 0.16, color, sw)?;
            }
        }
        Ok(())
    }

    fn draw_polyline(
        &self,
        pts: &[(f32, f32)],
        closed: bool,
        color: Color,
        stroke_width: f32,
    ) -> Result<()> {
        if pts.len() < 2 {
            return Ok(());
        }
        unsafe {
            let brush = self.create_brush(color)?;
            let round_cap = self.create_round_cap_stroke()?;
            let geo = self.d2d_factory.CreatePathGeometry()?;
            let sink = geo.Open()?;
            sink.BeginFigure(pt(pts[0].0, pts[0].1), D2D1_FIGURE_BEGIN_HOLLOW);
            for &(x, y) in &pts[1..] {
                sink.AddLine(pt(x, y));
            }
            sink.EndFigure(if closed {
                D2D1_FIGURE_END_CLOSED
            } else {
                D2D1_FIGURE_END_OPEN
            });
            sink.Close()?;
            self.d2d_target
                .DrawGeometry(&geo, &brush, stroke_width.max(0.5), Some(&round_cap));
            Ok(())
        }
    }

    /// Draws a rotating brightness sun icon inside `rect` with rays rotated by `angle_rad`.
    pub fn draw_brightness_sun(
        &self,
        rect: Rect,
        angle_rad: f32,
        color: Color,
        stroke_width: f32,
    ) -> Result<()> {
        let s = rect.width.min(rect.height).max(8.0);
        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;
        let r = (s * 0.22).max(2.0);
        let sw = if stroke_width > 0.0 {
            stroke_width.max(0.8)
        } else {
            (s * 0.105).clamp(1.1, 2.4)
        };
        self.stroke_circle(cx, cy, r, color, sw)?;
        let inner = r + s * 0.11;
        let outer = s * 0.46;
        for i in 0..8 {
            let angle = angle_rad + (i as f32) * std::f32::consts::FRAC_PI_4;
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

    /// Draws a crisp sun icon (brightness) centered at `(cx, cy)`.
    pub fn draw_sun_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        self.draw_brightness_sun(
            Rect::new(cx - size * 0.5, cy - size * 0.5, size, size),
            0.0,
            color,
            (size * 0.1).max(1.1),
        )
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

    /// Draws a 4-point SF-style sparkle icon centered at `(cx, cy)`.
    pub fn draw_sparkles_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let r = s * 0.36;
        let k = s * 0.08;
        unsafe {
            let brush = self.create_brush(color)?;
            let geo = self.d2d_factory.CreatePathGeometry()?;
            let sink = geo.Open()?;
            sink.BeginFigure(pt(cx, cy - r), D2D1_FIGURE_BEGIN_FILLED);
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: pt(cx, cy - k),
                point2: pt(cx + k, cy),
                point3: pt(cx + r, cy),
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: pt(cx + k, cy),
                point2: pt(cx, cy + k),
                point3: pt(cx, cy + r),
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: pt(cx, cy + k),
                point2: pt(cx - k, cy),
                point3: pt(cx - r, cy),
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: pt(cx - k, cy),
                point2: pt(cx, cy - k),
                point3: pt(cx, cy - r),
            });
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            self.d2d_target.FillGeometry(&geo, &brush, None);
            Ok(())
        }
    }

    /// Draws a notification bell icon centered at `(cx, cy)`.
    /// Draws a classic SF Symbols-style notification bell outline centered at `(cx, cy)`.
    pub fn draw_bell_icon(&self, cx: f32, cy: f32, size: f32, color: Color) -> Result<()> {
        let s = size.max(8.0);
        let sw = (s * 0.095).max(1.2);
        // Top finial stem
        self.draw_line(cx, cy - s * 0.38, cx, cy - s * 0.31, color, sw)?;
        // Classic flared bell body (smooth dome -> straight waist -> flared rim)
        let body = [
            (cx - s * 0.34, cy + s * 0.18),
            (cx - s * 0.25, cy + s * 0.07),
            (cx - s * 0.23, cy - s * 0.08),
            (cx - s * 0.19, cy - s * 0.20),
            (cx - s * 0.10, cy - s * 0.28),
            (cx, cy - s * 0.31),
            (cx + s * 0.10, cy - s * 0.28),
            (cx + s * 0.19, cy - s * 0.20),
            (cx + s * 0.23, cy - s * 0.08),
            (cx + s * 0.25, cy + s * 0.07),
            (cx + s * 0.34, cy + s * 0.18),
        ];
        self.draw_polyline(&body, true, color, sw)?;
        // Bottom clapper arc
        let clapper = [
            (cx - s * 0.09, cy + s * 0.25),
            (cx - s * 0.05, cy + s * 0.33),
            (cx + s * 0.05, cy + s * 0.33),
            (cx + s * 0.09, cy + s * 0.25),
        ];
        self.draw_polyline(&clapper, false, color, sw)?;
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
            lineJoin: D2D1_LINE_JOIN_ROUND,
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
            self.glass_bitmap_cache.borrow_mut().clear();
            self.cached_layer.borrow_mut().take();
            self.last_region_key.borrow_mut().take();
            let new_dib = BackingDib::new(width, height)?;
            *self.dib.borrow_mut() = new_dib;
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
        self.glass_bitmap_cache.borrow_mut().clear();
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

/// Resolves flat fallback `(base_fill, hairline_border)` from [`AdaptivePalette`] when no
/// live backdrop snapshot is available (zero Windows 7 diagonal interior gradients).
fn resolve_fallback_glass_colors(
    palette: &AdaptivePalette,
    style: LiquidGlassStyle,
) -> (Color, Color) {
    let lift = if style.pressed {
        0.06
    } else if style.hovered {
        0.03
    } else {
        0.0
    };
    let base_fill = match style.variant {
        LiquidGlassVariant::Regular => palette
            .glass_regular
            .with_alpha((palette.glass_regular.a + lift).clamp(0.62, 0.90)),
        LiquidGlassVariant::Clear => palette
            .glass_clear
            .with_alpha((palette.glass_clear.a + lift).clamp(0.55, 0.84)),
        LiquidGlassVariant::Elevated => {
            if style.pressed {
                palette.control_active.lerp(palette.glass_elevated, 0.6)
            } else if style.hovered {
                palette.control_hover.lerp(palette.glass_elevated, 0.7)
            } else {
                palette.glass_elevated
            }
        }
        LiquidGlassVariant::Tinted(tint) => {
            let alpha = (tint.a.clamp(0.26, 0.84) + lift).clamp(0.26, 0.88);
            palette.glass_regular.lerp(tint.with_alpha(alpha), 0.72)
        }
    };
    let border = match style.variant {
        LiquidGlassVariant::Tinted(tint) => tint
            .lerp(Color::from_rgb_u8(0xFF, 0xFF, 0xFF), 0.45)
            .with_alpha(0.45),
        _ => palette.hairline_border,
    };
    (base_fill, border)
}

fn compute_avg_luminance(width: u32, height: u32, bgra_premul: &[u8]) -> f32 {
    let total_px = (width as usize).saturating_mul(height as usize);
    if total_px == 0 {
        return 0.0;
    }
    let step = (total_px / 1024).max(1);
    let mut sum = 0.0_f64;
    let mut count = 0_usize;
    let mut idx = 0_usize;
    while idx < total_px {
        let o = idx * 4;
        let b = bgra_premul[o] as f32;
        let g = bgra_premul[o + 1] as f32;
        let r = bgra_premul[o + 2] as f32;
        let a = bgra_premul[o + 3] as f32;
        let (rf, gf, bf) = if a > 0.5 {
            ((r / a).min(1.0), (g / a).min(1.0), (b / a).min(1.0))
        } else {
            (r / 255.0, g / 255.0, b / 255.0)
        };
        sum += (0.2126 * rf + 0.7152 * gf + 0.0722 * bf) as f64;
        count += 1;
        idx += step;
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).clamp(0.0, 1.0) as f32
    }
}

/// Downsamples `src` 2x and runs a 3-pass O(W*H) separable box blur (converging to Gaussian
/// `sigma ~ 12px` at full resolution) for smooth Liquid Glass `frost`.
fn blur_bgra_half_res(width: u32, height: u32, src: &[u8]) -> (u32, u32, Vec<u8>) {
    let bw = (width / 2).max(1);
    let bh = (height / 2).max(1);
    let mut buf = vec![0u8; (bw as usize) * (bh as usize) * 4];
    for y in 0..bh {
        let sy0 = (y * 2).min(height - 1) as usize;
        let sy1 = (y * 2 + 1).min(height - 1) as usize;
        for x in 0..bw {
            let sx0 = (x * 2).min(width - 1) as usize;
            let sx1 = (x * 2 + 1).min(width - 1) as usize;
            let i00 = (sy0 * width as usize + sx0) * 4;
            let i10 = (sy0 * width as usize + sx1) * 4;
            let i01 = (sy1 * width as usize + sx0) * 4;
            let i11 = (sy1 * width as usize + sx1) * 4;
            let dst = (y as usize * bw as usize + x as usize) * 4;
            for c in 0..4 {
                let avg = (src[i00 + c] as u32
                    + src[i10 + c] as u32
                    + src[i01 + c] as u32
                    + src[i11 + c] as u32)
                    >> 2;
                buf[dst + c] = avg as u8;
            }
        }
    }

    let mut tmp = vec![0u8; buf.len()];
    let radius = (bw.min(bh) as i32 / 2).clamp(3, 8);
    for _ in 0..3 {
        box_blur_h(&buf, &mut tmp, bw as usize, bh as usize, radius);
        box_blur_v(&tmp, &mut buf, bw as usize, bh as usize, radius);
    }
    (bw, bh, buf)
}

fn box_blur_h(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: i32) {
    let win = (radius * 2 + 1) as u32;
    let w_max = w as i32 - 1;
    for y in 0..h {
        let row = y * w * 4;
        let mut sb = 0_u32;
        let mut sg = 0_u32;
        let mut sr = 0_u32;
        let mut sa = 0_u32;
        for k in -radius..=radius {
            let sx = k.clamp(0, w_max) as usize;
            let o = row + sx * 4;
            sb += src[o] as u32;
            sg += src[o + 1] as u32;
            sr += src[o + 2] as u32;
            sa += src[o + 3] as u32;
        }
        for x in 0..w {
            let o = row + x * 4;
            dst[o] = (sb / win) as u8;
            dst[o + 1] = (sg / win) as u8;
            dst[o + 2] = (sr / win) as u8;
            dst[o + 3] = (sa / win) as u8;

            let rem_x = (x as i32 - radius).clamp(0, w_max) as usize;
            let add_x = (x as i32 + radius + 1).clamp(0, w_max) as usize;
            let ro = row + rem_x * 4;
            let ao = row + add_x * 4;
            sb = sb + src[ao] as u32 - src[ro] as u32;
            sg = sg + src[ao + 1] as u32 - src[ro + 1] as u32;
            sr = sr + src[ao + 2] as u32 - src[ro + 2] as u32;
            sa = sa + src[ao + 3] as u32 - src[ro + 3] as u32;
        }
    }
}

fn box_blur_v(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: i32) {
    let win = (radius * 2 + 1) as u32;
    let h_max = h as i32 - 1;
    let stride = w * 4;
    for x in 0..w {
        let col = x * 4;
        let mut sb = 0_u32;
        let mut sg = 0_u32;
        let mut sr = 0_u32;
        let mut sa = 0_u32;
        for k in -radius..=radius {
            let sy = k.clamp(0, h_max) as usize;
            let o = sy * stride + col;
            sb += src[o] as u32;
            sg += src[o + 1] as u32;
            sr += src[o + 2] as u32;
            sa += src[o + 3] as u32;
        }
        for y in 0..h {
            let o = y * stride + col;
            dst[o] = (sb / win) as u8;
            dst[o + 1] = (sg / win) as u8;
            dst[o + 2] = (sr / win) as u8;
            dst[o + 3] = (sa / win) as u8;

            let rem_y = (y as i32 - radius).clamp(0, h_max) as usize;
            let add_y = (y as i32 + radius + 1).clamp(0, h_max) as usize;
            let ro = rem_y * stride + col;
            let ao = add_y * stride + col;
            sb = sb + src[ao] as u32 - src[ro] as u32;
            sg = sg + src[ao + 1] as u32 - src[ro + 1] as u32;
            sr = sr + src[ao + 2] as u32 - src[ro + 2] as u32;
            sa = sa + src[ao + 3] as u32 - src[ro + 3] as u32;
        }
    }
}

/// Squircle rounded-rectangle signed distance & outward normal (`liquid-rust` `sdg_rrect`, `n = 2.79`).
#[inline]
fn sdg_rrect(px: f32, py: f32, hw: f32, hh: f32, r: f32, soft: f32) -> (f32, f32, f32) {
    let sx = if px >= 0.0 { 1.0 } else { -1.0 };
    let sy = if py >= 0.0 { 1.0 } else { -1.0 };
    let wx = px.abs() - (hw - r);
    let wy = py.abs() - (hh - r);
    let g = wx.max(wy);
    if g > 0.0 {
        // Exact Euclidean circular corner arc (L^2) matching Direct2D's `D2D1_ROUNDED_RECT`
        // so glass refraction and 1px border geometry align with zero corner overhang.
        let qx = wx.max(1e-7);
        let qy = wy.max(1e-7);
        let l = qx.hypot(qy).max(1e-6);
        return (l - r, sx * qx / l, sy * qy / l);
    }
    let inv_s = 1.0 / soft.max(1e-3);
    let ex = ((wx - g) * inv_s).exp();
    let ey = ((wy - g) * inv_s).exp();
    let el = ex.hypot(ey).max(1e-6);
    (g - r, sx * ex / el, sy * ey / el)
}

/// Signed distance & outward normal for an `EdgeSurface` contour (with optional concave ears).
#[inline]
fn sdg_edge_contour(
    lx: f32,
    ly: f32,
    lw: f32,
    lh: f32,
    ear_w: f32,
    radius: f32,
    edge: EdgeSide,
    flares: bool,
) -> (f32, f32, f32) {
    let (clw, clh, ux, uy) = match edge {
        EdgeSide::Top => (lw, lh, lx, ly),
        EdgeSide::Bottom => (lw, lh, lx, lh - ly),
        EdgeSide::Left => (lh, lw, ly, lx),
        EdgeSide::Right => (lh, lw, ly, lw - lx),
    };
    let ear = if flares {
        ear_w.max(0.0).min(clw * 0.25)
    } else {
        0.0
    };
    let left = ear;
    let right = (clw - ear).max(left + 1.0);
    let body_w = right - left;
    let r = radius.max(0.0).min(clh * 0.5).min(body_w * 0.5);

    // Canonical top-anchored body: rounded only on bottom corners unless Bottom+!flares.
    let (mut d, mut unx, mut uny) = if edge == EdgeSide::Bottom && !flares && r > 0.0 {
        sdg_rrect(
            ux - (left + body_w * 0.5),
            uy - clh * 0.5,
            body_w * 0.5,
            clh * 0.5,
            r,
            6.0,
        )
    } else {
        // Shift top boundary above y=0 so top bezel edge has no rounded corners or inward bevel.
        let top_pad = 64.0_f32;
        let eff_h = clh + top_pad;
        let (bd, bnx, bny) = sdg_rrect(
            ux - (left + body_w * 0.5),
            (uy + top_pad) - eff_h * 0.5,
            body_w * 0.5,
            eff_h * 0.5,
            r,
            6.0,
        );
        // Clamp against y=0 flush edge
        let top_d = -uy;
        if top_d > bd {
            (top_d, 0.0, -1.0)
        } else {
            (bd, bnx, bny)
        }
    };

    if flares && ear > 0.0 && uy <= ear && uy < (clh - r).min(clh * 0.5).max(ear) {
        let tail_len = (clh - r - ear).max(0.0);
        if ux < clw * 0.5 {
            let (ear_d, enx, eny) = sdg_canonical_ear(ux, uy, ear, tail_len);
            if ux < left || ear_d >= ux - right {
                d = ear_d;
                unx = enx;
                uny = eny;
            }
        } else {
            let mx = clw - ux;
            let (ear_d, mnx, eny) = sdg_canonical_ear(mx, uy, ear, tail_len);
            if mx < left || ear_d >= mx - right {
                d = ear_d;
                unx = -mnx;
                uny = eny;
            }
        }
    }

    let (nx, ny) = match edge {
        EdgeSide::Top => (unx, uny),
        EdgeSide::Bottom => (unx, -uny),
        EdgeSide::Left => (uny, unx),
        EdgeSide::Right => (-uny, unx),
    };
    (d, nx, ny)
}

/// Signed distance & outward normal from canonical `(px, py)` to the `edge_contour_points`
/// concave flare cubic Bezier (`c_off = ear * 10/13`) plus vertical wall continuation.
#[inline]
fn sdg_canonical_ear(px: f32, py: f32, ear: f32, tail_len: f32) -> (f32, f32, f32) {
    if px < 0.0 {
        let cy = py.max(0.0);
        let dy = py - cy;
        let dist = px.hypot(dy).max(1e-5);
        return (dist, px / dist, dy / dist);
    }

    const SEGS: usize = 12;
    let c_off = ear * (10.0 / 13.0);
    let eval_pt = |t: f32| -> (f32, f32) {
        let u = 1.0 - t;
        let u2 = u * u;
        let t2 = t * t;
        let b1 = 3.0 * u2 * t;
        let b2 = 3.0 * u * t2;
        let b3 = t2 * t;
        (
            b1 * c_off + b2 * ear + b3 * ear,
            b2 * (ear - c_off) + b3 * ear,
        )
    };

    let mut min_dist2 = f32::MAX;
    let mut best_dx = 0.0_f32;
    let mut best_dy = 0.0_f32;
    let mut best_nx = -1.0_f32;
    let mut best_ny = 0.0_f32;
    let mut curve_x = ear;
    let mut found_scan = false;

    let mut prev = (0.0_f32, 0.0_f32);
    for i in 1..=SEGS {
        let curr = eval_pt(i as f32 / SEGS as f32);
        if !found_scan && py >= prev.1 && py <= curr.1 {
            let dy_seg = curr.1 - prev.1;
            let frac = if dy_seg > 1e-6 {
                ((py - prev.1) / dy_seg).clamp(0.0, 1.0)
            } else {
                1.0
            };
            curve_x = prev.0 + frac * (curr.0 - prev.0);
            found_scan = true;
        }
        let vx = curr.0 - prev.0;
        let vy = curr.1 - prev.1;
        let len2 = (vx * vx + vy * vy).max(1e-8);
        let proj = (((px - prev.0) * vx + (py - prev.1) * vy) / len2).clamp(0.0, 1.0);
        let cx = prev.0 + proj * vx;
        let cy = prev.1 + proj * vy;
        let dx = px - cx;
        let dy = py - cy;
        let d2 = dx * dx + dy * dy;
        if d2 < min_dist2 {
            min_dist2 = d2;
            best_dx = dx;
            best_dy = dy;
            let len = len2.sqrt();
            best_nx = -vy / len;
            best_ny = vx / len;
        }
        prev = curr;
    }

    if tail_len > 0.0 {
        let ty_end = ear + tail_len;
        let cy = py.clamp(ear, ty_end);
        let dx = px - ear;
        let dy = py - cy;
        let d2 = dx * dx + dy * dy;
        if d2 < min_dist2 {
            min_dist2 = d2;
            best_dx = dx;
            best_dy = dy;
            best_nx = -1.0;
            best_ny = 0.0;
        }
    }

    let sign = if py >= ear {
        if px >= ear { -1.0 } else { 1.0 }
    } else if px >= curve_x {
        -1.0
    } else {
        1.0
    };

    let dist = min_dist2.sqrt();
    let (nx, ny) = if dist > 1e-4 {
        ((best_dx / dist) * sign, (best_dy / dist) * sign)
    } else {
        (best_nx, best_ny)
    };
    let ear_d = dist * sign;
    let top_d = -py;
    if top_d > ear_d {
        (top_d, 0.0, -1.0)
    } else {
        (ear_d, nx, ny)
    }
}

#[inline]
fn sample_bgra_bilinear(bgra: &[u8], w: u32, h: u32, x: f32, y: f32) -> [f32; 3] {
    let wf = (w as i32 - 1).max(0) as f32;
    let hf = (h as i32 - 1).max(0) as f32;
    let cx = x.clamp(0.0, wf);
    let cy = y.clamp(0.0, hf);
    let x0 = cx.floor() as u32;
    let y0 = cy.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = cx - x0 as f32;
    let fy = cy - y0 as f32;

    let idx = |px: u32, py: u32| ((py as usize) * (w as usize) + (px as usize)) * 4;
    let i00 = idx(x0, y0);
    let i10 = idx(x1, y0);
    let i01 = idx(x0, y1);
    let i11 = idx(x1, y1);

    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;

    let inv255 = 1.0 / 255.0;
    let r = (bgra[i00 + 2] as f32 * w00
        + bgra[i10 + 2] as f32 * w10
        + bgra[i01 + 2] as f32 * w01
        + bgra[i11 + 2] as f32 * w11)
        * inv255;
    let g = (bgra[i00 + 1] as f32 * w00
        + bgra[i10 + 1] as f32 * w10
        + bgra[i01 + 1] as f32 * w01
        + bgra[i11 + 1] as f32 * w11)
        * inv255;
    let b = (bgra[i00] as f32 * w00
        + bgra[i10] as f32 * w10
        + bgra[i01] as f32 * w01
        + bgra[i11] as f32 * w11)
        * inv255;
    [r, g, b]
}

static REFRACT_LUT: OnceLock<[f32; 65]> = OnceLock::new();

fn bevel_refract_lut() -> &'static [f32; 65] {
    REFRACT_LUT.get_or_init(|| {
        let mut lut = [0.0_f32; 65];
        for (i, slot) in lut.iter_mut().enumerate() {
            let t = (i as f32 / 64.0).clamp(0.0, 1.0);
            let u = 1.0 - t;
            let u3 = u * u * u;
            let slope = (u3 * (1.0 - u3 * u).max(1e-4).powf(-0.75)).min(36.0);
            let theta = slope.atan();
            *slot = (theta - (theta.sin() / 1.5).clamp(-1.0, 1.0).asin()).tan();
        }
        lut
    })
}

#[inline]
fn lookup_refract(t: f32) -> f32 {
    let pos = t.clamp(0.0, 1.0) * 64.0;
    let idx = (pos as usize).min(63);
    let frac = pos - idx as f32;
    let lut = bevel_refract_lut();
    lut[idx] * (1.0 - frac) + lut[idx + 1] * frac
}

#[allow(clippy::too_many_arguments)]
fn rasterize_liquid_glass(
    bd: &BackdropState,
    surf_w: u32,
    surf_h: u32,
    px_x: f32,
    px_y: f32,
    px_w: u32,
    px_h: u32,
    gw: u32,
    gh: u32,
    scale: f32,
    style: LiquidGlassStyle,
    polarity: ThemePolarity,
    edge_params: Option<(f32, f32, EdgeSide, bool)>,
) -> Vec<u8> {
    let mut out = vec![0u8; (gw as usize) * (gh as usize) * 4];
    let w = px_w as f32;
    let h = px_h as f32;
    let hw = w * 0.5;
    let hh = h * 0.5;
    let r_px = (style.corner_radius * scale)
        .max(0.0)
        .min(hw)
        .min(hh);
    let depth = (w.min(h) * 0.34).min(22.0 * scale).max(2.0);
    let soft = (depth * 0.35).max(3.0);
    let flat_hw = (hw - depth - 1.5).max(0.0);
    let flat_hh = (hh - depth - 1.5).max(0.0);
    let can_skip_center_sdf = edge_params.is_none() && flat_hw > 0.0 && flat_hh > 0.0;

    let (refract_reach, tone_mix) = match style.variant {
        LiquidGlassVariant::Regular => (depth * 0.55, 0.30_f32),
        LiquidGlassVariant::Clear => (depth * 0.72, 0.09_f32),
        LiquidGlassVariant::Elevated => (depth * 0.42, 0.38_f32),
        LiquidGlassVariant::Tinted(_) => (depth * 0.50, 0.30_f32),
    };
    let tone_target = match polarity {
        ThemePolarity::LightGlass => [0.955_f32, 0.962_f32, 0.978_f32],
        ThemePolarity::DarkGlass => [0.075_f32, 0.078_f32, 0.095_f32],
    };

    let inv_surf_w = 1.0 / surf_w.max(1) as f32;
    let inv_surf_h = 1.0 / surf_h.max(1) as f32;
    let step_x = w / gw.max(1) as f32;
    let step_y = h / gh.max(1) as f32;
    let bw_f = bd.blurred_w as f32;
    let bh_f = bd.blurred_h as f32;
    let rw_f = bd.raw.width as f32;
    let rh_f = bd.raw.height as f32;

    let tint_opt = match style.variant {
        LiquidGlassVariant::Tinted(c) => Some(c),
        _ => None,
    };
    let hover_lift = if style.pressed {
        0.05_f32
    } else if style.hovered {
        0.03_f32
    } else {
        0.0
    };

    for iy in 0..gh {
        let ly = (iy as f32 + 0.5) * step_y;
        let gy = (px_y + ly) * inv_surf_h;
        let rel_y = ly - hh;
        let in_flat_y = can_skip_center_sdf && rel_y.abs() <= flat_hh;

        for ix in 0..gw {
            let lx = (ix as f32 + 0.5) * step_x;
            let rel_x = lx - hw;
            let gx = (px_x + lx) * inv_surf_w;

            // Fast interior path: strictly inside the flat center box (t == 1.0, cov == 1.0)
            if in_flat_y && rel_x.abs() <= flat_hw {
                let frosted = sample_bgra_bilinear(
                    &bd.blurred_bgra,
                    bd.blurred_w,
                    bd.blurred_h,
                    gx * bw_f,
                    gy * bh_f,
                );
                let mut rgb = [
                    frosted[0] * (1.0 - tone_mix) + tone_target[0] * tone_mix,
                    frosted[1] * (1.0 - tone_mix) + tone_target[1] * tone_mix,
                    frosted[2] * (1.0 - tone_mix) + tone_target[2] * tone_mix,
                ];
                if let Some(tint) = tint_opt {
                    let ta = tint.a.clamp(0.22, 0.65);
                    rgb[0] = rgb[0] * (1.0 - ta) + tint.r * ta;
                    rgb[1] = rgb[1] * (1.0 - ta) + tint.g * ta;
                    rgb[2] = rgb[2] * (1.0 - ta) + tint.b * ta;
                }
                if hover_lift > 0.0 {
                    for c in &mut rgb {
                        *c = (*c + hover_lift * 0.6).min(1.0);
                    }
                }
                let idx = ((iy as usize) * (gw as usize) + (ix as usize)) * 4;
                out[idx] = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                out[idx + 1] = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                out[idx + 2] = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                out[idx + 3] = 255;
                continue;
            }

            let (d, nx, ny) = if let Some((ear_w, er, edge, flares)) = edge_params {
                sdg_edge_contour(lx, ly, w, h, ear_w * scale, er * scale, edge, flares)
            } else {
                sdg_rrect(rel_x, rel_y, hw, hh, r_px, soft)
            };

            let cov = (0.5 - d * 0.5).clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }

            let t = (-d / depth).clamp(0.0, 1.0);

            // Snell bevel refraction + subtle RGB dispersion via precomputed 65-entry LUT
            let frosted = if t < 1.0 {
                let refr = lookup_refract(t);
                let off_uv_x = (refract_reach * refr * nx) * inv_surf_w;
                let off_uv_y = (refract_reach * refr * ny) * inv_surf_h;

                let sample_blur_uv = |ux: f32, uy: f32| {
                    sample_bgra_bilinear(
                        &bd.blurred_bgra,
                        bd.blurred_w,
                        bd.blurred_h,
                        ux * bw_f,
                        uy * bh_f,
                    )
                };
                let r_c = sample_blur_uv(gx - off_uv_x * 0.90, gy - off_uv_y * 0.90)[0];
                let g_c = sample_blur_uv(gx - off_uv_x, gy - off_uv_y)[1];
                let b_c = sample_blur_uv(gx - off_uv_x * 1.10, gy - off_uv_y * 1.10)[2];
                let sharp = sample_bgra_bilinear(
                    &bd.raw.bgra_premul,
                    bd.raw.width,
                    bd.raw.height,
                    (gx - off_uv_x) * rw_f,
                    (gy - off_uv_y) * rh_f,
                );
                let clarity = (1.0 - t) * 0.35;
                [
                    r_c * (1.0 - clarity) + sharp[0] * clarity,
                    g_c * (1.0 - clarity) + sharp[1] * clarity,
                    b_c * (1.0 - clarity) + sharp[2] * clarity,
                ]
            } else {
                sample_bgra_bilinear(
                    &bd.blurred_bgra,
                    bd.blurred_w,
                    bd.blurred_h,
                    gx * bw_f,
                    gy * bh_f,
                )
            };

            // Adaptive tone mapping (Light Frosted White Glass vs Dark Smoked Glass)
            let mut rgb = [
                frosted[0] * (1.0 - tone_mix) + tone_target[0] * tone_mix,
                frosted[1] * (1.0 - tone_mix) + tone_target[1] * tone_mix,
                frosted[2] * (1.0 - tone_mix) + tone_target[2] * tone_mix,
            ];

            if let Some(tint) = tint_opt {
                let ta = tint.a.clamp(0.22, 0.65);
                rgb[0] = rgb[0] * (1.0 - ta) + tint.r * ta;
                rgb[1] = rgb[1] * (1.0 - ta) + tint.g * ta;
                rgb[2] = rgb[2] * (1.0 - ta) + tint.b * ta;
            }

            if hover_lift > 0.0 {
                for c in &mut rgb {
                    *c = (*c + hover_lift * 0.6).min(1.0);
                }
            }

            // Crisp optical edge specular rim only near the outer edge
            if d > -3.0 * scale {
                let rim_band = (-d.abs() / (1.5 * scale)).exp();
                if rim_band > 0.02 {
                    let facing = nx * (-0.45) + ny * (-0.89);
                    let dir_w = if facing > 0.0 {
                        0.22 + 0.78 * facing * facing
                    } else {
                        0.12 + 0.22 * facing * facing
                    };
                    let rim_strength = match polarity {
                        ThemePolarity::LightGlass => 0.36 * rim_band * dir_w,
                        ThemePolarity::DarkGlass => 0.26 * rim_band * dir_w,
                    };
                    for c in &mut rgb {
                        *c = (*c + rim_strength * (1.0 - *c * 0.35)).min(1.0);
                    }
                }
            }

            let idx = ((iy as usize) * (gw as usize) + (ix as usize)) * 4;
            out[idx] = (rgb[2].clamp(0.0, 1.0) * cov * 255.0).round() as u8;
            out[idx + 1] = (rgb[1].clamp(0.0, 1.0) * cov * 255.0).round() as u8;
            out[idx + 2] = (rgb[0].clamp(0.0, 1.0) * cov * 255.0).round() as u8;
            out[idx + 3] = (cov * 255.0).round() as u8;
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn glass_cache_key(
    backdrop_gen: u64,
    polarity: ThemePolarity,
    px_x: i32,
    px_y: i32,
    px_w: u32,
    px_h: u32,
    style: LiquidGlassStyle,
    edge_params: Option<(f32, f32, EdgeSide, bool)>,
) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x1000_0000_01b3);
    };
    mix(backdrop_gen);
    mix(match polarity {
        ThemePolarity::DarkGlass => 1,
        ThemePolarity::LightGlass => 2,
    });
    mix(px_x as u64);
    mix(px_y as u64);
    mix(px_w as u64);
    mix(px_h as u64);
    mix((style.corner_radius * 10.0).round() as u64);
    mix(u64::from(style.hovered));
    mix(u64::from(style.pressed));
    match style.variant {
        LiquidGlassVariant::Regular => mix(10),
        LiquidGlassVariant::Clear => mix(20),
        LiquidGlassVariant::Elevated => mix(30),
        LiquidGlassVariant::Tinted(c) => {
            mix(40);
            mix(pack_color_rgba(c) as u64);
        }
    }
    if let Some((ear_w, r, edge, flares)) = edge_params {
        mix(100);
        mix((ear_w * 10.0).round() as u64);
        mix((r * 10.0).round() as u64);
        mix(match edge {
            EdgeSide::Top => 0,
            EdgeSide::Bottom => 1,
            EdgeSide::Left => 2,
            EdgeSide::Right => 3,
        });
        mix(u64::from(flares));
    }
    h
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
    fn native_surface_uses_layered_per_pixel_alpha_and_renders() {
        let mut surface = match NativeSurface::new("HimeTestSurface", -2000, -2000, 160, 48) {
            Ok(s) => s,
            Err(_) => return,
        };
        let ex_style = unsafe { GetWindowLongW(surface.hwnd(), GWL_EXSTYLE) } as u32;
        assert_ne!(ex_style & WS_EX_TOPMOST.0, 0);
        assert_ne!(ex_style & WS_EX_TOOLWINDOW.0, 0);
        assert_ne!(ex_style & WS_EX_NOACTIVATE.0, 0);
        assert_ne!(ex_style & WS_EX_LAYERED.0, 0);

        surface.set_click_through(true);
        let ct_style = unsafe { GetWindowLongW(surface.hwnd(), GWL_EXSTYLE) } as u32;
        assert_ne!(ct_style & WS_EX_TRANSPARENT.0, 0);

        surface.set_click_through(false);
        let nct_style = unsafe { GetWindowLongW(surface.hwnd(), GWL_EXSTYLE) } as u32;
        assert_eq!(nct_style & WS_EX_TRANSPARENT.0, 0);

        surface
            .set_bounds(-2000, -2000, 540, 200)
            .expect("set_bounds should resize target");
        assert_eq!(surface.size(), (540, 200));

        // Verify true 32-bit premultiplied per-pixel alpha transparency:
        // Corners outside the Liquid Glass panel must stay alpha == 0 (never solid black),
        // while the interior of the drawn glass panel has alpha > 0.
        surface.begin_draw();
        surface
            .draw_liquid_glass_panel(
                Rect::new(40.0, 30.0, 220.0, 100.0),
                LiquidGlassStyle::regular(18.0).without_shadow(),
            )
            .expect("draw_liquid_glass_panel should succeed");
        surface
            .draw_liquid_glass_button(
                Rect::new(280.0, 40.0, 44.0, 44.0),
                12.0,
                Some(theme::SYSTEM_BLUE),
                true,
                false,
            )
            .expect("draw_liquid_glass_button should succeed");
        surface
            .draw_liquid_glass_edge_surface(
                Rect::new(340.0, 0.0, 180.0, 40.0),
                13.0,
                16.0,
                EdgeSide::Top,
                true,
                LiquidGlassStyle::clear(16.0).without_shadow(),
            )
            .expect("draw_liquid_glass_edge_surface should succeed");
        surface
            .end_draw_and_present()
            .expect("layered UpdateLayeredWindow should succeed");

        unsafe {
            let dib = surface.dib.borrow();
            assert_eq!(dib.pixel_bgra(1, 1), Some([0, 0, 0, 0]));
            let center_px = dib.pixel_bgra(120, 80).unwrap();
            assert!(center_px[3] > 0, "expected non-zero glass alpha, got {:?}", center_px);
        }

        let notch = crate::notch::NotchView::default();
        surface.begin_draw();
        notch.render(&surface).expect("NotchView::render should succeed");
        surface
            .end_draw_and_present()
            .expect("notch end_draw_and_present");

        let mut dock = crate::dock::DockBar::new();
        dock.add_item(crate::dock::DockItem::new("term", "Terminal", "wt.exe", true));
        surface.begin_draw();
        dock.render(&surface).expect("DockBar::render should succeed");
        surface
            .end_draw_and_present()
            .expect("dock end_draw_and_present");

        surface.begin_draw();
        surface
            .draw_line(10.0, 10.0, 40.0, 40.0, crate::theme::TEXT_PRIMARY, 1.5)
            .unwrap();
        surface
            .fill_circle(60.0, 24.0, 8.0, crate::theme::SYSTEM_BLUE)
            .unwrap();
        surface
            .stroke_circle(84.0, 24.0, 8.0, crate::theme::TEXT_PRIMARY, 1.2)
            .unwrap();
        surface
            .draw_circle(108.0, 24.0, 8.0, crate::theme::TEXT_PRIMARY, false, 1.0)
            .unwrap();
        surface
            .draw_sun_icon(132.0, 24.0, 14.0, crate::theme::TEXT_PRIMARY)
            .unwrap();
        surface
            .draw_volume_icon(156.0, 24.0, 14.0, false, 0.8, crate::theme::TEXT_PRIMARY)
            .unwrap();
        surface
            .draw_bluetooth_icon(180.0, 24.0, 14.0, crate::theme::TEXT_PRIMARY)
            .unwrap();
        surface
            .draw_calendar_icon(204.0, 24.0, 14.0, crate::theme::TEXT_PRIMARY)
            .unwrap();
        surface
            .draw_text_bold(
                "16:50",
                Rect::new(220.0, 10.0, 60.0, 28.0),
                14.0,
                crate::theme::TEXT_PRIMARY,
                TextAlign::Center,
            )
            .unwrap();
        surface
            .end_draw_and_present()
            .expect("vector icon helpers");
    }

    #[test]
    fn all_sf_symbols_render_cleanly() {
        let surface = match NativeSurface::new("HimeSfSymbolsTest", -2000, -2000, 480, 160) {
            Ok(s) => s,
            Err(_) => return,
        };
        let symbols = [
            SfSymbol::CloseX,
            SfSymbol::ChevronLeft,
            SfSymbol::ChevronRight,
            SfSymbol::ChevronDown,
            SfSymbol::ChevronUp,
            SfSymbol::SettingsSliders,
            SfSymbol::Desktops,
            SfSymbol::TrayGrid,
            SfSymbol::VolumeHigh,
            SfSymbol::VolumeMuted,
            SfSymbol::BrightnessSun,
            SfSymbol::Play,
            SfSymbol::Pause,
            SfSymbol::SkipBack,
            SfSymbol::SkipForward,
            SfSymbol::Plus,
            SfSymbol::Minus,
            SfSymbol::Trash,
            SfSymbol::Check,
            SfSymbol::Wifi,
            SfSymbol::WifiOff,
            SfSymbol::FolderDrop,
            SfSymbol::Search,
            SfSymbol::Sparkle,
            SfSymbol::Pin,
            SfSymbol::Calendar,
            SfSymbol::FocusTimer,
            SfSymbol::ShortcutKeys,
            SfSymbol::Gear,
            SfSymbol::MusicNote,
            SfSymbol::Bell,
            SfSymbol::Bluetooth,
            SfSymbol::Copy,
            SfSymbol::SendArrow,
            SfSymbol::ShieldCheck,
            SfSymbol::ListChecklist,
            SfSymbol::Terminal,
            SfSymbol::Document,
            SfSymbol::Globe,
            SfSymbol::Refresh,
            SfSymbol::StopCircle,
            SfSymbol::PrivateMask,
        ];
        surface.begin_draw();
        for (idx, &sym) in symbols.iter().enumerate() {
            let col = (idx % 12) as f32;
            let row = (idx / 12) as f32;
            surface
                .draw_sf_symbol(
                    Rect::new(10.0 + col * 32.0, 10.0 + row * 32.0, 22.0, 22.0),
                    sym,
                    theme::TEXT_PRIMARY,
                    0.0,
                )
                .expect("SfSymbol should render without error");
        }
        surface.end_draw_and_present().expect("SfSymbol present");
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

        // Intermediate tiny heights during Peek wake/hide keep concave ears cleanly bounded.
        let tiny_ops =
            edge_contour_points(Rect::new(0.0, 0.0, 64.0, 10.0), 13.0, 18.0, EdgeSide::Top, true);
        if let Some(&ContourOp::CubicTo { end: (_, ear_y), .. }) = tiny_ops.get(1) {
            assert!(
                ear_y <= 5.0,
                "concave ear must stay above bottom corner radius at tiny heights, got {ear_y}"
            );
        }
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

        // Empty region marks empty hide state, and non-empty clears it.
        s1.set_rounded_region(Rect::new(0.0, 0.0, 0.0, 0.0), 0.0);
        assert!(*s1.region_is_empty.borrow());
        s1.set_rounded_region(Rect::new(0.0, 0.0, 120.0, 48.0), 18.0);
        assert!(!*s1.region_is_empty.borrow());

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

    #[test]
    fn backdrop_blur_refraction_and_adaptive_polarity() {
        let surface = match NativeSurface::new("HimeBackdropGlassTest", -2000, -2000, 240, 120) {
            Ok(s) => s,
            Err(_) => return,
        };

        // 1. Dark/black backdrop under Auto -> Dark Smoked Glass + crisp white text (Apple HIG)
        let mut dark_bg = vec![0u8; 240 * 120 * 4];
        for px in dark_bg.chunks_exact_mut(4) {
            px[0] = 0x10;
            px[1] = 0x10;
            px[2] = 0x12;
            px[3] = 0xFF;
        }
        surface.set_backdrop_pixels(240, 120, &dark_bg);
        assert_eq!(surface.theme_polarity(), ThemePolarity::DarkGlass);
        assert_eq!(surface.palette().text_primary, theme::TEXT_PRIMARY);

        surface.begin_draw();
        surface
            .draw_liquid_glass_panel(
                Rect::new(20.0, 20.0, 120.0, 60.0),
                LiquidGlassStyle::regular(16.0).without_shadow(),
            )
            .unwrap();
        surface
            .draw_liquid_glass_edge_surface(
                Rect::new(150.0, 0.0, 80.0, 36.0),
                10.0,
                14.0,
                EdgeSide::Top,
                true,
                LiquidGlassStyle::clear(14.0).without_shadow(),
            )
            .unwrap();
        surface.end_draw_and_present().unwrap();

        unsafe {
            let dib = surface.dib.borrow();
            assert_eq!(dib.pixel_bgra(2, 2), Some([0, 0, 0, 0]));
            // Corner pixel inside bounding rect (20,20) but outside r=16 circular corner arc:
            // distance from arc center (36,36) to (21,21) = sqrt(15^2 + 15^2) = 21.21 > 16 => alpha 0.
            assert_eq!(dib.pixel_bgra(21, 21), Some([0, 0, 0, 0]));
            let interior = dib.pixel_bgra(75, 50).unwrap();
            assert!(
                interior[0] < 90 && interior[1] < 90 && interior[2] < 90 && interior[3] > 220,
                "expected translucent dark smoked glass on dark backdrop, got {:?}",
                interior
            );
        }

        // 2. White/bright backdrop under Auto -> Light Frosted White Glass + near-black text (Apple HIG)
        let mut light_bg = vec![0u8; 240 * 120 * 4];
        for px in light_bg.chunks_exact_mut(4) {
            px[0] = 0xF5;
            px[1] = 0xF5;
            px[2] = 0xF8;
            px[3] = 0xFF;
        }
        surface.set_backdrop_pixels(240, 120, &light_bg);
        assert_eq!(surface.theme_polarity(), ThemePolarity::LightGlass);
        assert_eq!(surface.palette().text_primary, theme::LIGHT_TEXT_PRIMARY);

        surface.begin_draw();
        surface
            .draw_liquid_glass_panel(
                Rect::new(20.0, 20.0, 120.0, 60.0),
                LiquidGlassStyle::regular(16.0).without_shadow(),
            )
            .unwrap();
        surface.end_draw_and_present().unwrap();

        unsafe {
            let dib = surface.dib.borrow();
            let interior = dib.pixel_bgra(75, 50).unwrap();
            assert!(
                interior[0] > 180 && interior[1] > 180 && interior[2] > 180 && interior[3] > 220,
                "expected translucent light frosted white glass on bright backdrop, got {:?}",
                interior
            );
        }

        // 3. Explicit Light/Dark mode overrides backdrop luminance
        surface.set_color_theme_mode(ColorThemeMode::Light);
        assert_eq!(surface.theme_polarity(), ThemePolarity::LightGlass);
        surface.set_color_theme_mode(ColorThemeMode::Dark);
        assert_eq!(surface.theme_polarity(), ThemePolarity::DarkGlass);

        // 4. draw_icon_button is frameless at rest (0 pixels) and draws wash on hover/active
        surface.begin_draw();
        surface
            .draw_icon_button(Rect::new(10.0, 10.0, 28.0, 28.0), 8.0, false, false, false)
            .unwrap();
        surface
            .draw_icon_button(Rect::new(50.0, 10.0, 28.0, 28.0), 8.0, true, false, false)
            .unwrap();
        surface.end_draw_and_present().unwrap();
        unsafe {
            let dib = surface.dib.borrow();
            assert_eq!(dib.pixel_bgra(24, 24), Some([0, 0, 0, 0]));
            assert!(dib.pixel_bgra(64, 24).unwrap()[3] > 0);
        }
    }

    #[test]
    fn brightness_sun_rotation_and_windows_shortcut_icon() {
        let surface = match NativeSurface::new("HimeSunTest", -2000, -2000, 160, 64) {
            Ok(s) => s,
            Err(_) => return,
        };
        surface.begin_draw();
        for (i, angle) in [0.0_f32, 0.75, 1.5, std::f32::consts::PI].iter().enumerate() {
            surface
                .draw_brightness_sun(
                    Rect::new(8.0 + (i as f32) * 28.0, 16.0, 22.0, 22.0),
                    *angle,
                    theme::TEXT_PRIMARY,
                    0.0,
                )
                .unwrap();
        }
        surface
            .draw_sf_symbol(
                Rect::new(128.0, 16.0, 22.0, 22.0),
                SfSymbol::ShortcutKeys,
                theme::TEXT_PRIMARY,
                0.0,
            )
            .unwrap();
        surface.end_draw_and_present().unwrap();
    }

    #[test]
    fn wic_png_decode_and_uwp_assets_fallback() {
        fn crc32(data: &[u8]) -> u32 {
            let mut crc = 0xFFFF_FFFF_u32;
            for &b in data {
                crc ^= b as u32;
                for _ in 0..8 {
                    let mask = (crc & 1).wrapping_neg();
                    crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
                }
            }
            !crc
        }
        fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            let start = out.len();
            out.extend_from_slice(kind);
            out.extend_from_slice(data);
            let crc = crc32(&out[start..]);
            out.extend_from_slice(&crc.to_be_bytes());
        }
        fn adler32(data: &[u8]) -> u32 {
            let (mut s1, mut s2) = (1_u32, 0_u32);
            for &b in data {
                s1 = (s1 + b as u32) % 65521;
                s2 = (s2 + s1) % 65521;
            }
            (s2 << 16) | s1
        }

        // Valid 2x2 RGBA PNG (blue/white pixels with full alpha)
        let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10];
        let ihdr = [
            0, 0, 0, 2, // width = 2
            0, 0, 0, 2, // height = 2
            8, 6, 0, 0, 0, // 8-bit RGBA
        ];
        push_chunk(&mut png, b"IHDR", &ihdr);
        let raw_rows: [u8; 18] = [
            0, 10, 132, 255, 255, 255, 255, 255, 255, // row 0
            0, 255, 255, 255, 255, 10, 132, 255, 255, // row 1
        ];
        let mut idat = vec![0x78, 0x01, 0x01, 18, 0, !18_u8, 0xFF];
        idat.extend_from_slice(&raw_rows);
        idat.extend_from_slice(&adler32(&raw_rows).to_be_bytes());
        push_chunk(&mut png, b"IDAT", &idat);
        push_chunk(&mut png, b"IEND", &[]);

        let tmp_dir = std::env::temp_dir().join("hime_uwp_icon_test");
        let assets_dir = tmp_dir.join("Assets");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&assets_dir).unwrap();
        let png_path = assets_dir.join("CalculatorSquare44x44Logo.altform-unplated_targetsize-48.png");
        std::fs::write(&png_path, &png).unwrap();
        let fake_exe = tmp_dir.join("CalculatorApp.exe");
        std::fs::write(&fake_exe, b"").unwrap();

        let extracted = extract_window_or_exe_icon(0, fake_exe.to_str().unwrap(), 48)
            .expect("UWP Assets PNG fallback should decode via WIC");
        assert_eq!(extracted.width, 48);
        assert_eq!(extracted.height, 48);
        assert_eq!(extracted.bgra_premul.len(), 48 * 48 * 4);
        assert!(extracted.bgra_premul.chunks_exact(4).any(|px| px[3] > 200));

        // Verify WIC decoding also works on a COINIT_MULTITHREADED (MTA) thread.
        let exe_str = fake_exe.to_string_lossy().to_string();
        let mta_extracted = std::thread::spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED);
            }
            extract_window_or_exe_icon(0, &exe_str, 48)
        })
        .join()
        .unwrap()
        .expect("UWP Assets PNG decode should succeed under MTA");
        assert_eq!(mta_extracted.width, 48);

        let mem_decoded = decode_image_bytes_bgra_premul(&png, 20, 20)
            .expect("in-memory PNG bytes should decode via WIC stream");
        assert_eq!(mem_decoded.width, 20);
        assert_eq!(mem_decoded.height, 20);
        assert!(mem_decoded.bgra_premul.chunks_exact(4).any(|px| px[3] > 200));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn edge_surface_flared_shadow_and_contour_match() {
        let surface = match NativeSurface::new("HimeEdgeShadowTest", -2000, -2000, 120, 240) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Verify signed distance matches the 10/13 concave flare Bezier for Left & Right.
        let ear = 13.0_f32;
        let rect = Rect::new(24.0, 24.0, 56.0, 180.0);
        // Canonical (ux = ear * 0.72, uy = ear * 0.25) is strictly OUTSIDE the 10/13 Bezier flare.
        let (d_out_right, _, _) = sdg_edge_contour(
            rect.width - ear * 0.25,
            ear * 0.72,
            rect.width,
            rect.height,
            ear,
            18.0,
            EdgeSide::Right,
            true,
        );
        assert!(
            d_out_right > 0.5,
            "concave Right flare notch should be outside contour, got d={d_out_right}"
        );
        let (d_out_left, _, _) = sdg_edge_contour(
            ear * 0.25,
            ear * 0.72,
            rect.width,
            rect.height,
            ear,
            18.0,
            EdgeSide::Left,
            true,
        );
        assert!(
            d_out_left > 0.5,
            "concave Left flare notch should be outside contour, got d={d_out_left}"
        );

        let w = 120_u32;
        let h = 240_u32;
        let bgra = vec![96_u8; (w * h * 4) as usize];
        surface.set_backdrop_pixels(w, h, &bgra);
        surface.begin_draw();
        surface
            .draw_liquid_glass_edge_surface(
                rect,
                ear,
                18.0,
                EdgeSide::Right,
                true,
                LiquidGlassStyle::regular(18.0),
            )
            .expect("draw Right edge surface");
        surface.end_draw_and_present().expect("end_draw_and_present");

        let alpha_at = |x: u32, y: u32| -> u8 {
            unsafe { surface.dib.borrow().pixel_bgra(x, y).unwrap()[3] }
        };

        // Right screen-bezel edge stays flush: zero shadow/glass bleeding past rect.x + rect.width.
        assert_eq!(alpha_at(82, 110), 0);
        assert_eq!(alpha_at(82, 24), 0);
        assert_eq!(alpha_at(82, 204), 0);

        // Above/below the top & bottom body corners (outside the concave flares), no hard
        // rectangular/rounded-corner shadow bump should protrude.
        assert!(
            alpha_at(30, 18) <= 30,
            "top-left corner above flare alpha={}",
            alpha_at(30, 18)
        );
        assert!(
            alpha_at(30, 210) <= 30,
            "bottom-left corner below flare alpha={}",
            alpha_at(30, 210)
        );
        // Inside the main glass body, rendered opaque glass is solid/translucent (>150).
        assert!(
            alpha_at(52, 114) > 150,
            "glass center alpha={}",
            alpha_at(52, 114)
        );

        // Repeat bezel-flush check for EdgeSide::Left flared surface.
        let left_rect = Rect::new(24.0, 24.0, 56.0, 180.0);
        surface.begin_draw();
        surface
            .draw_liquid_glass_edge_surface(
                left_rect,
                ear,
                18.0,
                EdgeSide::Left,
                true,
                LiquidGlassStyle::regular(18.0),
            )
            .expect("draw Left edge surface");
        surface.end_draw_and_present().expect("end_draw_and_present");
        assert_eq!(alpha_at(22, 110), 0);
        assert!(
            alpha_at(74, 18) <= 30,
            "Left top-right above flare alpha={}",
            alpha_at(74, 18)
        );
        assert!(
            alpha_at(74, 210) <= 30,
            "Left bottom-right below flare alpha={}",
            alpha_at(74, 210)
        );
        assert!(
            alpha_at(52, 114) > 150,
            "Left glass center alpha={}",
            alpha_at(52, 114)
        );
    }
}
