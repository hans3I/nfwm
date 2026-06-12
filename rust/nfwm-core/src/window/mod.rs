//! Window registry and classification for runtime window tracking.
//!
//! The registry is the single source of truth for discovered windows and
//! their tiling state. It is pure logic that lives in `nfwm-core` and
//! is testable with `FakeWindowProvider`.

pub mod classifier;
pub mod discovery;
pub mod registry;

pub use classifier::*;
pub use discovery::*;
pub use registry::*;
