//! Panel: split and stack panel containers.

/// A panel that contains other nodes.
pub trait PanelNode {}

/// A split panel divides space among children.
pub struct SplitPanelNode;

/// A stack panel shows one child at a time.
pub struct StackPanelNode;
