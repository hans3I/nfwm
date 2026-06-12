//! nfwm-win32: Win32 abstractions for window, display, desktop, and hook APIs.
//!
//! All `unsafe` Win32 code is isolated in this crate. Core logic in `nfwm-core`
//! should never call Win32 APIs directly.

pub mod window;
pub mod display;
pub mod hooks;
pub mod ipc;

pub use window::*;
pub use display::*;
pub use hooks::*;
pub use ipc::*;
