//! Hotkey engine: tracks key state and matches keybindings to actions.
//!
//! The engine supports two modes:
//!
//! 1. **Direct mode**: keybindings trigger immediately when pressed.
//! 2. **Activation mode**: an activation keybinding (e.g. `CapsLock`) must be
//!    pressed first. While active, action keybindings trigger. Releasing the
//!    activation key exits the mode.
//!
//! This mirrors the legacy behavior where direct hotkeys and command-sequence
//! mode are both supported.

use crate::commands::Action;
use crate::input::keybinding::{Key, Modifiers};
use crate::input::Keybinding;
use std::collections::HashMap;

/// The operating mode of the hotkey engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationMode {
    /// Direct hotkeys: no activation key required.
    Direct,
    /// Hold activation key: action keybindings only trigger while the
    /// activation key is held.
    Hold,
    /// Toggle activation key: pressing the activation key toggles sequence
    /// mode on/off.
    Toggle,
}

/// State machine for the hotkey engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EngineState {
    /// No activation key is active.
    Idle,
    /// Activation key is held (Hold mode).
    Active,
    /// Activation key was toggled on (Toggle mode).
    Sequence,
}

/// Engine that tracks pressed keys and emits actions when keybindings match.
pub struct HotkeyEngine {
    /// Map from keybindings to actions.
    bindings: HashMap<Keybinding, Action>,
    /// Currently pressed keys.
    pressed_keys: Vec<Key>,
    /// Currently pressed modifiers.
    pressed_modifiers: Modifiers,
    /// Activation keybinding, if any.
    activation_key: Option<Keybinding>,
    /// Current activation mode.
    mode: ActivationMode,
    /// Current state machine state.
    state: EngineState,
}

impl HotkeyEngine {
    /// Create a new engine with the given bindings.
    pub fn new(bindings: HashMap<Keybinding, Action>) -> Self {
        Self {
            bindings,
            pressed_keys: Vec::new(),
            pressed_modifiers: Modifiers::default(),
            activation_key: None,
            mode: ActivationMode::Direct,
            state: EngineState::Idle,
        }
    }

    /// Set the activation key and mode.
    ///
    /// If `key` is `None`, the engine operates in direct mode.
    pub fn set_activation(&mut self, key: Option<Keybinding>, mode: ActivationMode) {
        self.activation_key = key;
        self.mode = mode;
        self.state = EngineState::Idle;
    }

    /// Get the current activation mode.
    pub fn activation_mode(&self) -> ActivationMode {
        self.mode
    }

    /// Get the current engine state.
    pub fn state(&self) -> &'static str {
        match self.state {
            EngineState::Idle => "idle",
            EngineState::Active => "active",
            EngineState::Sequence => "sequence",
        }
    }

    /// Register a key press.
    ///
    /// Returns the action to dispatch, if any.
    pub fn on_key_down(&mut self, key: Key, modifiers: Modifiers) -> Option<Action> {
        self.update_modifiers(modifiers);
        if !self.pressed_keys.contains(&key) {
            self.pressed_keys.push(key);
        }

        // Check if this is the activation key
        if let Some(ref activation) = self.activation_key {
            if activation.matches(key, &modifiers) {
                match self.mode {
                    ActivationMode::Hold => {
                        self.state = EngineState::Active;
                        return None;
                    }
                    ActivationMode::Toggle => {
                        self.state = if self.state == EngineState::Sequence {
                            EngineState::Idle
                        } else {
                            EngineState::Sequence
                        };
                        return None;
                    }
                    ActivationMode::Direct => {
                        // In direct mode, activation key is just another binding
                    }
                }
            }
        }

        // Check if we should dispatch an action
        if self.can_dispatch() {
            self.dispatch(key, modifiers)
        } else {
            None
        }
    }

    /// Register a key release.
    ///
    /// Returns the action to dispatch, if any (e.g. for key-up triggers).
    pub fn on_key_up(&mut self, key: Key, modifiers: Modifiers) -> Option<Action> {
        self.update_modifiers(modifiers);
        self.pressed_keys.retain(|&k| k != key);

        // Check if activation key was released in Hold mode
        if let Some(ref activation) = self.activation_key {
            if activation.matches(key, &modifiers) && self.mode == ActivationMode::Hold {
                self.state = EngineState::Idle;
            }
        }

        None
    }

    /// Check if the engine is currently in an active dispatch state.
    fn can_dispatch(&self) -> bool {
        match self.mode {
            ActivationMode::Direct => true,
            ActivationMode::Hold => self.state == EngineState::Active,
            ActivationMode::Toggle => self.state == EngineState::Sequence,
        }
    }

    /// Find a matching binding for the given key and modifiers.
    fn dispatch(&self, key: Key, modifiers: Modifiers) -> Option<Action> {
        for (binding, action) in &self.bindings {
            if binding.matches(key, &modifiers) {
                return Some(*action);
            }
        }
        None
    }

    /// Update tracked modifiers.
    fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.pressed_modifiers = modifiers;
    }

    /// Clear all pressed state (e.g. on window focus loss).
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.pressed_modifiers = Modifiers::default();
        if self.mode == ActivationMode::Hold {
            self.state = EngineState::Idle;
        }
    }
}

/// A dictionary of keybindings to actions, supporting defaults.
#[derive(Debug, Clone, Default)]
pub struct KeybindingDictionary {
    bindings: HashMap<Keybinding, Action>,
}

impl KeybindingDictionary {
    /// Create an empty dictionary.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Create a dictionary with default keybindings.
    pub fn with_defaults() -> Self {
        let mut dict = Self::new();
        use crate::commands::Action;
        use crate::types::Direction;

        let defaults = [
            ("Shift+H", Action::SplitHorizontal),
            ("Shift+V", Action::SplitVertical),
            ("Shift+S", Action::Stack),
            ("Shift+F", Action::Float),
            ("Shift+Left", Action::MoveFocus(Direction::Left)),
            ("Shift+Right", Action::MoveFocus(Direction::Right)),
            ("Shift+Up", Action::MoveFocus(Direction::Up)),
            ("Shift+Down", Action::MoveFocus(Direction::Down)),
            ("Shift+Ctrl+Left", Action::Swap(Direction::Left)),
            ("Shift+Ctrl+Right", Action::Swap(Direction::Right)),
            ("Shift+Ctrl+Up", Action::Swap(Direction::Up)),
            ("Shift+Ctrl+Down", Action::Swap(Direction::Down)),
            ("Shift+Alt+Left", Action::MoveWindow(Direction::Left)),
            ("Shift+Alt+Right", Action::MoveWindow(Direction::Right)),
            ("Shift+Alt+Up", Action::MoveWindow(Direction::Up)),
            ("Shift+Alt+Down", Action::MoveWindow(Direction::Down)),
            ("Shift+U", Action::PullUp),
            ("Shift+Minus", Action::Resize(Direction::Left)),
            ("Shift+Plus", Action::Resize(Direction::Right)),
        ];

        for (s, action) in defaults {
            if let Ok(kb) = s.parse::<Keybinding>() {
                dict.bindings.insert(kb, action);
            }
        }
        dict
    }

    /// Insert a binding.
    pub fn insert(&mut self, binding: Keybinding, action: Action) {
        self.bindings.insert(binding, action);
    }

    /// Remove a binding.
    pub fn remove(&mut self, binding: &Keybinding) {
        self.bindings.remove(binding);
    }

    /// Look up an action by binding.
    pub fn get(&self, binding: &Keybinding) -> Option<&Action> {
        self.bindings.get(binding)
    }

    /// Get all bindings.
    pub fn bindings(&self) -> &HashMap<Keybinding, Action> {
        &self.bindings
    }

    /// Convert to a `HashMap` for the engine.
    pub fn into_engine(self) -> HotkeyEngine {
        HotkeyEngine::new(self.bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Action;

    #[test]
    fn engine_direct_mode() {
        let mut bindings = HashMap::new();
        bindings.insert(
            "Shift+H".parse::<Keybinding>().unwrap(),
            Action::SplitHorizontal,
        );
        let mut engine = HotkeyEngine::new(bindings);
        engine.set_activation(None, ActivationMode::Direct);

        let result = engine.on_key_down(
            Key::Char('H'),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(result, Some(Action::SplitHorizontal));
    }

    #[test]
    fn engine_hold_mode() {
        let mut bindings = HashMap::new();
        bindings.insert(
            "Shift+H".parse::<Keybinding>().unwrap(),
            Action::SplitHorizontal,
        );
        let mut engine = HotkeyEngine::new(bindings);
        let activation = "CapsLock".parse::<Keybinding>().unwrap();
        engine.set_activation(Some(activation), ActivationMode::Hold);

        // Without activation, nothing happens
        let result = engine.on_key_down(
            Key::Char('H'),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(result, None);

        // Press activation key
        engine.on_key_down(Key::CapsLock, Modifiers::default());
        assert_eq!(engine.state(), "active");

        // Now action works
        let result = engine.on_key_down(
            Key::Char('H'),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(result, Some(Action::SplitHorizontal));

        // Release activation key
        engine.on_key_up(Key::CapsLock, Modifiers::default());
        assert_eq!(engine.state(), "idle");
    }

    #[test]
    fn engine_toggle_mode() {
        let mut bindings = HashMap::new();
        bindings.insert(
            "Shift+H".parse::<Keybinding>().unwrap(),
            Action::SplitHorizontal,
        );
        let mut engine = HotkeyEngine::new(bindings);
        let activation = "CapsLock".parse::<Keybinding>().unwrap();
        engine.set_activation(Some(activation), ActivationMode::Toggle);

        // Toggle on
        engine.on_key_down(Key::CapsLock, Modifiers::default());
        assert_eq!(engine.state(), "sequence");

        // Action works
        let result = engine.on_key_down(
            Key::Char('H'),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(result, Some(Action::SplitHorizontal));

        // Toggle off
        engine.on_key_down(Key::CapsLock, Modifiers::default());
        assert_eq!(engine.state(), "idle");
    }

    #[test]
    fn engine_no_match() {
        let mut bindings = HashMap::new();
        bindings.insert(
            "Shift+H".parse::<Keybinding>().unwrap(),
            Action::SplitHorizontal,
        );
        let mut engine = HotkeyEngine::new(bindings);
        engine.set_activation(None, ActivationMode::Direct);

        let result = engine.on_key_down(
            Key::Char('V'),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(result, None);
    }

    #[test]
    fn dictionary_defaults() {
        let dict = KeybindingDictionary::with_defaults();
        assert!(
            dict.get(&"Shift+H".parse().unwrap()).is_some(),
            "Shift+H should be bound"
        );
        assert!(
            dict.get(&"Shift+Ctrl+Left".parse().unwrap()).is_some(),
            "Shift+Ctrl+Left should be bound"
        );
    }

    #[test]
    fn dictionary_into_engine() {
        let dict = KeybindingDictionary::with_defaults();
        let mut engine = dict.into_engine();
        engine.set_activation(None, ActivationMode::Direct);

        let result = engine.on_key_down(
            Key::Char('H'),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(result, Some(Action::SplitHorizontal));
    }

    #[test]
    fn engine_clear() {
        let mut engine = KeybindingDictionary::with_defaults().into_engine();
        engine.set_activation(Some("CapsLock".parse().unwrap()), ActivationMode::Hold);
        engine.on_key_down(Key::CapsLock, Modifiers::default());
        assert_eq!(engine.state(), "active");
        engine.clear();
        assert_eq!(engine.state(), "idle");
    }
}
