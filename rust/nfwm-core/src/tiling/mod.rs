//! Tiling service and workspace: application-facing operations.

pub mod error;
pub mod multi_display;
pub mod service;
pub mod workspace;

pub use error::*;
pub use multi_display::*;
pub use service::*;
pub use workspace::*;
