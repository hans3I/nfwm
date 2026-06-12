//! Monitor: display bounds, work area, DPI.

use nfwm_core::types::Rectangle;

/// A native monitor with its properties.
#[derive(Debug, Clone)]
pub struct Monitor {
    pub id: usize,
    pub bounds: Rectangle,
    pub work_area: Rectangle,
    pub dpi: f32,
    pub refresh_rate: f32,
    pub is_primary: bool,
}
