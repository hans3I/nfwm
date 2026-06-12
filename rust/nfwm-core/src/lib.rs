//! nfwm-core: Pure tiling logic, settings model, and state transitions.
//!
//! This crate contains no OS dependencies and can be tested entirely without
//! real Windows windows or displays.

pub mod commands;
pub mod layout;
pub mod settings;
pub mod traits;
pub mod types;

pub use commands::*;
pub use layout::*;
pub use settings::*;
pub use traits::*;
pub use types::*;
