//! Keybinding parsing and serialization.
//!
//! Keybindings are strings like `"Shift+H"`, `"Shift+Ctrl+Left"`, or `"CapsLock"`.
//! They consist of zero or more modifiers (`Shift`, `Ctrl`, `Alt`) followed by a key name.

use std::fmt;
use std::str::FromStr;

/// A key on the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A single character key (A-Z, 0-9).
    Char(char),
    /// Arrow keys.
    Left,
    Right,
    Up,
    Down,
    /// Plus / minus keys.
    Plus,
    Minus,
    /// Win key.
    Win,
    /// Modifier keys used as the primary key in activation chords.
    Shift,
    Ctrl,
    Alt,
    /// Punctuation keys used by legacy bindings.
    OpenBracket,
    CloseBracket,
    Semicolon,
    Quote,
    /// Tab.
    Tab,
    /// Escape.
    Escape,
    /// Space.
    Space,
    /// Enter / Return.
    Enter,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Function keys F1-F24.
    F(u8),
    /// Caps Lock.
    CapsLock,
}

/// Modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    /// Shift is pressed.
    pub shift: bool,
    /// Ctrl is pressed.
    pub ctrl: bool,
    /// Alt is pressed.
    pub alt: bool,
    /// Caps Lock is active.
    pub caps_lock: bool,
}

impl Modifiers {
    /// Check if any modifier is active.
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.caps_lock
    }

    /// Check if these modifiers are satisfied by another set.
    ///
    /// A binding's modifiers are satisfied if the currently pressed modifiers
    /// contain at least the binding's modifiers.
    pub fn is_satisfied_by(&self, pressed: &Modifiers) -> bool {
        (!self.shift || pressed.shift)
            && (!self.ctrl || pressed.ctrl)
            && (!self.alt || pressed.alt)
            && (!self.caps_lock || pressed.caps_lock)
    }
}

/// A keybinding: modifiers plus a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keybinding {
    /// Modifiers that must be held.
    pub modifiers: Modifiers,
    /// The key that must be pressed.
    pub key: Key,
}

impl Keybinding {
    /// Create a new keybinding.
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Check if this keybinding matches the current key and modifiers.
    ///
    /// The key must match exactly, and all binding modifiers must be present
    /// in the pressed modifiers.
    pub fn matches(&self, key: Key, pressed: &Modifiers) -> bool {
        self.key == key && self.modifiers.is_satisfied_by(pressed)
    }
}

impl fmt::Display for Keybinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.caps_lock {
            parts.push("CapsLock");
        }
        let key_str = format!("{}", self.key);
        parts.push(&key_str);
        write!(f, "{}", parts.join("+"))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Char(c) => write!(f, "{}", c),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::Plus => write!(f, "Plus"),
            Key::Minus => write!(f, "Minus"),
            Key::Win => write!(f, "Win"),
            Key::Shift => write!(f, "Shift"),
            Key::Ctrl => write!(f, "Ctrl"),
            Key::Alt => write!(f, "Alt"),
            Key::OpenBracket => write!(f, "OpenBracket"),
            Key::CloseBracket => write!(f, "CloseBracket"),
            Key::Semicolon => write!(f, "Semicolon"),
            Key::Quote => write!(f, "Quote"),
            Key::Tab => write!(f, "Tab"),
            Key::Escape => write!(f, "Escape"),
            Key::Space => write!(f, "Space"),
            Key::Enter => write!(f, "Enter"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::F(n) => write!(f, "F{}", n),
            Key::CapsLock => write!(f, "CapsLock"),
        }
    }
}

/// Error parsing a keybinding string.
#[derive(Debug, thiserror::Error)]
#[error("Invalid keybinding: {0}")]
pub struct KeybindingParseError(String);

impl FromStr for Keybinding {
    type Err = KeybindingParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err(KeybindingParseError(s.to_string()));
        }

        let mut modifiers = Modifiers::default();
        let key_part = parts.last().unwrap();

        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            match *part {
                "Shift" => modifiers.shift = true,
                "Ctrl" => modifiers.ctrl = true,
                "Alt" => modifiers.alt = true,
                "CapsLock" => modifiers.caps_lock = true,
                _ => return Err(KeybindingParseError(format!("Unknown modifier: {}", part))),
            }
        }

        let key = parse_key(key_part)?;
        Ok(Keybinding::new(modifiers, key))
    }
}

fn parse_key(s: &str) -> Result<Key, KeybindingParseError> {
    match s {
        "Left" => Ok(Key::Left),
        "Right" => Ok(Key::Right),
        "Up" => Ok(Key::Up),
        "Down" => Ok(Key::Down),
        "Plus" => Ok(Key::Plus),
        "Minus" => Ok(Key::Minus),
        "Win" => Ok(Key::Win),
        "Shift" => Ok(Key::Shift),
        "Ctrl" => Ok(Key::Ctrl),
        "Alt" => Ok(Key::Alt),
        "OpenBracket" => Ok(Key::OpenBracket),
        "CloseBracket" => Ok(Key::CloseBracket),
        "Semicolon" => Ok(Key::Semicolon),
        "Quote" => Ok(Key::Quote),
        "Tab" => Ok(Key::Tab),
        "Escape" => Ok(Key::Escape),
        "Space" => Ok(Key::Space),
        "Enter" => Ok(Key::Enter),
        "Backspace" => Ok(Key::Backspace),
        "Delete" => Ok(Key::Delete),
        "Home" => Ok(Key::Home),
        "End" => Ok(Key::End),
        "PageUp" => Ok(Key::PageUp),
        "PageDown" => Ok(Key::PageDown),
        "CapsLock" => Ok(Key::CapsLock),
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            if c.is_ascii_alphanumeric() {
                Ok(Key::Char(c.to_ascii_uppercase()))
            } else {
                Err(KeybindingParseError(format!("Unknown key: {}", s)))
            }
        }
        s if s.starts_with('F') => {
            let num: u8 = s[1..]
                .parse()
                .map_err(|_| KeybindingParseError(format!("Invalid F key: {}", s)))?;
            if (1..=24).contains(&num) {
                Ok(Key::F(num))
            } else {
                Err(KeybindingParseError(format!("F key out of range: {}", s)))
            }
        }
        _ => Err(KeybindingParseError(format!("Unknown key: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shift_h() {
        let kb: Keybinding = "Shift+H".parse().unwrap();
        assert!(kb.modifiers.shift);
        assert!(!kb.modifiers.ctrl);
        assert!(!kb.modifiers.alt);
        assert_eq!(kb.key, Key::Char('H'));
    }

    #[test]
    fn parse_shift_ctrl_left() {
        let kb: Keybinding = "Shift+Ctrl+Left".parse().unwrap();
        assert!(kb.modifiers.shift);
        assert!(kb.modifiers.ctrl);
        assert!(!kb.modifiers.alt);
        assert_eq!(kb.key, Key::Left);
    }

    #[test]
    fn parse_caps_lock() {
        let kb: Keybinding = "CapsLock".parse().unwrap();
        assert!(!kb.modifiers.shift);
        assert!(!kb.modifiers.ctrl);
        assert!(!kb.modifiers.alt);
        assert_eq!(kb.key, Key::CapsLock);
    }

    #[test]
    fn parse_f1() {
        let kb: Keybinding = "F1".parse().unwrap();
        assert_eq!(kb.key, Key::F(1));
    }

    #[test]
    fn parse_f24() {
        let kb: Keybinding = "F24".parse().unwrap();
        assert_eq!(kb.key, Key::F(24));
    }

    #[test]
    fn parse_invalid_f_key() {
        assert!("F25".parse::<Keybinding>().is_err());
    }

    #[test]
    fn parse_invalid_modifier() {
        assert!("Win+H".parse::<Keybinding>().is_err());
    }

    #[test]
    fn parse_activation_style_win_chord() {
        let kb: Keybinding = "Shift+Win".parse().unwrap();
        assert!(kb.modifiers.shift);
        assert_eq!(kb.key, Key::Win);
    }

    #[test]
    fn parse_legacy_punctuation_key() {
        let kb: Keybinding = "CloseBracket".parse().unwrap();
        assert_eq!(kb.key, Key::CloseBracket);
    }

    #[test]
    fn parse_lowercase() {
        let kb: Keybinding = "h".parse().unwrap();
        assert_eq!(kb.key, Key::Char('H'));
    }

    #[test]
    fn serialize_roundtrip() {
        let kb: Keybinding = "Shift+Alt+Right".parse().unwrap();
        let s = kb.to_string();
        assert_eq!(s, "Shift+Alt+Right");
        let kb2: Keybinding = s.parse().unwrap();
        assert_eq!(kb, kb2);
    }

    #[test]
    fn modifiers_satisfied() {
        let binding = Modifiers {
            shift: true,
            ctrl: true,
            ..Default::default()
        };
        let pressed = Modifiers {
            shift: true,
            ctrl: true,
            alt: true,
            ..Default::default()
        };
        assert!(binding.is_satisfied_by(&pressed));
    }

    #[test]
    fn modifiers_not_satisfied() {
        let binding = Modifiers {
            shift: true,
            ctrl: true,
            ..Default::default()
        };
        let pressed = Modifiers {
            shift: true,
            ..Default::default()
        };
        assert!(!binding.is_satisfied_by(&pressed));
    }

    #[test]
    fn keybinding_matches() {
        let kb = Keybinding::new(
            Modifiers {
                shift: true,
                ..Default::default()
            },
            Key::Char('H'),
        );
        assert!(kb.matches(
            Key::Char('H'),
            &Modifiers {
                shift: true,
                ..Default::default()
            }
        ));
        assert!(!kb.matches(Key::Char('H'), &Modifiers::default()));
        assert!(!kb.matches(
            Key::Char('V'),
            &Modifiers {
                shift: true,
                ..Default::default()
            }
        ));
    }
}
