//! Modifier-assisted window moving.
//!
//! When a configured modifier combination is held, mouse movement events
//! translate into window move operations. This is pure logic that receives
//! mouse events from the platform hook and computes window position deltas.

use crate::input::keybinding::Modifiers;
use crate::types::{Point, WindowId};

/// An event from the modifier-assisted window mover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMoveEvent {
    /// Started moving a window.
    Start { window_id: WindowId },
    /// Moving the window by a delta.
    Move {
        window_id: WindowId,
        delta_x: i32,
        delta_y: i32,
    },
    /// Finished moving.
    End { window_id: WindowId },
}

/// Tracks modifier-assisted window moving.
///
/// The mover requires a modifier combination (e.g. `Shift+Alt`) and
/// a window provider to resolve the focused window. When the modifiers
/// are held and the mouse moves, it emits move events.
pub struct ModifierWindowMover {
    /// Modifiers that must be held to trigger window moving.
    required_modifiers: Modifiers,
    /// Whether the required modifiers are currently pressed.
    modifiers_active: bool,
    /// Current mouse position.
    mouse_pos: Option<Point>,
    /// Whether a move is currently in progress.
    is_moving: bool,
    /// The window currently being moved.
    window_id: Option<WindowId>,
}

impl ModifierWindowMover {
    /// Create a new mover with the required modifiers.
    pub fn new(required_modifiers: Modifiers) -> Self {
        Self {
            required_modifiers,
            modifiers_active: false,
            mouse_pos: None,
            is_moving: false,
            window_id: None,
        }
    }

    /// Update the current modifier state.
    pub fn update_modifiers(&mut self, modifiers: Modifiers) {
        let was_active = self.modifiers_active;
        self.modifiers_active = self.required_modifiers.is_satisfied_by(&modifiers);

        if !self.modifiers_active && was_active {
            // Modifiers released: end move
            if self.window_id.is_some() {
                self.is_moving = false;
                self.window_id = None;
                self.mouse_pos = None;
            }
        }
    }

    /// Handle a mouse move event.
    ///
    /// If the required modifiers are held and a window is being tracked,
    /// returns a `WindowMoveEvent::Move` with the delta.
    ///
    /// `window_id` should be the window under the cursor or the focused window.
    pub fn on_mouse_move(
        &mut self,
        x: i32,
        y: i32,
        window_id: WindowId,
    ) -> Option<WindowMoveEvent> {
        if !self.modifiers_active {
            return None;
        }

        let new_pos = Point::new(x, y);

        if !self.is_moving {
            // Start a new move
            self.is_moving = true;
            self.window_id = Some(window_id);
            self.mouse_pos = Some(new_pos);
            return Some(WindowMoveEvent::Start { window_id });
        }

        if self.window_id != Some(window_id) {
            // Window changed: end old, start new
            let old_window = self.window_id;
            self.is_moving = true;
            self.window_id = Some(window_id);
            self.mouse_pos = Some(new_pos);
            // Return end for the old window if we had one
            return old_window.map(|w| WindowMoveEvent::End { window_id: w });
        }

        let old_pos = self.mouse_pos.unwrap_or(new_pos);
        let delta_x = new_pos.x - old_pos.x;
        let delta_y = new_pos.y - old_pos.y;

        if delta_x == 0 && delta_y == 0 {
            return None;
        }

        self.mouse_pos = Some(new_pos);
        Some(WindowMoveEvent::Move {
            window_id,
            delta_x,
            delta_y,
        })
    }

    /// Handle a mouse button release.
    ///
    /// Ends the current move if one is in progress.
    pub fn on_mouse_up(&mut self) -> Option<WindowMoveEvent> {
        if self.is_moving {
            let window_id = self.window_id?;
            self.is_moving = false;
            self.window_id = None;
            self.mouse_pos = None;
            Some(WindowMoveEvent::End { window_id })
        } else {
            None
        }
    }

    /// Set the required modifiers.
    pub fn set_required_modifiers(&mut self, modifiers: Modifiers) {
        self.required_modifiers = modifiers;
    }

    /// Check if a move is currently in progress.
    pub fn is_moving(&self) -> bool {
        self.is_moving
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::keybinding::Modifiers;
    use crate::WindowId;

    #[test]
    fn mover_starts_when_modifiers_held() {
        let mut mover = ModifierWindowMover::new(Modifiers {
            shift: true,
            alt: true,
            ..Default::default()
        });

        mover.update_modifiers(Modifiers {
            shift: true,
            alt: true,
            ..Default::default()
        });

        let ev = mover.on_mouse_move(100, 100, WindowId(1));
        assert!(
            matches!(ev, Some(WindowMoveEvent::Start { window_id }) if window_id == WindowId(1))
        );
    }

    #[test]
    fn mover_emits_delta() {
        let mut mover = ModifierWindowMover::new(Modifiers {
            shift: true,
            ..Default::default()
        });

        mover.update_modifiers(Modifiers {
            shift: true,
            ..Default::default()
        });

        mover.on_mouse_move(100, 100, WindowId(1));
        let ev = mover.on_mouse_move(110, 120, WindowId(1));
        assert!(
            matches!(ev, Some(WindowMoveEvent::Move { window_id, delta_x, delta_y }) if window_id == WindowId(1) && delta_x == 10 && delta_y == 20)
        );
    }

    #[test]
    fn mover_ignores_without_modifiers() {
        let mut mover = ModifierWindowMover::new(Modifiers {
            shift: true,
            ..Default::default()
        });

        // No modifiers held
        mover.update_modifiers(Modifiers::default());
        let ev = mover.on_mouse_move(100, 100, WindowId(1));
        assert_eq!(ev, None);
    }

    #[test]
    fn mover_ends_on_mouse_up() {
        let mut mover = ModifierWindowMover::new(Modifiers {
            shift: true,
            ..Default::default()
        });

        mover.update_modifiers(Modifiers {
            shift: true,
            ..Default::default()
        });
        mover.on_mouse_move(100, 100, WindowId(1));

        let ev = mover.on_mouse_up();
        assert!(matches!(ev, Some(WindowMoveEvent::End { window_id }) if window_id == WindowId(1)));
        assert!(!mover.is_moving());
    }

    #[test]
    fn mover_ends_on_modifier_release() {
        let mut mover = ModifierWindowMover::new(Modifiers {
            shift: true,
            ..Default::default()
        });

        mover.update_modifiers(Modifiers {
            shift: true,
            ..Default::default()
        });
        mover.on_mouse_move(100, 100, WindowId(1));
        assert!(mover.is_moving());

        // Release modifier
        mover.update_modifiers(Modifiers::default());
        assert!(!mover.is_moving());
    }
}
