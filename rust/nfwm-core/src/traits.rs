//! Traits for OS integration so pure logic can be tested without real windows.
//!
//! Core logic depends on these traits, not on concrete Win32 implementations.
//! This allows tests to use fake/mock providers.

use crate::types::*;

/// Provides information about native windows.
///
/// Implementations live in `nfwm-win32`. Core logic uses this trait
/// to query window properties without depending on Win32 directly.
pub trait WindowProvider {
    /// Get the title of a window.
    fn title(&self, id: WindowId) -> Option<String>;

    /// Get the class name of a window.
    fn class_name(&self, id: WindowId) -> Option<String>;

    /// Get the process ID owning a window.
    fn process_id(&self, id: WindowId) -> Option<ProcessId>;

    /// Get the current bounds of a window.
    fn bounds(&self, id: WindowId) -> Option<Rectangle>;

    /// Check if a window is currently visible.
    fn is_visible(&self, id: WindowId) -> bool;

    /// Check if a window is minimized.
    fn is_minimized(&self, id: WindowId) -> bool;

    /// Check if a window is maximized.
    fn is_maximized(&self, id: WindowId) -> bool;

    /// Check if a window is topmost.
    fn is_topmost(&self, id: WindowId) -> bool;

    /// Check if a window is resizable.
    fn is_resizable(&self, id: WindowId) -> bool;

    /// Get the minimum size of a window.
    fn min_size(&self, id: WindowId) -> Option<Size>;

    /// Get the display a window is currently on.
    fn display_id(&self, id: WindowId) -> Option<DisplayId>;

    /// Get the virtual desktop a window is currently on.
    fn virtual_desktop_id(&self, id: WindowId) -> Option<VirtualDesktopId>;

    /// Check if this window is the foreground window.
    fn is_focused(&self, id: WindowId) -> bool;
}

/// Provides information about displays/monitors.
///
/// Implementations live in `nfwm-win32`. Core logic uses this trait
/// to query display properties without depending on Win32 directly.
pub trait DisplayProvider {
    /// Get all connected displays.
    fn all_displays(&self) -> Vec<DisplayId>;

    /// Get the primary display.
    fn primary_display(&self) -> DisplayId;

    /// Get the bounds of a display.
    fn bounds(&self, id: DisplayId) -> Option<Rectangle>;

    /// Get the work area of a display (excluding taskbars).
    fn work_area(&self, id: DisplayId) -> Option<Rectangle>;

    /// Get the DPI of a display.
    fn dpi(&self, id: DisplayId) -> Option<f32>;

    /// Get the refresh rate of a display.
    fn refresh_rate(&self, id: DisplayId) -> Option<f32>;
}

/// Provides virtual desktop operations.
///
/// Implementations live in `nfwm-win32`. Core logic uses this trait
/// to query virtual desktop state without depending on Win32 directly.
pub trait VirtualDesktopProvider {
    /// Get the current virtual desktop.
    fn current_desktop(&self) -> VirtualDesktopId;

    /// Get all virtual desktops.
    fn all_desktops(&self) -> Vec<VirtualDesktopId>;

    /// Move a window to a virtual desktop.
    fn move_window(&self, window: WindowId, desktop: VirtualDesktopId) -> Result<(), DesktopError>;
}

/// Error for virtual desktop operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DesktopError {
    #[error("Desktop not found")]
    NotFound,
    #[error("Window not found")]
    WindowNotFound,
    #[error("Operation not supported")]
    NotSupported,
}

/// Provides window placement operations.
///
/// Implementations live in `nfwm-win32`. Core logic uses this trait
/// to move/resize windows without depending on Win32 directly.
pub trait PlacementProvider {
    /// Move and resize a window to the given rectangle.
    fn set_bounds(&self, id: WindowId, rect: Rectangle) -> Result<(), PlacementError>;

    /// Restore a window to its normal state.
    fn restore(&self, id: WindowId) -> Result<(), PlacementError>;

    /// Minimize a window.
    fn minimize(&self, id: WindowId) -> Result<(), PlacementError>;
}

/// Error for placement operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlacementError {
    #[error("Window not found")]
    WindowNotFound,
    #[error("Access denied")]
    AccessDenied,
    #[error("Invalid bounds")]
    InvalidBounds,
}

/// A fake window provider for testing core logic without real windows.
#[cfg(test)]
pub mod fake {
    use super::*;
    use std::collections::HashMap;

    /// A fake window for testing.
    #[derive(Debug, Clone, Default)]
    pub struct FakeWindow {
        pub title: String,
        pub class_name: String,
        pub process_id: ProcessId,
        pub bounds: Rectangle,
        pub visible: bool,
        pub minimized: bool,
        pub maximized: bool,
        pub topmost: bool,
        pub resizable: bool,
        pub min_size: Option<Size>,
        pub display_id: DisplayId,
        pub virtual_desktop_id: VirtualDesktopId,
        pub focused: bool,
    }

    /// A fake window provider backed by a HashMap.
    pub struct FakeWindowProvider {
        pub windows: HashMap<WindowId, FakeWindow>,
        pub focused: WindowId,
    }

    impl FakeWindowProvider {
        pub fn new() -> Self {
            Self {
                windows: HashMap::new(),
                focused: WindowId::default(),
            }
        }

        pub fn add(&mut self, id: WindowId, window: FakeWindow) {
            self.windows.insert(id, window);
        }
    }

    impl WindowProvider for FakeWindowProvider {
        fn title(&self, id: WindowId) -> Option<String> {
            self.windows.get(&id).map(|w| w.title.clone())
        }

        fn class_name(&self, id: WindowId) -> Option<String> {
            self.windows.get(&id).map(|w| w.class_name.clone())
        }

        fn process_id(&self, id: WindowId) -> Option<ProcessId> {
            self.windows.get(&id).map(|w| w.process_id)
        }

        fn bounds(&self, id: WindowId) -> Option<Rectangle> {
            self.windows.get(&id).map(|w| w.bounds)
        }

        fn is_visible(&self, id: WindowId) -> bool {
            self.windows.get(&id).map(|w| w.visible).unwrap_or(false)
        }

        fn is_minimized(&self, id: WindowId) -> bool {
            self.windows.get(&id).map(|w| w.minimized).unwrap_or(false)
        }

        fn is_maximized(&self, id: WindowId) -> bool {
            self.windows.get(&id).map(|w| w.maximized).unwrap_or(false)
        }

        fn is_topmost(&self, id: WindowId) -> bool {
            self.windows.get(&id).map(|w| w.topmost).unwrap_or(false)
        }

        fn is_resizable(&self, id: WindowId) -> bool {
            self.windows.get(&id).map(|w| w.resizable).unwrap_or(false)
        }

        fn min_size(&self, id: WindowId) -> Option<Size> {
            self.windows.get(&id).and_then(|w| w.min_size)
        }

        fn display_id(&self, id: WindowId) -> Option<DisplayId> {
            self.windows.get(&id).map(|w| w.display_id)
        }

        fn virtual_desktop_id(&self, id: WindowId) -> Option<VirtualDesktopId> {
            self.windows.get(&id).map(|w| w.virtual_desktop_id)
        }

        fn is_focused(&self, id: WindowId) -> bool {
            self.focused == id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::*;
    use super::*;

    #[test]
    fn fake_window_provider() {
        let mut provider = FakeWindowProvider::new();
        let window_id = WindowId(1);
        let window = FakeWindow {
            title: "Test Window".to_string(),
            class_name: "TestClass".to_string(),
            process_id: ProcessId(1234),
            bounds: Rectangle::new(0, 0, 800, 600),
            visible: true,
            minimized: false,
            maximized: false,
            topmost: false,
            resizable: true,
            min_size: Some(Size::new(200, 100)),
            display_id: DisplayId(0),
            virtual_desktop_id: VirtualDesktopId(0),
            focused: false,
        };
        provider.add(window_id, window);
        provider.focused = window_id;

        assert_eq!(provider.title(window_id), Some("Test Window".to_string()));
        assert_eq!(provider.process_id(window_id), Some(ProcessId(1234)));
        assert!(provider.is_visible(window_id));
        assert!(provider.is_focused(window_id));
        assert!(!provider.is_minimized(window_id));
    }

    #[test]
    fn desktop_error_display() {
        let err = DesktopError::NotFound;
        assert_eq!(format!("{}", err), "Desktop not found");
    }

    #[test]
    fn placement_error_display() {
        let err = PlacementError::AccessDenied;
        assert_eq!(format!("{}", err), "Access denied");
    }
}
