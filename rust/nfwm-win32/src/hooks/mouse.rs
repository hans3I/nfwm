//! Mouse hook: global low-level mouse hook.
//!
//! The hook runs on a dedicated thread with a message loop. Dropping the handle
//! uninstalls the hook and joins the thread.

use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use tracing::info;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, MSLLHOOKSTRUCT, WH_MOUSE_LL,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
};

/// A global low-level mouse hook.
pub struct MouseHook {
    hook_id: HHOOK,
    thread: Option<JoinHandle<()>>,
}

/// An event from the mouse hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEvent {
    /// Mouse moved.
    Move { x: i32, y: i32 },
    /// Left button pressed.
    LeftButtonDown { x: i32, y: i32 },
    /// Left button released.
    LeftButtonUp { x: i32, y: i32 },
    /// Right button pressed.
    RightButtonDown { x: i32, y: i32 },
    /// Right button released.
    RightButtonUp { x: i32, y: i32 },
    /// Mouse wheel scrolled.
    Wheel { delta: i16 },
}

/// Handle for receiving mouse events from the hook.
pub struct MouseHookHandle {
    _hook: MouseHook,
}

impl MouseHook {
    /// Install a global low-level mouse hook.
    pub fn install(
        tx: Sender<MouseEvent>,
    ) -> Result<MouseHookHandle, crate::hooks::keyboard::HookError> {
        let (setup_tx, setup_rx) =
            std::sync::mpsc::channel::<Result<HHOOK, crate::hooks::keyboard::HookError>>();

        let thread = thread::spawn(move || {
            let hook_id = unsafe {
                // SAFETY: SetWindowsHookExW is a well-defined API. HINSTANCE(0) is valid for
                // global hooks (WH_KEYBOARD_LL / WH_MOUSE_LL) because they are not injected
                // into other processes.
                SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), HINSTANCE(0), 0)
            };

            let hook_id = match hook_id {
                Ok(id) => id,
                Err(_) => {
                    let _ = setup_tx.send(Err(crate::hooks::keyboard::HookError::InstallFailed));
                    return;
                }
            };

            MOUSE_HOOK_STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.hook_id = Some(hook_id);
                state.sender = Some(tx);
            });

            let _ = setup_tx.send(Ok(hook_id));

            info!("Mouse hook message loop started");
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

                if MOUSE_HOOK_STATE.with(|state| state.borrow().should_exit) {
                    break;
                }
            }
            info!("Mouse hook message loop exiting");
        });

        let hook_id = setup_rx
            .recv()
            .map_err(|_| crate::hooks::keyboard::HookError::InstallFailed)??;

        let hook = MouseHook {
            hook_id,
            thread: Some(thread),
        };

        Ok(MouseHookHandle { _hook: hook })
    }
}

impl Drop for MouseHook {
    fn drop(&mut self) {
        MOUSE_HOOK_STATE.with(|state| {
            state.borrow_mut().should_exit = true;
        });

        unsafe {
            if let Some(thread) = self.thread.take() {
                use std::os::windows::io::AsRawHandle;
                let handle = thread.as_raw_handle() as isize;
                let thread_id = windows::Win32::System::Threading::GetThreadId(
                    windows::Win32::Foundation::HANDLE(handle),
                );
                // SAFETY: PostThreadMessageW is a well-defined API
                let _ = windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                    thread_id,
                    windows::Win32::UI::WindowsAndMessaging::WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }

        if self.hook_id.0 != 0 {
            // SAFETY: hook_id is valid
            let _ = unsafe { UnhookWindowsHookEx(self.hook_id) };
            info!("Mouse hook uninstalled");
        }

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

thread_local! {
    static MOUSE_HOOK_STATE: std::cell::RefCell<MouseHookState> = std::cell::RefCell::new(MouseHookState::default());
}

#[derive(Default)]
struct MouseHookState {
    hook_id: Option<HHOOK>,
    sender: Option<Sender<MouseEvent>>,
    should_exit: bool,
}

// SAFETY: This is the low-level mouse hook procedure. LPARAM is a pointer to MSLLHOOKSTRUCT.
unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        // SAFETY: code < 0 means we must pass through without processing
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
    let x = info.pt.x;
    let y = info.pt.y;

    let event = match wparam.0 as u32 {
        WM_MOUSEMOVE => Some(MouseEvent::Move { x, y }),
        WM_LBUTTONDOWN => Some(MouseEvent::LeftButtonDown { x, y }),
        WM_LBUTTONUP => Some(MouseEvent::LeftButtonUp { x, y }),
        WM_RBUTTONDOWN => Some(MouseEvent::RightButtonDown { x, y }),
        WM_RBUTTONUP => Some(MouseEvent::RightButtonUp { x, y }),
        WM_MOUSEWHEEL => {
            // The high word of mouseData contains the wheel delta
            let delta = ((info.mouseData >> 16) & 0xFFFF) as i16;
            Some(MouseEvent::Wheel { delta })
        }
        _ => None,
    };

    if let Some(event) = event {
        MOUSE_HOOK_STATE.with(|state| {
            if let Some(sender) = state.borrow().sender.as_ref() {
                let _ = sender.send(event);
            }
        });
    }

    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_event_debug() {
        let ev = MouseEvent::Move { x: 10, y: 20 };
        assert_eq!(format!("{:?}", ev), "Move { x: 10, y: 20 }");
    }
}
