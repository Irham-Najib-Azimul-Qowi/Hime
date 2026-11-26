//! Direct2D + DirectWrite + DirectComposition window & rendering surface.

use std::cell::RefCell;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use windows::core::{s, w, HRESULT, PCWSTR, Result};
use windows_numerics::Vector2 as D2D_POINT_2F;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_IGNORE, D2D1_BEZIER_SEGMENT, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
    D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D1_PIXEL_FORMAT,
    D2D_RECT_F, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1PathGeometry,
    ID2D1SolidColorBrush, ID2D1StrokeStyle, D2D1_CAP_STYLE_ROUND, D2D1_ELLIPSE,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
    D2D1_ROUNDED_RECT, D2D1_STROKE_STYLE_PROPERTIES1,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_SEMI_BOLD,
    DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn, ValidateRect};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongW, PostMessageW, RegisterClassExW,
    SetWindowLongW, SetWindowPos, ShowWindow, CS_HREDRAW, CS_VREDRAW, GWL_EXSTYLE, HWND_TOPMOST,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE,
    SW_SHOWNOACTIVATE, WM_APP, WM_DROPFILES, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT,
    WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
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

/// Hardware-accelerated borderless topmost window backed by
/// Direct2D `ID2D1HwndRenderTarget` + DirectWrite + `SetWindowRgn` rounded clipping.
pub struct NativeSurface {
    hwnd: HWND,
    width: u32,
    height: u32,
    dpi: f32,
    d2d_factory: ID2D1Factory1,
    dwrite_factory: IDWriteFactory,
    d2d_target: ID2D1HwndRenderTarget,
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

            let d2d_factory: ID2D1Factory1 =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
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
        unsafe {
            let rgn = CreateRoundRectRgn(left, top, right, bottom, diam, diam);
            if !rgn.is_invalid() {
                // On success, the system owns `rgn` and deletes it automatically.
                let _ = SetWindowRgn(self.hwnd, Some(rgn), true);
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

    /// Draws UTF-8 text inside `rect` using DirectWrite.
    pub fn draw_text(
        &self,
        text: &str,
        rect: Rect,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> Result<()> {
        unsafe {
            let brush = self.create_brush(color)?;
            let format = self.create_text_format(font_size, align)?;
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
            let stroke_props = D2D1_STROKE_STYLE_PROPERTIES1 {
                startCap: D2D1_CAP_STYLE_ROUND,
                endCap: D2D1_CAP_STYLE_ROUND,
                ..Default::default()
            };
            let round_cap: ID2D1StrokeStyle =
                self.d2d_factory.CreateStrokeStyle(&stroke_props, None)?.into();

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

    unsafe fn create_brush(&self, color: Color) -> Result<ID2D1SolidColorBrush> {
        let d2d_color: D2D1_COLOR_F = color.to_d2d();
        self.d2d_target.CreateSolidColorBrush(&d2d_color, None)
    }

    unsafe fn create_text_format(
        &self,
        font_size: f32,
        align: TextAlign,
    ) -> Result<IDWriteTextFormat> {
        let family = to_wide_null(FONT_FAMILY);
        let locale = to_wide_null("en-US");
        let format = self.dwrite_factory.CreateTextFormat(
            PCWSTR(family.as_ptr()),
            None,
            DWRITE_FONT_WEIGHT_SEMI_BOLD,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            font_size.max(1.0),
            PCWSTR(locale.as_ptr()),
        )?;
        let dwrite_align = match align {
            TextAlign::Leading => DWRITE_TEXT_ALIGNMENT_LEADING,
            TextAlign::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
            TextAlign::Trailing => DWRITE_TEXT_ALIGNMENT_TRAILING,
        };
        format.SetTextAlignment(dwrite_align)?;
        format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
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
        unsafe {
            if !self.hwnd.0.is_null() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

fn register_window_class(hinstance: HINSTANCE) {
    CLASS_REGISTERED.get_or_init(|| unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(surface_wnd_proc),
            hInstance: hinstance,
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
    }
}
