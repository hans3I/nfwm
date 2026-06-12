//! nfwm-ui: Tray, settings, overlays, toasts, and user interaction.
//!
//! UI state is kept separate from tiling state. This crate depends on
//! `nfwm-core` for data models and `nfwm-win32` for OS integration.

pub mod overlay;
pub mod settings_ui;
pub mod toast;
pub mod tray;

pub use overlay::*;
pub use settings_ui::*;
pub use toast::*;
pub use tray::*;
