//! Key sequence handling for multi-key bindings.
//!
//! This module provides [`KeySequence`] for representing multi-key bindings
//! like "Ctrl+X Ctrl+S" that require pressing keys in sequence.
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::{KeyBinding, KeySequence};
//! use terminput::{KeyCode, KeyModifiers};
//!
//! // Create a two-key sequence: Ctrl+X Ctrl+S
//! let save_sequence = KeySequence::new(vec![
//!     KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
//!     KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
//! ]);
//!
//! // Single key can also be a sequence
//! let quit = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
//! ```

use std::fmt;
use terminput::KeyEvent;

use super::KeyBinding;

/// A sequence of key bindings that must be pressed in order.
///
/// Key sequences enable Emacs-style multi-key bindings where pressing
/// one key combination leads to another. For example, "Ctrl+X Ctrl+S"
/// is a two-key sequence commonly used for saving.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::{KeyBinding, KeySequence};
/// use terminput::{KeyCode, KeyModifiers};
///
/// // Two-key sequence
/// let sequence = KeySequence::new(vec![
///     KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
///     KeyBinding::new(KeyCode::Char('c')).with_modifiers(KeyModifiers::CTRL),
/// ]);
///
/// assert_eq!(sequence.len(), 2);
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    keys: Vec<KeyBinding>,
}

impl KeySequence {
    /// Creates a new key sequence from a vector of bindings.
    ///
    /// # Arguments
    ///
    /// * `keys` - The key bindings that make up this sequence
    ///
    /// # Panics
    ///
    /// Panics if `keys` is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::{KeyBinding, KeySequence};
    /// use terminput::KeyCode;
    ///
    /// let sequence = KeySequence::new(vec![
    ///     KeyBinding::new(KeyCode::Char('g')),
    ///     KeyBinding::new(KeyCode::Char('g')),
    /// ]);
    /// ```
    pub fn new(keys: Vec<KeyBinding>) -> Self {
        assert!(!keys.is_empty(), "Key sequence cannot be empty");
        Self { keys }
    }

    /// Creates a sequence from a single key binding.
    ///
    /// This is useful when you want to use a single key as part of a
    /// unified binding system that supports both single keys and sequences.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::{KeyBinding, KeySequence};
    /// use terminput::KeyCode;
    ///
    /// let sequence = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
    /// assert_eq!(sequence.len(), 1);
    /// ```
    pub fn single(binding: KeyBinding) -> Self {
        Self {
            keys: vec![binding],
        }
    }

    /// Returns the number of keys in this sequence.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if the sequence is empty.
    ///
    /// Note: A properly constructed KeySequence is never empty
    /// since the constructor panics on empty input.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns true if this is a single-key sequence.
    pub fn is_single(&self) -> bool {
        self.keys.len() == 1
    }

    /// Returns true if this is a multi-key sequence.
    pub fn is_multi(&self) -> bool {
        self.keys.len() > 1
    }

    /// Returns the key bindings in this sequence.
    pub fn keys(&self) -> &[KeyBinding] {
        &self.keys
    }

    /// Returns the first key binding in this sequence.
    pub fn first(&self) -> &KeyBinding {
        &self.keys[0]
    }

    /// Checks if a given key event matches the binding at the specified position.
    ///
    /// # Arguments
    ///
    /// * `position` - The index in the sequence (0-based)
    /// * `event` - The key event to match
    ///
    /// # Returns
    ///
    /// `true` if the event matches the binding at the position, or `false`
    /// if position is out of bounds or the event doesn't match.
    pub fn matches_at(&self, position: usize, event: &KeyEvent) -> bool {
        self.keys
            .get(position)
            .is_some_and(|binding| binding.matches(event))
    }

    /// Returns an iterator over the key bindings.
    pub fn iter(&self) -> impl Iterator<Item = &KeyBinding> {
        self.keys.iter()
    }
}

impl fmt::Debug for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeySequence")
            .field("keys", &self.keys)
            .finish()
    }
}

impl fmt::Display for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key_strs: Vec<String> = self.keys.iter().map(|k| k.to_string()).collect();
        write!(f, "{}", key_strs.join(" "))
    }
}

impl From<KeyBinding> for KeySequence {
    fn from(binding: KeyBinding) -> Self {
        Self::single(binding)
    }
}

impl From<Vec<KeyBinding>> for KeySequence {
    fn from(keys: Vec<KeyBinding>) -> Self {
        Self::new(keys)
    }
}

impl<'a> IntoIterator for &'a KeySequence {
    type Item = &'a KeyBinding;
    type IntoIter = std::slice::Iter<'a, KeyBinding>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.iter()
    }
}

impl IntoIterator for KeySequence {
    type Item = KeyBinding;
    type IntoIter = std::vec::IntoIter<KeyBinding>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter()
    }
}

/// Builder for creating key sequences with a fluent API.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::KeySequenceBuilder;
/// use terminput::{KeyCode, KeyModifiers};
///
/// let sequence = KeySequenceBuilder::new()
///     .key(KeyCode::Char('x'), KeyModifiers::CTRL)
///     .key(KeyCode::Char('s'), KeyModifiers::CTRL)
///     .build();
///
/// assert_eq!(sequence.len(), 2);
/// ```
#[derive(Default)]
pub struct KeySequenceBuilder {
    keys: Vec<KeyBinding>,
}

impl KeySequenceBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a key with modifiers to the sequence.
    pub fn key(mut self, code: terminput::KeyCode, modifiers: terminput::KeyModifiers) -> Self {
        self.keys.push(KeyBinding::with_mods(code, modifiers));
        self
    }

    /// Adds a key without modifiers to the sequence.
    pub fn simple_key(mut self, code: terminput::KeyCode) -> Self {
        self.keys.push(KeyBinding::new(code));
        self
    }

    /// Adds a character key without modifiers.
    pub fn char_key(mut self, c: char) -> Self {
        self.keys.push(KeyBinding::new(terminput::KeyCode::Char(c)));
        self
    }

    /// Adds an existing KeyBinding to the sequence.
    pub fn binding(mut self, binding: KeyBinding) -> Self {
        self.keys.push(binding);
        self
    }

    /// Builds the key sequence.
    ///
    /// # Panics
    ///
    /// Panics if no keys have been added.
    pub fn build(self) -> KeySequence {
        KeySequence::new(self.keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminput::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_sequence_creation() {
        let seq = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
            KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
        ]);

        assert_eq!(seq.len(), 2);
        assert!(!seq.is_single());
        assert!(seq.is_multi());
    }

    #[test]
    fn test_sequence_single() {
        let seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));

        assert_eq!(seq.len(), 1);
        assert!(seq.is_single());
        assert!(!seq.is_multi());
    }

    #[test]
    #[should_panic(expected = "Key sequence cannot be empty")]
    fn test_sequence_empty_panics() {
        KeySequence::new(vec![]);
    }

    #[test]
    fn test_sequence_first() {
        let seq = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('b')),
        ]);

        assert_eq!(seq.first().key(), KeyCode::Char('a'));
    }

    #[test]
    fn test_sequence_matches_at() {
        let seq = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
            KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
        ]);

        let ctrl_x = make_key_event(KeyCode::Char('x'), KeyModifiers::CTRL);
        let ctrl_s = make_key_event(KeyCode::Char('s'), KeyModifiers::CTRL);
        let just_s = make_key_event(KeyCode::Char('s'), KeyModifiers::NONE);

        assert!(seq.matches_at(0, &ctrl_x));
        assert!(!seq.matches_at(0, &ctrl_s));

        assert!(seq.matches_at(1, &ctrl_s));
        assert!(!seq.matches_at(1, &just_s));

        // Out of bounds
        assert!(!seq.matches_at(2, &ctrl_x));
    }

    #[test]
    fn test_sequence_display_single() {
        let seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
        assert_eq!(format!("{}", seq), "q");
    }

    #[test]
    fn test_sequence_display_multi() {
        let seq = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
            KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
        ]);
        assert_eq!(format!("{}", seq), "Ctrl+x Ctrl+s");
    }

    #[test]
    fn test_sequence_from_binding() {
        let binding = KeyBinding::new(KeyCode::Char('q'));
        let seq: KeySequence = binding.into();
        assert!(seq.is_single());
    }

    #[test]
    fn test_sequence_from_vec() {
        let bindings = vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('b')),
        ];
        let seq: KeySequence = bindings.into();
        assert_eq!(seq.len(), 2);
    }

    #[test]
    fn test_sequence_iter() {
        let seq = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('b')),
        ]);

        let keys: Vec<_> = seq.iter().collect();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].key(), KeyCode::Char('a'));
        assert_eq!(keys[1].key(), KeyCode::Char('b'));
    }

    #[test]
    fn test_sequence_into_iter() {
        let seq = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('b')),
        ]);

        let keys: Vec<_> = seq.into_iter().collect();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_sequence_equality() {
        let seq1 = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('b')),
        ]);
        let seq2 = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('b')),
        ]);
        let seq3 = KeySequence::new(vec![
            KeyBinding::new(KeyCode::Char('a')),
            KeyBinding::new(KeyCode::Char('c')),
        ]);

        assert_eq!(seq1, seq2);
        assert_ne!(seq1, seq3);
    }

    #[test]
    fn test_builder() {
        let seq = KeySequenceBuilder::new()
            .key(KeyCode::Char('x'), KeyModifiers::CTRL)
            .key(KeyCode::Char('s'), KeyModifiers::CTRL)
            .build();

        assert_eq!(seq.len(), 2);
        assert_eq!(seq.keys()[0].modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_builder_simple_key() {
        let seq = KeySequenceBuilder::new()
            .simple_key(KeyCode::Char('g'))
            .simple_key(KeyCode::Char('g'))
            .build();

        assert_eq!(seq.len(), 2);
        assert_eq!(seq.keys()[0].modifiers(), KeyModifiers::NONE);
    }

    #[test]
    fn test_builder_char_key() {
        let seq = KeySequenceBuilder::new()
            .char_key('g')
            .char_key('g')
            .build();

        assert_eq!(seq.len(), 2);
        assert_eq!(seq.keys()[0].key(), KeyCode::Char('g'));
    }

    #[test]
    fn test_builder_binding() {
        let binding = KeyBinding::new(KeyCode::Char('q')).with_modifiers(KeyModifiers::CTRL);
        let seq = KeySequenceBuilder::new().binding(binding).build();

        assert!(seq.is_single());
        assert_eq!(seq.first().modifiers(), KeyModifiers::CTRL);
    }
}
