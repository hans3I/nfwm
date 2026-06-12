use crate::types::*;
use std::fmt;
use std::str::FromStr;

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

impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Action::SplitHorizontal => "split-horizontal",
            Action::SplitVertical => "split-vertical",
            Action::Stack => "stack",
            Action::Float => "float",
            Action::MoveFocus(Direction::Left) => "move-focus-left",
            Action::MoveFocus(Direction::Right) => "move-focus-right",
            Action::MoveFocus(Direction::Up) => "move-focus-up",
            Action::MoveFocus(Direction::Down) => "move-focus-down",
            Action::Swap(Direction::Left) => "swap-left",
            Action::Swap(Direction::Right) => "swap-right",
            Action::Swap(Direction::Up) => "swap-up",
            Action::Swap(Direction::Down) => "swap-down",
            Action::MoveWindow(Direction::Left) => "move-window-left",
            Action::MoveWindow(Direction::Right) => "move-window-right",
            Action::MoveWindow(Direction::Up) => "move-window-up",
            Action::MoveWindow(Direction::Down) => "move-window-down",
            Action::PullUp => "pull-up",
            Action::Resize(Direction::Left) => "resize-left",
            Action::Resize(Direction::Right) => "resize-right",
            Action::Resize(Direction::Up) => "resize-up",
            Action::Resize(Direction::Down) => "resize-down",
            Action::Start => "start",
            Action::Stop => "stop",
            Action::Discover => "discover",
            Action::Refresh => "refresh",
            Action::Toggle => "toggle",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown action: {0}")]
pub struct ParseActionError(pub String);

impl FromStr for Action {
    type Err = ParseActionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "split-horizontal" => Ok(Action::SplitHorizontal),
            "split-vertical" => Ok(Action::SplitVertical),
            "stack" => Ok(Action::Stack),
            "float" => Ok(Action::Float),
            "move-focus-left" => Ok(Action::MoveFocus(Direction::Left)),
            "move-focus-right" => Ok(Action::MoveFocus(Direction::Right)),
            "move-focus-up" => Ok(Action::MoveFocus(Direction::Up)),
            "move-focus-down" => Ok(Action::MoveFocus(Direction::Down)),
            "swap-left" => Ok(Action::Swap(Direction::Left)),
            "swap-right" => Ok(Action::Swap(Direction::Right)),
            "swap-up" => Ok(Action::Swap(Direction::Up)),
            "swap-down" => Ok(Action::Swap(Direction::Down)),
            "move-window-left" => Ok(Action::MoveWindow(Direction::Left)),
            "move-window-right" => Ok(Action::MoveWindow(Direction::Right)),
            "move-window-up" => Ok(Action::MoveWindow(Direction::Up)),
            "move-window-down" => Ok(Action::MoveWindow(Direction::Down)),
            "pull-up" => Ok(Action::PullUp),
            "resize-left" => Ok(Action::Resize(Direction::Left)),
            "resize-right" => Ok(Action::Resize(Direction::Right)),
            "resize-up" => Ok(Action::Resize(Direction::Up)),
            "resize-down" => Ok(Action::Resize(Direction::Down)),
            "start" => Ok(Action::Start),
            "stop" => Ok(Action::Stop),
            "discover" => Ok(Action::Discover),
            "refresh" => Ok(Action::Refresh),
            "toggle" => Ok(Action::Toggle),
            _ => Err(ParseActionError(value.to_string())),
        }
    }
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

    #[test]
    fn action_name_roundtrip() {
        let action = Action::MoveWindow(Direction::Left);
        assert_eq!(action.name(), "move-window-left");
        assert_eq!("move-window-left".parse::<Action>().unwrap(), action);
    }

    #[test]
    fn parse_invalid_action() {
        assert!("explode".parse::<Action>().is_err());
    }
}
