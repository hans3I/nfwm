//! CopyData: IPC via WM_COPYDATA.
//!
//! This module implements a client that can send action names to a running
//! nfwm instance, and a server that receives them. The server integration
//! lives in the app layer; this module provides the core primitives.
//!
//! The protocol is: `COPYDATASTRUCT` with `dwData = 0` and `lpData` pointing
//! to a UTF-8 encoded action name string.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use thiserror::Error;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_COPYDATA};

#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(lpclassname: *const u16, lpwindowname: *const u16) -> HWND;
}

/// Error from CopyData IPC operations.
#[derive(Debug, Error)]
pub enum CopyDataError {
    #[error("Target window not found")]
    TargetNotFound,
    #[error("SendMessage failed")]
    SendFailed,
    #[error("Invalid action name")]
    InvalidAction,
}

/// Client for sending action names to a running nfwm instance.
pub struct CopyDataClient {
    target_class: Vec<u16>,
    target_title: Vec<u16>,
}

impl CopyDataClient {
    /// Create a new client targeting the main nfwm window.
    ///
    /// The target window class and title should match the hidden main window
    /// created by the app (e.g. `FancyWMMainWindow` in the legacy app).
    pub fn new(target_class: &str, target_title: &str) -> Self {
        Self {
            target_class: to_wide(target_class),
            target_title: to_wide(target_title),
        }
    }

    /// Send an action name to the running instance.
    ///
    /// Returns `Ok(())` if the message was sent successfully.
    pub fn send_action(&self, action: &str) -> Result<(), CopyDataError> {
        if action.is_empty() {
            return Err(CopyDataError::InvalidAction);
        }

        // SAFETY: both strings are null-terminated wide strings that live for the call.
        let hwnd = unsafe { FindWindowW(self.target_class.as_ptr(), self.target_title.as_ptr()) };

        if hwnd.0 == 0 {
            return Err(CopyDataError::TargetNotFound);
        }

        // Encode action as UTF-8 bytes
        let bytes = action.as_bytes();
        let cds = COPYDATASTRUCT {
            dwData: 0,
            cbData: bytes.len() as u32,
            lpData: bytes.as_ptr() as *mut _,
        };

        // SAFETY: SendMessageW is a well-defined API; hwnd is valid, cds lives for the call
        let result = unsafe {
            SendMessageW(
                hwnd,
                WM_COPYDATA,
                WPARAM(0),
                LPARAM(&cds as *const _ as isize),
            )
        };

        if result.0 != 0 {
            Ok(())
        } else {
            Err(CopyDataError::SendFailed)
        }
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

/// Decode a WM_COPYDATA message into an action name.
///
/// This is intended to be called in the window procedure of the receiving
/// application. The `lparam` is the LPARAM from the `WM_COPYDATA` message.
///
/// # Safety
///
/// `lparam` must be a valid pointer to a `COPYDATASTRUCT` that was sent by
/// the `CopyDataClient` or an equivalent sender.
pub unsafe fn decode_copydata_action(lparam: LPARAM) -> Option<String> {
    let cds = &*(lparam.0 as *const COPYDATASTRUCT);
    if cds.dwData != 0 {
        return None;
    }
    let bytes = std::slice::from_raw_parts(cds.lpData as *const u8, cds.cbData as usize);
    String::from_utf8(bytes.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copydata_error_display() {
        let err = CopyDataError::TargetNotFound;
        assert_eq!(format!("{}", err), "Target window not found");
    }

    #[test]
    fn copydata_client_invalid_action() {
        let client = CopyDataClient::new("nfwm", "nfwm");
        assert!(client.send_action("").is_err());
    }

    #[test]
    fn decode_copydata_action_roundtrip() {
        let payload = b"Toggle";
        let cds = COPYDATASTRUCT {
            dwData: 0,
            cbData: payload.len() as u32,
            lpData: payload.as_ptr() as *mut _,
        };

        // SAFETY: cds points at valid test data for the duration of the call.
        let decoded = unsafe { decode_copydata_action(LPARAM(&cds as *const _ as isize)) };
        assert_eq!(decoded.as_deref(), Some("Toggle"));
    }
}
