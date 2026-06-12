//! Keyboard hook: global low-level keyboard hook.

use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use tracing::info;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// A global low-level keyboard hook.
///
/// The hook runs on a dedicated thread with a message loop. Dropping the handle
/// uninstalls the hook and joins the thread.
///
/// # Safety
///
/// Hooks must be installed and removed on the same thread. We enforce this by
/// spawning a dedicated thread for the hook and joining it on drop.
pub struct KeyboardHook {
    hook_id: HHOOK,
    thread: Option<JoinHandle<()>>,
}

/// An event from the keyboard hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardEvent {
    KeyDown { vk_code: u32, scan_code: u32 },
    KeyUp { vk_code: u32, scan_code: u32 },
    SysKeyDown { vk_code: u32, scan_code: u32 },
    SysKeyUp { vk_code: u32, scan_code: u32 },
}

/// Handle for receiving keyboard events from the hook.
pub struct KeyboardHookHandle {
    _hook: KeyboardHook,
}

impl KeyboardHook {
    /// Install a global low-level keyboard hook.
    ///
    /// Returns a handle that will receive events via the provided channel.
    /// The hook thread is started immediately.
    pub fn install(tx: Sender<KeyboardEvent>) -> Result<KeyboardHookHandle, HookError> {
        let (setup_tx, setup_rx) = std::sync::mpsc::channel::<Result<HHOOK, HookError>>();

        let thread = thread::spawn(move || {
            // Install the hook on this thread
            let hook_id = unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), HINSTANCE(0), 0)
            };

            let hook_id = match hook_id {
                Ok(id) => id,
                Err(_) => {
                    let _ = setup_tx.send(Err(HookError::InstallFailed));
                    return;
                }
            };

            // Store the hook ID and sender in thread-local storage
            HOOK_STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.hook_id = Some(hook_id);
                state.sender = Some(tx);
            });

            let _ = setup_tx.send(Ok(hook_id));

            // Message loop
            info!("Keyboard hook message loop started");
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            loop {
                // SAFETY: GetMessageW is a well-defined API
                let result = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0)
                };
                if result.0 == 0 || result.0 == -1 {
                    break;
                }
                // SAFETY: TranslateMessage and DispatchMessageW are well-defined
                unsafe {
                    windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                }

                // Check if we should exit
                if HOOK_STATE.with(|state| state.borrow().should_exit) {
                    break;
                }
            }
            info!("Keyboard hook message loop exiting");
        });

        // Wait for the hook to be installed
        let hook_id = setup_rx.recv().map_err(|_| HookError::InstallFailed)??;

        let hook = KeyboardHook {
            hook_id,
            thread: Some(thread),
        };

        Ok(KeyboardHookHandle { _hook: hook })
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        // Signal the thread to exit
        HOOK_STATE.with(|state| {
            state.borrow_mut().should_exit = true;
        });

        // Post a WM_NULL message to wake up GetMessageW
        // SAFETY: PostThreadMessageW is a well-defined API
        unsafe {
            if let Some(thread) = self.thread.take() {
                use std::os::windows::io::AsRawHandle;
                let handle = thread.as_raw_handle() as isize;
                let thread_id = windows::Win32::System::Threading::GetThreadId(
                    windows::Win32::Foundation::HANDLE(handle),
                );
                let _ = windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                    thread_id,
                    windows::Win32::UI::WindowsAndMessaging::WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }

        // Unhook
        if self.hook_id.0 != 0 {
            // SAFETY: hook_id is valid
            let _ = unsafe { UnhookWindowsHookEx(self.hook_id) };
            info!("Keyboard hook uninstalled");
        }

        // Join the thread
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Error installing a hook.
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Failed to install hook")]
    InstallFailed,
    #[error("Hook thread panicked")]
    ThreadPanicked,
}

// Thread-local state for the hook procedure
thread_local! {
    static HOOK_STATE: std::cell::RefCell<HookState> = std::cell::RefCell::new(HookState::default());
}

#[derive(Default)]
struct HookState {
    hook_id: Option<HHOOK>,
    sender: Option<Sender<KeyboardEvent>>,
    should_exit: bool,
}

// SAFETY: This is the low-level keyboard hook procedure. LPARAM is a pointer to KBDLLHOOKSTRUCT.
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        // SAFETY: code < 0 means we must pass through without processing
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

    let event = match wparam.0 as u32 {
        WM_KEYDOWN => KeyboardEvent::KeyDown {
            vk_code: info.vkCode,
            scan_code: info.scanCode,
        },
        WM_KEYUP => KeyboardEvent::KeyUp {
            vk_code: info.vkCode,
            scan_code: info.scanCode,
        },
        WM_SYSKEYDOWN => KeyboardEvent::SysKeyDown {
            vk_code: info.vkCode,
            scan_code: info.scanCode,
        },
        WM_SYSKEYUP => KeyboardEvent::SysKeyUp {
            vk_code: info.vkCode,
            scan_code: info.scanCode,
        },
        _ => {
            return CallNextHookEx(None, code, wparam, lparam);
        }
    };

    HOOK_STATE.with(|state| {
        if let Some(sender) = state.borrow().sender.as_ref() {
            let _ = sender.send(event);
        }
    });

    // Pass through to the next hook
    CallNextHookEx(None, code, wparam, lparam)
}

/// A no-op hook for testing that doesn't actually install a real hook.
pub struct NullHook;

impl Default for NullHook {
    fn default() -> Self {
        Self::new()
    }
}

impl NullHook {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_error_display() {
        let err = HookError::InstallFailed;
        assert_eq!(format!("{}", err), "Failed to install hook");
    }

    #[test]
    fn keyboard_event_debug() {
        let event = KeyboardEvent::KeyDown {
            vk_code: 0x41,
            scan_code: 0x1E,
        };
        assert_eq!(
            format!("{:?}", event),
            "KeyDown { vk_code: 65, scan_code: 30 }"
        );
    }
}
