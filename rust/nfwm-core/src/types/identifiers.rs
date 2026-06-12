//! Stable identifiers for windows, displays, and virtual desktops.
//!
//! These wrappers prevent raw handles from leaking into core logic.

/// A stable identifier for a native window.
///
/// This is a wrapper around a raw HWND (usize) to provide type safety
/// and prevent accidental mixing with other handle types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WindowId(pub usize);

/// A stable identifier for a display/monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DisplayId(pub usize);

/// A stable identifier for a virtual desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VirtualDesktopId(pub usize);

/// A stable identifier for a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ProcessId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_equality() {
        let a = WindowId(42);
        let b = WindowId(42);
        let c = WindowId(43);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn display_id_equality() {
        let a = DisplayId(1);
        let b = DisplayId(1);
        let c = DisplayId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
