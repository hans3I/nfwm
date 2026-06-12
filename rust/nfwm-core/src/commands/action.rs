use crate::types::*;

/// A user-bindable tiling action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Split the focused window horizontally.
    SplitHorizontal,
    /// Split the focused window vertically.
    SplitVertical,
    /// Stack the focused window with its neighbor.
    Stack,
    /// Float the focused window.
    Float,
    /// Move focus in a direction.
    MoveFocus(Direction),
    /// Swap the focused window with its neighbor.
    Swap(Direction),
    /// Move the focused window in a direction.
    MoveWindow(Direction),
    /// Pull the focused window up (promote).
    PullUp,
    /// Resize the focused panel.
    Resize(Direction),
    /// Start tiling on the current desktop.
    Start,
    /// Stop tiling on the current desktop.
    Stop,
    /// Discover new windows.
    Discover,
    /// Refresh the current layout.
    Refresh,
    /// Toggle tiling on the current desktop.
    Toggle,
}

/// Default keybindings for nfwm actions.
///
/// These mirror the legacy default bindings where applicable.
pub mod defaults {
    use super::Action;
    use crate::types::Direction;

    /// Get the default keybinding string for an action.
    pub fn keybinding(action: Action) -> Option<&'static str> {
        match action {
            Action::SplitHorizontal => Some("Shift+H"),
            Action::SplitVertical => Some("Shift+V"),
            Action::Stack => Some("Shift+S"),
            Action::Float => Some("Shift+F"),
            Action::MoveFocus(Direction::Left) => Some("Shift+Left"),
            Action::MoveFocus(Direction::Right) => Some("Shift+Right"),
            Action::MoveFocus(Direction::Up) => Some("Shift+Up"),
            Action::MoveFocus(Direction::Down) => Some("Shift+Down"),
            Action::Swap(Direction::Left) => Some("Shift+Ctrl+Left"),
            Action::Swap(Direction::Right) => Some("Shift+Ctrl+Right"),
            Action::Swap(Direction::Up) => Some("Shift+Ctrl+Up"),
            Action::Swap(Direction::Down) => Some("Shift+Ctrl+Down"),
            Action::MoveWindow(Direction::Left) => Some("Shift+Alt+Left"),
            Action::MoveWindow(Direction::Right) => Some("Shift+Alt+Right"),
            Action::MoveWindow(Direction::Up) => Some("Shift+Alt+Up"),
            Action::MoveWindow(Direction::Down) => Some("Shift+Alt+Down"),
            Action::PullUp => Some("Shift+U"),
            Action::Resize(Direction::Left) => Some("Shift+Minus"),
            Action::Resize(Direction::Right) => Some("Shift+Plus"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::defaults::*;
    use super::*;

    #[test]
    fn default_keybinding_exists() {
        assert_eq!(keybinding(Action::SplitHorizontal), Some("Shift+H"));
        assert_eq!(keybinding(Action::SplitVertical), Some("Shift+V"));
    }

    #[test]
    fn default_keybinding_none() {
        assert_eq!(keybinding(Action::Start), None);
        assert_eq!(keybinding(Action::Stop), None);
    }
}
