//! String parsing for key combination strings.
//!
//! This module provides parsing functionality to convert human-readable key
//! combination strings like "Ctrl+s", "Alt+Enter", or "Ctrl+Shift+x" into
//! [`KeyBinding`] and [`KeySequence`] objects.
//!
//! # Supported Formats
//!
//! ## Modifiers
//! - `Ctrl`, `Control` (case insensitive)
//! - `Alt`, `Meta`, `Option` (case insensitive)
//! - `Shift` (case insensitive)
//! - `Super`, `Win`, `Cmd`, `Command` (case insensitive)
//!
//! ## Keys
//! - Single characters: `a`, `z`, `1`, `/`, etc.
//! - Function keys: `F1`, `F12`
//! - Named keys: `Enter`, `Escape`, `Esc`, `Space`, `Tab`, `Backspace`, etc.
//! - Arrow keys: `Up`, `Down`, `Left`, `Right`
//! - Navigation: `Home`, `End`, `PageUp`, `PageDown`, `Insert`, `Delete`
//!
//! ## Syntax
//! - Modifiers and keys are separated by `+` or `-`
//! - Multiple modifiers can be combined: `Ctrl+Shift+x`
//! - Key sequences are space-separated: `Ctrl+x Ctrl+s`
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::parser::{parse_key_binding, parse_key_sequence};
//!
//! // Single key
//! let q = parse_key_binding("q").unwrap();
//!
//! // Key with modifier
//! let ctrl_s = parse_key_binding("Ctrl+s").unwrap();
//!
//! // Multiple modifiers
//! let ctrl_shift_x = parse_key_binding("Ctrl+Shift+x").unwrap();
//!
//! // Key sequence (Emacs-style)
//! let save_seq = parse_key_sequence("Ctrl+x Ctrl+s").unwrap();
//! ```

use std::fmt;

use terminput::{KeyCode, KeyModifiers};

use super::{KeyBinding, KeySequence};

/// Error type for key parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseKeyError {
    input: String,
    kind: ParseKeyErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseKeyErrorKind {
    EmptyInput,
    InvalidKey(String),
    InvalidModifier(String),
    NoKeySpecified,
}

impl ParseKeyError {
    fn empty_input() -> Self {
        Self {
            input: String::new(),
            kind: ParseKeyErrorKind::EmptyInput,
        }
    }

    fn invalid_key(input: &str, key: &str) -> Self {
        Self {
            input: input.to_string(),
            kind: ParseKeyErrorKind::InvalidKey(key.to_string()),
        }
    }

    fn invalid_modifier(input: &str, modifier: &str) -> Self {
        Self {
            input: input.to_string(),
            kind: ParseKeyErrorKind::InvalidModifier(modifier.to_string()),
        }
    }

    fn no_key_specified(input: &str) -> Self {
        Self {
            input: input.to_string(),
            kind: ParseKeyErrorKind::NoKeySpecified,
        }
    }

    /// Returns the original input string that failed to parse.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Returns the specific kind of parse error.
    pub fn kind(&self) -> &ParseKeyErrorKind {
        &self.kind
    }
}

impl fmt::Display for ParseKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseKeyErrorKind::EmptyInput => write!(f, "empty key binding string"),
            ParseKeyErrorKind::InvalidKey(key) => {
                write!(f, "invalid key '{}' in '{}'", key, self.input)
            }
            ParseKeyErrorKind::InvalidModifier(modifier) => {
                write!(f, "invalid modifier '{}' in '{}'", modifier, self.input)
            }
            ParseKeyErrorKind::NoKeySpecified => {
                write!(f, "no key specified in '{}'", self.input)
            }
        }
    }
}

impl std::error::Error for ParseKeyError {}

/// Parses a key binding string like "Ctrl+s" into a [`KeyBinding`].
///
/// # Arguments
///
/// * `input` - A string representing a key binding
///
/// # Returns
///
/// Returns `Ok(KeyBinding)` on success, or `Err(ParseKeyError)` on failure.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::parser::parse_key_binding;
/// use terminput::{KeyCode, KeyModifiers};
///
/// let binding = parse_key_binding("Ctrl+s").unwrap();
/// assert_eq!(binding.key(), KeyCode::Char('s'));
/// assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
///
/// let binding = parse_key_binding("F1").unwrap();
/// assert_eq!(binding.key(), KeyCode::F(1));
/// ```
pub fn parse_key_binding(input: &str) -> Result<KeyBinding, ParseKeyError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ParseKeyError::empty_input());
    }

    // Split by + or - (both are valid separators)
    let parts: Vec<&str> = input.split(['+', '-']).collect();

    let mut modifiers = KeyModifiers::NONE;
    let mut key_code: Option<KeyCode> = None;

    for (i, part) in parts.iter().enumerate() {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let is_last = i == parts.len() - 1;

        // Try to parse as modifier first
        if let Some(modifier) = parse_modifier(part) {
            modifiers |= modifier;
        } else if let Some(code) = parse_key_code(part) {
            // If it's a key code, it should be the last part
            if key_code.is_some() {
                // We already have a key, this might be an ambiguous modifier
                return Err(ParseKeyError::invalid_modifier(input, part));
            }
            key_code = Some(code);
        } else if is_last {
            // Last part must be a valid key
            return Err(ParseKeyError::invalid_key(input, part));
        } else {
            // Non-last part must be a modifier
            return Err(ParseKeyError::invalid_modifier(input, part));
        }
    }

    let key = key_code.ok_or_else(|| ParseKeyError::no_key_specified(input))?;

    Ok(KeyBinding::with_mods(key, modifiers))
}

/// Parses a key sequence string like "Ctrl+x Ctrl+s" into a [`KeySequence`].
///
/// Key sequences are space-separated key bindings that must be pressed in order.
///
/// # Arguments
///
/// * `input` - A space-separated string of key bindings
///
/// # Returns
///
/// Returns `Ok(KeySequence)` on success, or `Err(ParseKeyError)` on failure.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::parser::parse_key_sequence;
///
/// // Single key sequence
/// let seq = parse_key_sequence("q").unwrap();
/// assert_eq!(seq.len(), 1);
///
/// // Multi-key sequence
/// let seq = parse_key_sequence("Ctrl+x Ctrl+s").unwrap();
/// assert_eq!(seq.len(), 2);
///
/// // Vim-style gg
/// let seq = parse_key_sequence("g g").unwrap();
/// assert_eq!(seq.len(), 2);
/// ```
pub fn parse_key_sequence(input: &str) -> Result<KeySequence, ParseKeyError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ParseKeyError::empty_input());
    }

    let mut bindings = Vec::new();

    for part in input.split_whitespace() {
        let binding = parse_key_binding(part)?;
        bindings.push(binding);
    }

    if bindings.is_empty() {
        return Err(ParseKeyError::empty_input());
    }

    Ok(KeySequence::new(bindings))
}

/// Parses a modifier string into [`KeyModifiers`].
/// Only matches full modifier names, not single characters (which are keys).
fn parse_modifier(s: &str) -> Option<KeyModifiers> {
    match s.to_lowercase().as_str() {
        "ctrl" | "control" => Some(KeyModifiers::CTRL),
        "alt" | "meta" | "option" => Some(KeyModifiers::ALT),
        "shift" => Some(KeyModifiers::SHIFT),
        "super" | "win" | "cmd" | "command" => Some(KeyModifiers::SUPER),
        _ => None,
    }
}

/// Parses a key name string into [`KeyCode`].
fn parse_key_code(s: &str) -> Option<KeyCode> {
    // Single character
    if s.len() == 1 {
        return Some(KeyCode::Char(s.chars().next().unwrap()));
    }

    // Function keys
    if (s.starts_with('F') || s.starts_with('f'))
        && s.len() >= 2
        && s[1..].chars().all(|c| c.is_ascii_digit())
    {
        if let Ok(n) = s[1..].parse::<u8>() {
            if (1..=24).contains(&n) {
                return Some(KeyCode::F(n));
            }
        }
    }

    // Named keys (case insensitive)
    match s.to_lowercase().as_str() {
        "enter" | "return" | "cr" => Some(KeyCode::Enter),
        "escape" | "esc" => Some(KeyCode::Esc),
        "space" | "spc" => Some(KeyCode::Char(' ')),
        "tab" => Some(KeyCode::Tab),
        "backspace" | "bs" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "insert" | "ins" => Some(KeyCode::Insert),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "pgup" => Some(KeyCode::PageUp),
        "pagedown" | "pgdn" | "pgdown" => Some(KeyCode::PageDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_char() {
        let binding = parse_key_binding("q").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('q'));
        assert_eq!(binding.modifiers(), KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_ctrl_char() {
        let binding = parse_key_binding("Ctrl+s").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_parse_ctrl_lowercase() {
        let binding = parse_key_binding("ctrl+s").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_parse_alt_char() {
        let binding = parse_key_binding("Alt+x").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('x'));
        assert_eq!(binding.modifiers(), KeyModifiers::ALT);
    }

    #[test]
    fn test_parse_shift_char() {
        let binding = parse_key_binding("Shift+a").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('a'));
        assert_eq!(binding.modifiers(), KeyModifiers::SHIFT);
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let binding = parse_key_binding("Ctrl+Shift+x").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('x'));
        assert!(binding.modifiers().contains(KeyModifiers::CTRL));
        assert!(binding.modifiers().contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_all_modifiers() {
        let binding = parse_key_binding("Ctrl+Alt+Shift+Super+x").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('x'));
        assert!(binding.modifiers().contains(KeyModifiers::CTRL));
        assert!(binding.modifiers().contains(KeyModifiers::ALT));
        assert!(binding.modifiers().contains(KeyModifiers::SHIFT));
        assert!(binding.modifiers().contains(KeyModifiers::SUPER));
    }

    #[test]
    fn test_parse_function_key() {
        let binding = parse_key_binding("F1").unwrap();
        assert_eq!(binding.key(), KeyCode::F(1));
    }

    #[test]
    fn test_parse_function_key_with_modifier() {
        let binding = parse_key_binding("Ctrl+F12").unwrap();
        assert_eq!(binding.key(), KeyCode::F(12));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_parse_enter() {
        let binding = parse_key_binding("Enter").unwrap();
        assert_eq!(binding.key(), KeyCode::Enter);
    }

    #[test]
    fn test_parse_escape() {
        let binding = parse_key_binding("Escape").unwrap();
        assert_eq!(binding.key(), KeyCode::Esc);

        let binding = parse_key_binding("Esc").unwrap();
        assert_eq!(binding.key(), KeyCode::Esc);
    }

    #[test]
    fn test_parse_space() {
        let binding = parse_key_binding("Space").unwrap();
        assert_eq!(binding.key(), KeyCode::Char(' '));
    }

    #[test]
    fn test_parse_arrow_keys() {
        assert_eq!(parse_key_binding("Up").unwrap().key(), KeyCode::Up);
        assert_eq!(parse_key_binding("Down").unwrap().key(), KeyCode::Down);
        assert_eq!(parse_key_binding("Left").unwrap().key(), KeyCode::Left);
        assert_eq!(parse_key_binding("Right").unwrap().key(), KeyCode::Right);
    }

    #[test]
    fn test_parse_navigation_keys() {
        assert_eq!(parse_key_binding("Home").unwrap().key(), KeyCode::Home);
        assert_eq!(parse_key_binding("End").unwrap().key(), KeyCode::End);
        assert_eq!(parse_key_binding("PageUp").unwrap().key(), KeyCode::PageUp);
        assert_eq!(
            parse_key_binding("PageDown").unwrap().key(),
            KeyCode::PageDown
        );
        assert_eq!(parse_key_binding("Insert").unwrap().key(), KeyCode::Insert);
        assert_eq!(parse_key_binding("Delete").unwrap().key(), KeyCode::Delete);
    }

    #[test]
    fn test_parse_hyphen_separator() {
        let binding = parse_key_binding("Ctrl-s").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_parse_modifier_aliases() {
        // Control variants
        assert_eq!(
            parse_key_binding("Control+x").unwrap().modifiers(),
            KeyModifiers::CTRL
        );

        // Alt variants
        assert_eq!(
            parse_key_binding("Meta+x").unwrap().modifiers(),
            KeyModifiers::ALT
        );
        assert_eq!(
            parse_key_binding("Option+x").unwrap().modifiers(),
            KeyModifiers::ALT
        );

        // Super variants
        assert_eq!(
            parse_key_binding("Win+x").unwrap().modifiers(),
            KeyModifiers::SUPER
        );
        assert_eq!(
            parse_key_binding("Cmd+x").unwrap().modifiers(),
            KeyModifiers::SUPER
        );
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let binding = parse_key_binding("  Ctrl + s  ").unwrap();
        assert_eq!(binding.key(), KeyCode::Char('s'));
        assert_eq!(binding.modifiers(), KeyModifiers::CTRL);
    }

    #[test]
    fn test_parse_sequence_single() {
        let seq = parse_key_sequence("q").unwrap();
        assert_eq!(seq.len(), 1);
        assert_eq!(seq.first().key(), KeyCode::Char('q'));
    }

    #[test]
    fn test_parse_sequence_two_keys() {
        let seq = parse_key_sequence("Ctrl+x Ctrl+s").unwrap();
        assert_eq!(seq.len(), 2);
        assert_eq!(seq.keys()[0].key(), KeyCode::Char('x'));
        assert_eq!(seq.keys()[1].key(), KeyCode::Char('s'));
    }

    #[test]
    fn test_parse_sequence_vim_gg() {
        let seq = parse_key_sequence("g g").unwrap();
        assert_eq!(seq.len(), 2);
        assert_eq!(seq.keys()[0].key(), KeyCode::Char('g'));
        assert_eq!(seq.keys()[1].key(), KeyCode::Char('g'));
    }

    #[test]
    fn test_parse_error_empty() {
        let err = parse_key_binding("").unwrap_err();
        assert_eq!(err.kind(), &ParseKeyErrorKind::EmptyInput);
    }

    #[test]
    fn test_parse_error_invalid_key() {
        let err = parse_key_binding("InvalidKeyName").unwrap_err();
        matches!(err.kind(), ParseKeyErrorKind::InvalidKey(_));
    }

    #[test]
    fn test_parse_error_only_modifiers() {
        // "Ctrl+Shift" - Shift is now parsed as a key (since we removed single-letter aliases)
        // Use a truly ambiguous case
        let err = parse_key_binding("Ctrl+").unwrap_err();
        assert!(matches!(err.kind(), ParseKeyErrorKind::NoKeySpecified));
    }

    #[test]
    fn test_error_display() {
        let err = parse_key_binding("").unwrap_err();
        assert_eq!(err.to_string(), "empty key binding string");

        let err = parse_key_binding("InvalidKey").unwrap_err();
        assert!(err.to_string().contains("invalid key"));
    }

    #[test]
    fn test_parse_key_shorthand_abbreviations() {
        // Test BS = Backspace
        assert_eq!(parse_key_binding("bs").unwrap().key(), KeyCode::Backspace);

        // Test CR = Enter
        assert_eq!(parse_key_binding("CR").unwrap().key(), KeyCode::Enter);

        // Test PgUp/PgDn
        assert_eq!(parse_key_binding("PgUp").unwrap().key(), KeyCode::PageUp);
        assert_eq!(parse_key_binding("PgDn").unwrap().key(), KeyCode::PageDown);
    }
}
