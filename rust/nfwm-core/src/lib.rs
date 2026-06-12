//! nfwm-core: Pure tiling logic, settings model, and state transitions.
//!
//! This crate contains no OS dependencies and can be tested entirely without
//! real Windows windows or displays.

pub mod layout;
pub mod commands;
pub mod settings;

pub use layout::*;
pub use commands::*;
pub use settings::*;
