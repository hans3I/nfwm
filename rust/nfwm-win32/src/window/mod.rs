//! Window abstractions: HWND wrapper, properties, classification.

pub mod manager;
pub mod properties;
pub mod window_id;

pub use manager::*;
pub use properties::*;
pub use window_id::*;
