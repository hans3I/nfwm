use crate::commands::Action;

/// A pending tiling operation that is waiting for user confirmation
/// or a target selection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Intent {
    /// Waiting for a direction to complete a split.
    Split { action: Action },
    /// Waiting for a target window to complete a move.
    Move { action: Action },
    /// Waiting for a target window to complete a swap.
    Swap { action: Action },
    /// Waiting for a resize direction.
    Resize { action: Action },
    /// No pending intent.
    #[default]
    None,
}

impl Intent {
    pub fn is_pending(&self) -> bool {
        !matches!(self, Intent::None)
    }

    pub fn clear(&mut self) {
        *self = Intent::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Action;
    use crate::types::Direction;

    #[test]
    fn intent_pending() {
        let intent = Intent::Split {
            action: Action::SplitHorizontal,
        };
        assert!(intent.is_pending());
    }

    #[test]
    fn intent_not_pending() {
        let intent = Intent::None;
        assert!(!intent.is_pending());
    }

    #[test]
    fn intent_clear() {
        let mut intent = Intent::Move {
            action: Action::MoveWindow(Direction::Left),
        };
        assert!(intent.is_pending());
        intent.clear();
        assert!(!intent.is_pending());
    }
}
