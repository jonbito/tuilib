//! Key binding types for mapping keys to actions.
//!
//! This module provides the [`KeyBinding`] struct for representing single
//! key combinations (key + modifiers) that can trigger actions.
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::KeyBinding;
//! use terminput::{KeyCode, KeyModifiers};
//!
//! // Simple key binding for 'q'
//! let quit_key = KeyBinding::new(KeyCode::Char('q'));
//!
//! // Binding with modifiers: Ctrl+S
//! let save_key = KeyBinding::new(KeyCode::Char('s'))
//!     .with_modifiers(KeyModifiers::CTRL);
//!
//! // Multiple modifiers: Ctrl+Shift+S
//! let save_as_key = KeyBinding::new(KeyCode::Char('s'))
//!     .with_modifiers(KeyModifiers::CTRL | KeyModifiers::SHIFT);
//! ```

use std::fmt;
use terminput::{KeyCode, KeyEvent, KeyModifiers};

/// A single key binding consisting of a key and optional modifiers.
///
/// KeyBinding represents a specific key combination like "Ctrl+S" or just "q".
/// It can be matched against [`KeyEvent`] instances from terminput.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::KeyBinding;
/// use terminput::{KeyCode, KeyModifiers};
///
/// // Create a binding for Ctrl+X
/// let binding = KeyBinding::new(KeyCode::Char('x'))
///     .with_modifiers(KeyModifiers::CTRL);
///
/// assert_eq!(binding.key(), KeyCode::Char('x'));
/// assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    key: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Creates a new key binding with no modifiers.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code for this binding
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBinding;
    /// use terminput::KeyCode;
    ///
    /// let binding = KeyBinding::new(KeyCode::Char('q'));
    /// assert_eq!(binding.key(), KeyCode::Char('q'));
    /// assert!(binding.modifiers().is_empty());
    /// ```
    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::NONE,
        }
    }

    /// Creates a new key binding with the specified modifiers.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code for this binding
    /// * `modifiers` - The modifier keys required
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBinding;
    /// use terminput::{KeyCode, KeyModifiers};
    ///
    /// let binding = KeyBinding::with_mods(KeyCode::Char('s'), KeyModifiers::CTRL);
    /// assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    /// ```
    pub fn with_mods(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Returns a new binding with the specified modifiers added.
    ///
    /// This is a builder-style method for fluent construction.
    ///
    /// # Arguments
    ///
    /// * `modifiers` - The modifier keys to require
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBinding;
    /// use terminput::{KeyCode, KeyModifiers};
    ///
    /// let binding = KeyBinding::new(KeyCode::Char('s'))
    ///     .with_modifiers(KeyModifiers::CTRL | KeyModifiers::SHIFT);
    /// ```
    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Returns the key code of this binding.
    pub fn key(&self) -> KeyCode {
        self.key
    }

    /// Returns the modifiers of this binding.
    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }

    /// Checks if this binding matches a key event.
    ///
    /// The match succeeds if both the key code and modifiers match exactly.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to match against
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBinding;
    /// use terminput::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind, KeyEventState};
    ///
    /// let binding = KeyBinding::new(KeyCode::Char('s'))
    ///     .with_modifiers(KeyModifiers::CTRL);
    ///
    /// let event = KeyEvent {
    ///     code: KeyCode::Char('s'),
    ///     modifiers: KeyModifiers::CTRL,
    ///     kind: KeyEventKind::Press,
    ///     state: KeyEventState::NONE,
    /// };
    ///
    /// assert!(binding.matches(&event));
    /// ```
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key == event.code && self.modifiers == event.modifiers
    }

    /// Checks if this binding matches a key event, treating lowercase/uppercase
    /// characters as equivalent when Shift is the only difference.
    ///
    /// This is useful for bindings where you want "S" (Shift+s) and "s" to match
    /// the same binding without Shift explicitly in the modifiers.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to match against
    pub fn matches_ignoring_shift_case(&self, event: &KeyEvent) -> bool {
        if self.matches(event) {
            return true;
        }

        // Check if the only difference is shift modifier on a character key
        if let (KeyCode::Char(binding_char), KeyCode::Char(event_char)) = (self.key, event.code) {
            let binding_without_shift = self.modifiers - KeyModifiers::SHIFT;
            let event_without_shift = event.modifiers - KeyModifiers::SHIFT;

            if binding_without_shift == event_without_shift {
                return binding_char.eq_ignore_ascii_case(&event_char);
            }
        }

        false
    }
}

impl fmt::Debug for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyBinding")
            .field("key", &self.key)
            .field("modifiers", &self.modifiers)
            .finish()
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CTRL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        if self.modifiers.contains(KeyModifiers::SUPER) {
            parts.push("Super");
        }

        let key_str = match self.key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            _ => format!("{:?}", self.key),
        };

        parts.push(&key_str);
        write!(f, "{}", parts.join("+"))
    }
}

impl From<KeyCode> for KeyBinding {
    fn from(key: KeyCode) -> Self {
        Self::new(key)
    }
}

impl From<(KeyCode, KeyModifiers)> for KeyBinding {
    fn from((key, modifiers): (KeyCode, KeyModifiers)) -> Self {
        Self::with_mods(key, modifiers)
    }
}

impl From<char> for KeyBinding {
    fn from(c: char) -> Self {
        Self::new(KeyCode::Char(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminput::{KeyEventKind, KeyEventState};

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_binding_creation() {
        let binding = KeyBinding::new(KeyCode::Char('q'));
        assert_eq!(binding.key(), KeyCode::Char('q'));
        assert_eq!(binding.modifiers(), KeyModifiers::NONE);
    }

    #[test]
    fn test_binding_with_modifiers() {
        let binding = KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL);
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_binding_with_mods_constructor() {
        let binding =
            KeyBinding::with_mods(KeyCode::Char('s'), KeyModifiers::CTRL | KeyModifiers::SHIFT);
        assert!(binding.modifiers().contains(KeyModifiers::CTRL));
        assert!(binding.modifiers().contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_binding_matches_simple_key() {
        let binding = KeyBinding::new(KeyCode::Char('q'));
        let event = make_key_event(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(binding.matches(&event));
    }

    #[test]
    fn test_binding_matches_with_modifiers() {
        let binding = KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL);
        let event = make_key_event(KeyCode::Char('s'), KeyModifiers::CTRL);
        assert!(binding.matches(&event));
    }

    #[test]
    fn test_binding_no_match_wrong_key() {
        let binding = KeyBinding::new(KeyCode::Char('q'));
        let event = make_key_event(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!binding.matches(&event));
    }

    #[test]
    fn test_binding_no_match_missing_modifier() {
        let binding = KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL);
        let event = make_key_event(KeyCode::Char('s'), KeyModifiers::NONE);
        assert!(!binding.matches(&event));
    }

    #[test]
    fn test_binding_no_match_extra_modifier() {
        let binding = KeyBinding::new(KeyCode::Char('s'));
        let event = make_key_event(KeyCode::Char('s'), KeyModifiers::CTRL);
        assert!(!binding.matches(&event));
    }

    #[test]
    fn test_binding_display_simple() {
        let binding = KeyBinding::new(KeyCode::Char('q'));
        assert_eq!(format!("{}", binding), "q");
    }

    #[test]
    fn test_binding_display_with_ctrl() {
        let binding = KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL);
        assert_eq!(format!("{}", binding), "Ctrl+s");
    }

    #[test]
    fn test_binding_display_multiple_modifiers() {
        let binding = KeyBinding::new(KeyCode::Char('s'))
            .with_modifiers(KeyModifiers::CTRL | KeyModifiers::SHIFT);
        assert_eq!(format!("{}", binding), "Ctrl+Shift+s");
    }

    #[test]
    fn test_binding_display_function_key() {
        let binding = KeyBinding::new(KeyCode::F(1)).with_modifiers(KeyModifiers::CTRL);
        assert_eq!(format!("{}", binding), "Ctrl+F1");
    }

    #[test]
    fn test_binding_from_key_code() {
        let binding: KeyBinding = KeyCode::Char('q').into();
        assert_eq!(binding.key(), KeyCode::Char('q'));
    }

    #[test]
    fn test_binding_from_tuple() {
        let binding: KeyBinding = (KeyCode::Char('s'), KeyModifiers::CTRL).into();
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_binding_from_char() {
        let binding: KeyBinding = 'q'.into();
        assert_eq!(binding.key(), KeyCode::Char('q'));
    }

    #[test]
    fn test_binding_equality() {
        let b1 = KeyBinding::new(KeyCode::Char('q'));
        let b2 = KeyBinding::new(KeyCode::Char('q'));
        let b3 = KeyBinding::new(KeyCode::Char('w'));

        assert_eq!(b1, b2);
        assert_ne!(b1, b3);
    }

    #[test]
    fn test_binding_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(KeyBinding::new(KeyCode::Char('q')));
        set.insert(KeyBinding::new(KeyCode::Char('w')));

        assert!(set.contains(&KeyBinding::new(KeyCode::Char('q'))));
        assert!(!set.contains(&KeyBinding::new(KeyCode::Char('e'))));
    }

    #[test]
    fn test_matches_ignoring_shift_case() {
        let binding = KeyBinding::new(KeyCode::Char('s'));

        // Direct match
        let event = make_key_event(KeyCode::Char('s'), KeyModifiers::NONE);
        assert!(binding.matches_ignoring_shift_case(&event));

        // Uppercase with shift should match lowercase binding
        let event_upper = make_key_event(KeyCode::Char('S'), KeyModifiers::SHIFT);
        assert!(binding.matches_ignoring_shift_case(&event_upper));

        // But Ctrl+S should not match plain 's' binding
        let event_ctrl = make_key_event(KeyCode::Char('s'), KeyModifiers::CTRL);
        assert!(!binding.matches_ignoring_shift_case(&event_ctrl));
    }
}
