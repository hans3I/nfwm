//! Window properties: title, class, process, styles, bounds.

use nfwm_core::types::WindowId;

/// A snapshot of window properties for classification.
#[derive(Debug, Clone)]
pub struct WindowProperties {
    pub id: WindowId,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub bounds: nfwm_core::types::Rectangle,
    pub visible: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub topmost: bool,
    pub resizable: bool,
}

impl WindowProperties {
    /// Determine if this window should be floated (excluded from tiling).
    pub fn should_float(&self) -> bool {
        if self.minimized || self.maximized {
            return true;
        }
        if self.topmost {
            return true;
        }
        if !self.resizable {
            return true;
        }
        // TODO: Add exclusion matcher for process/class in Phase 05
        false
    }

    /// Determine if this window should be completely ignored.
    pub fn should_ignore(&self) -> bool {
        if self.title.is_empty() && self.class_name.is_empty() {
            return true;
        }
        // TODO: Add process/class ignore lists in Phase 05
        false
    }
}
