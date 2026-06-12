//! Virtual desktop manager: track Windows virtual desktops.
//!
//! This uses the Win32 desktop API (GetThreadDesktop/GetUserObjectInformation)
//! rather than the undocumented COM IVirtualDesktopManager interfaces.
//!
//! Each virtual desktop gets a stable ID based on its desktop name.

use nfwm_core::traits::DesktopError;
use nfwm_core::traits::VirtualDesktopProvider;
use nfwm_core::types::{VirtualDesktopId, WindowId};
use std::collections::HashMap;
use std::sync::Mutex;

// Raw Win32 imports that may not be in the `windows` crate features.
// SAFETY: These are standard Win32 APIs.
extern "system" {
    fn GetThreadDesktop(dwthreadid: u32) -> windows::Win32::Foundation::HANDLE;
    fn GetUserObjectInformationW(
        hobj: windows::Win32::Foundation::HANDLE,
        nindex: i32,
        pvinfo: *mut std::ffi::c_void,
        nlength: u32,
        lpnlengthneeded: *mut u32,
    ) -> windows::Win32::Foundation::BOOL;
}

const UOI_NAME: i32 = 2;

/// A virtual desktop manager that tracks the current virtual desktop.
///
/// All `unsafe` Win32 calls are isolated here. See `docs/coding-standards.md`.
pub struct Win32VirtualDesktopManager {
    /// Cache of known desktop names to stable IDs.
    cache: Mutex<HashMap<String, VirtualDesktopId>>,
    next_id: Mutex<usize>,
}

impl Default for Win32VirtualDesktopManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Win32VirtualDesktopManager {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Get the name of the current virtual desktop.
    ///
    /// This uses GetUserObjectInformation on the current thread's desktop.
    fn current_desktop_name(&self) -> Option<String> {
        let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
        // SAFETY: GetThreadDesktop is safe for any valid thread ID
        let hdesk = unsafe { GetThreadDesktop(thread_id) };
        if hdesk.is_invalid() {
            return None;
        }

        // Query desktop name using GetUserObjectInformation
        let mut buffer = [0u16; 256];
        let mut needed = 0u32;
        // SAFETY: hdesk is valid; buffer is large enough; pvinfo is a valid pointer
        let result = unsafe {
            GetUserObjectInformationW(
                hdesk,
                UOI_NAME,
                buffer.as_mut_ptr() as *mut _,
                (buffer.len() * 2) as u32,
                &mut needed,
            )
        };
        if result.as_bool() {
            let len = needed as usize / 2;
            let name = String::from_utf16_lossy(&buffer[..len]);
            // Remove trailing null
            Some(name.trim_end_matches('\0').to_string())
        } else {
            None
        }
    }

    /// Get a stable VirtualDesktopId for a desktop name.
    fn id_for_name(&self, name: &str) -> VirtualDesktopId {
        let mut cache = self.cache.lock().unwrap();
        if let Some(&id) = cache.get(name) {
            return id;
        }
        let mut next = self.next_id.lock().unwrap();
        let id = VirtualDesktopId(*next);
        *next += 1;
        cache.insert(name.to_string(), id);
        id
    }
}

impl VirtualDesktopProvider for Win32VirtualDesktopManager {
    fn current_desktop(&self) -> VirtualDesktopId {
        self.current_desktop_name()
            .map(|name| self.id_for_name(&name))
            .unwrap_or_else(|| VirtualDesktopId(0))
    }

    fn all_desktops(&self) -> Vec<VirtualDesktopId> {
        // We only know the current desktop. In a real implementation,
        // we'd enumerate all desktops via EnumDesktops or COM.
        // For Phase 08, we return just the current desktop.
        vec![self.current_desktop()]
    }

    fn move_window(
        &self,
        _window: WindowId,
        _desktop: VirtualDesktopId,
    ) -> Result<(), DesktopError> {
        // Moving windows between desktops requires COM IVirtualDesktopManager.
        // Not implemented in Phase 08.
        Err(DesktopError::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vdm_current_desktop() {
        let vdm = Win32VirtualDesktopManager::new();
        let _id = vdm.current_desktop();
        // Should not panic; ID is u32 and always >= 0
    }

    #[test]
    fn vdm_all_desktops() {
        let vdm = Win32VirtualDesktopManager::new();
        let all = vdm.all_desktops();
        assert!(!all.is_empty());
    }

    #[test]
    fn vdm_move_window_not_supported() {
        let vdm = Win32VirtualDesktopManager::new();
        let result = vdm.move_window(WindowId(1), VirtualDesktopId(1));
        assert_eq!(result, Err(DesktopError::NotSupported));
    }
}
