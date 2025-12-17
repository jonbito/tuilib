//! Terminput integration layer and helper utilities.
//!
//! This module provides convenient integration with the terminput crate,
//! including re-exports and helper functions for creating common key bindings.
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::{ctrl, alt, alt_key, shift_key, key, char_key};
//! use terminput::{KeyCode, KeyModifiers};
//!
//! // Create common bindings easily
//! let ctrl_s = ctrl('s');
//! let alt_x = alt('x');
//! let alt_f4 = alt_key(KeyCode::F(4));
//! let shift_tab = shift_key(KeyCode::Tab);
//! ```

use terminput::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{KeyBinding, KeySequence, KeySequenceBuilder};

/// Creates a key binding for the given character with Ctrl modifier.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::ctrl;
/// use terminput::KeyModifiers;
///
/// let binding = ctrl('s');
/// assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
/// ```
pub fn ctrl(c: char) -> KeyBinding {
    KeyBinding::with_mods(KeyCode::Char(c), KeyModifiers::CTRL)
}

/// Creates a key binding for the given key code with Ctrl modifier.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::ctrl_key;
/// use terminput::KeyCode;
///
/// let binding = ctrl_key(KeyCode::Home);
/// ```
pub fn ctrl_key(code: KeyCode) -> KeyBinding {
    KeyBinding::with_mods(code, KeyModifiers::CTRL)
}

/// Creates a key binding for the given character with Alt modifier.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::alt;
///
/// let binding = alt('x');
/// ```
pub fn alt(c: char) -> KeyBinding {
    KeyBinding::with_mods(KeyCode::Char(c), KeyModifiers::ALT)
}

/// Creates a key binding for the given key code with Alt modifier.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::alt_key;
/// use terminput::KeyCode;
///
/// let binding = alt_key(KeyCode::F(4));
/// ```
pub fn alt_key(code: KeyCode) -> KeyBinding {
    KeyBinding::with_mods(code, KeyModifiers::ALT)
}

/// Creates a key binding for the given character with Shift modifier.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::shift;
///
/// let binding = shift('a');
/// ```
pub fn shift(c: char) -> KeyBinding {
    KeyBinding::with_mods(KeyCode::Char(c), KeyModifiers::SHIFT)
}

/// Creates a key binding for the given key code with Shift modifier.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::shift_key;
/// use terminput::KeyCode;
///
/// let binding = shift_key(KeyCode::Tab);
/// ```
pub fn shift_key(code: KeyCode) -> KeyBinding {
    KeyBinding::with_mods(code, KeyModifiers::SHIFT)
}

/// Creates a key binding for a simple key (no modifiers).
///
/// # Examples
///
/// ```rust
/// use tuilib::input::key;
/// use terminput::KeyCode;
///
/// let binding = key(KeyCode::Enter);
/// ```
pub fn key(code: KeyCode) -> KeyBinding {
    KeyBinding::new(code)
}

/// Creates a key binding for a character key (no modifiers).
///
/// # Examples
///
/// ```rust
/// use tuilib::input::char_key;
///
/// let binding = char_key('q');
/// ```
pub fn char_key(c: char) -> KeyBinding {
    KeyBinding::new(KeyCode::Char(c))
}

/// Creates a key binding for a function key (no modifiers).
///
/// # Examples
///
/// ```rust
/// use tuilib::input::f_key;
///
/// let binding = f_key(1); // F1
/// ```
pub fn f_key(n: u8) -> KeyBinding {
    KeyBinding::new(KeyCode::F(n))
}

/// Creates a two-key sequence.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::{seq2, ctrl};
///
/// let save = seq2(ctrl('x'), ctrl('s'));
/// ```
pub fn seq2(first: KeyBinding, second: KeyBinding) -> KeySequence {
    KeySequence::new(vec![first, second])
}

/// Creates a key sequence from an iterator of bindings.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::{seq, char_key};
///
/// let gg = seq([char_key('g'), char_key('g')]);
/// ```
pub fn seq<I>(bindings: I) -> KeySequence
where
    I: IntoIterator<Item = KeyBinding>,
{
    KeySequence::new(bindings.into_iter().collect())
}

/// Extracts the key event from a terminput Event if it's a key press.
///
/// Returns `None` for key releases, repeats, or non-key events.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::extract_key_press;
/// use terminput::{Event, KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
///
/// let event = Event::Key(KeyEvent {
///     code: KeyCode::Char('q'),
///     modifiers: KeyModifiers::NONE,
///     kind: KeyEventKind::Press,
///     state: KeyEventState::NONE,
/// });
///
/// if let Some(key_event) = extract_key_press(&event) {
///     assert_eq!(key_event.code, KeyCode::Char('q'));
/// }
/// ```
pub fn extract_key_press(event: &Event) -> Option<&KeyEvent> {
    match event {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => Some(key_event),
        _ => None,
    }
}

/// Checks if an event is a key press event.
pub fn is_key_press(event: &Event) -> bool {
    matches!(event, Event::Key(ke) if ke.kind == KeyEventKind::Press)
}

/// Checks if an event is a key release event.
pub fn is_key_release(event: &Event) -> bool {
    matches!(event, Event::Key(ke) if ke.kind == KeyEventKind::Release)
}

/// Creates a KeyEvent for testing purposes.
///
/// This is primarily useful for unit testing input handling code.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::test_key_event;
/// use terminput::{KeyCode, KeyModifiers};
///
/// let event = test_key_event(KeyCode::Char('a'), KeyModifiers::CTRL);
/// assert_eq!(event.code, KeyCode::Char('a'));
/// ```
pub fn test_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: terminput::KeyEventState::NONE,
    }
}

/// Extension trait for KeySequenceBuilder with terminput helpers.
pub trait KeySequenceBuilderExt {
    /// Adds Ctrl+char to the sequence.
    fn ctrl(self, c: char) -> Self;

    /// Adds Alt+char to the sequence.
    fn alt(self, c: char) -> Self;

    /// Adds a plain character to the sequence.
    fn char(self, c: char) -> Self;
}

impl KeySequenceBuilderExt for KeySequenceBuilder {
    fn ctrl(self, c: char) -> Self {
        self.key(KeyCode::Char(c), KeyModifiers::CTRL)
    }

    fn alt(self, c: char) -> Self {
        self.key(KeyCode::Char(c), KeyModifiers::ALT)
    }

    fn char(self, c: char) -> Self {
        self.char_key(c)
    }
}

/// Common key bindings for typical TUI applications.
pub mod common {
    use super::*;

    /// Escape key binding.
    pub fn escape() -> KeyBinding {
        key(KeyCode::Esc)
    }

    /// Enter key binding.
    pub fn enter() -> KeyBinding {
        key(KeyCode::Enter)
    }

    /// Tab key binding.
    pub fn tab() -> KeyBinding {
        key(KeyCode::Tab)
    }

    /// Shift+Tab (backtab) key binding.
    pub fn backtab() -> KeyBinding {
        shift_key(KeyCode::Tab)
    }

    /// Backspace key binding.
    pub fn backspace() -> KeyBinding {
        key(KeyCode::Backspace)
    }

    /// Delete key binding.
    pub fn delete() -> KeyBinding {
        key(KeyCode::Delete)
    }

    /// Up arrow key binding.
    pub fn up() -> KeyBinding {
        key(KeyCode::Up)
    }

    /// Down arrow key binding.
    pub fn down() -> KeyBinding {
        key(KeyCode::Down)
    }

    /// Left arrow key binding.
    pub fn left() -> KeyBinding {
        key(KeyCode::Left)
    }

    /// Right arrow key binding.
    pub fn right() -> KeyBinding {
        key(KeyCode::Right)
    }

    /// Home key binding.
    pub fn home() -> KeyBinding {
        key(KeyCode::Home)
    }

    /// End key binding.
    pub fn end() -> KeyBinding {
        key(KeyCode::End)
    }

    /// Page Up key binding.
    pub fn page_up() -> KeyBinding {
        key(KeyCode::PageUp)
    }

    /// Page Down key binding.
    pub fn page_down() -> KeyBinding {
        key(KeyCode::PageDown)
    }

    /// Ctrl+C key binding (commonly used for quit/interrupt).
    pub fn ctrl_c() -> KeyBinding {
        ctrl('c')
    }

    /// Ctrl+D key binding (commonly used for EOF/exit).
    pub fn ctrl_d() -> KeyBinding {
        ctrl('d')
    }

    /// Ctrl+Z key binding (commonly used for undo/suspend).
    pub fn ctrl_z() -> KeyBinding {
        ctrl('z')
    }

    /// Ctrl+S key binding (commonly used for save).
    pub fn ctrl_s() -> KeyBinding {
        ctrl('s')
    }

    /// Ctrl+Q key binding (commonly used for quit).
    pub fn ctrl_q() -> KeyBinding {
        ctrl('q')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminput::KeyEventState;

    #[test]
    fn test_ctrl_helper() {
        let binding = ctrl('s');
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_ctrl_key_helper() {
        let binding = ctrl_key(KeyCode::Home);
        assert_eq!(binding.key(), KeyCode::Home);
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_alt_helper() {
        let binding = alt('x');
        assert_eq!(binding.key(), KeyCode::Char('x'));
        assert_eq!(binding.modifiers(), KeyModifiers::ALT);
    }

    #[test]
    fn test_shift_helper() {
        let binding = shift('a');
        assert_eq!(binding.key(), KeyCode::Char('a'));
        assert_eq!(binding.modifiers(), KeyModifiers::SHIFT);
    }

    #[test]
    fn test_key_helper() {
        let binding = key(KeyCode::Enter);
        assert_eq!(binding.key(), KeyCode::Enter);
        assert_eq!(binding.modifiers(), KeyModifiers::NONE);
    }

    #[test]
    fn test_char_key_helper() {
        let binding = char_key('q');
        assert_eq!(binding.key(), KeyCode::Char('q'));
        assert_eq!(binding.modifiers(), KeyModifiers::NONE);
    }

    #[test]
    fn test_f_key_helper() {
        let binding = f_key(5);
        assert_eq!(binding.key(), KeyCode::F(5));
    }

    #[test]
    fn test_seq2_helper() {
        let sequence = seq2(ctrl('x'), ctrl('s'));
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.keys()[0].modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_seq_helper() {
        let sequence = seq([char_key('g'), char_key('g')]);
        assert_eq!(sequence.len(), 2);
    }

    #[test]
    fn test_extract_key_press() {
        let press_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        let release_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        });

        assert!(extract_key_press(&press_event).is_some());
        assert!(extract_key_press(&release_event).is_none());
    }

    #[test]
    fn test_is_key_press() {
        let press_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        let release_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        });

        assert!(is_key_press(&press_event));
        assert!(!is_key_press(&release_event));
    }

    #[test]
    fn test_is_key_release() {
        let release_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        });

        assert!(is_key_release(&release_event));
    }

    #[test]
    fn test_test_key_event() {
        let event = test_key_event(KeyCode::Char('x'), KeyModifiers::CTRL);
        assert_eq!(event.code, KeyCode::Char('x'));
        assert_eq!(event.modifiers, KeyModifiers::CTRL);
        assert_eq!(event.kind, KeyEventKind::Press);
    }

    #[test]
    fn test_builder_ext() {
        let seq = KeySequenceBuilder::new().ctrl('x').ctrl('s').build();

        assert_eq!(seq.len(), 2);
        assert_eq!(seq.keys()[0].key(), KeyCode::Char('x'));
        assert_eq!(seq.keys()[0].modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_common_bindings() {
        assert_eq!(common::escape().key(), KeyCode::Esc);
        assert_eq!(common::enter().key(), KeyCode::Enter);
        assert_eq!(common::tab().key(), KeyCode::Tab);
        assert_eq!(common::backtab().modifiers(), KeyModifiers::SHIFT);
        assert_eq!(common::up().key(), KeyCode::Up);
        assert_eq!(common::down().key(), KeyCode::Down);
        assert_eq!(common::left().key(), KeyCode::Left);
        assert_eq!(common::right().key(), KeyCode::Right);
        assert_eq!(common::ctrl_c().modifiers(), KeyModifiers::CTRL);
        assert_eq!(common::ctrl_s().key(), KeyCode::Char('s'));
    }
}
