//! Window ID: stable wrapper around HWND.

use nfwm_core::types::WindowId;

/// Convert a WindowId to a raw HWND.
pub fn id_to_hwnd(id: WindowId) -> windows::Win32::Foundation::HWND {
    windows::Win32::Foundation::HWND(id.0 as isize)
}

/// Convert a raw HWND to a WindowId.
pub fn hwnd_to_id(hwnd: windows::Win32::Foundation::HWND) -> WindowId {
    WindowId(hwnd.0 as usize)
}
