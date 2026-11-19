#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        }
    }
}

pub struct PalWindowHelper;

impl PalWindowHelper {
    /// Returns default geometry for the Dynamic Island floating HUD bar.
    pub fn default_dynamic_island_geometry() -> WindowGeometry {
        WindowGeometry {
            x: 0,
            y: 16,
            width: 480,
            height: 48,
        }
    }
}
