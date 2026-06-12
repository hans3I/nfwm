//! Action: bindable user actions.

/// A user-bindable tiling action.
pub enum Action {
    SplitHorizontal,
    SplitVertical,
    Stack,
    Float,
    MoveFocusLeft,
    MoveFocusRight,
    MoveFocusUp,
    MoveFocusDown,
    SwapLeft,
    SwapRight,
    SwapUp,
    SwapDown,
    MoveWindowLeft,
    MoveWindowRight,
    MoveWindowUp,
    MoveWindowDown,
    PullUp,
    Resize,
    Start,
    Stop,
    Discover,
    Refresh,
}
