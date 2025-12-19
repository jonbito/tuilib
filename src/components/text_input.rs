//! TextInput component with cursor, selection, and validation support.
//!
//! A full-featured single-line text input component supporting:
//! - Cursor positioning and movement
//! - Text selection with shift+arrow keys
//! - Word navigation with Ctrl+Left/Right
//! - Clipboard operations (cut/copy/paste)
//! - Validation with error display
//! - Placeholder text and character limits
//!
//! # Examples
//!
//! ```rust
//! use tuilib::components::{TextInput, ValidationResult};
//!
//! // Basic text input
//! let mut input = TextInput::new();
//!
//! // With placeholder and max length
//! let mut input = TextInput::new()
//!     .with_placeholder("Enter your name...")
//!     .with_max_length(50);
//!
//! // With validation
//! let mut input = TextInput::new()
//!     .with_validator(|text| {
//!         if text.is_empty() {
//!             ValidationResult::Invalid("Field is required".to_string())
//!         } else {
//!             ValidationResult::Valid
//!         }
//!     });
//! ```

use std::ops::Range;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Focusable, Renderable};
use crate::theme::Theme;

/// Type alias for validation functions.
pub type ValidatorFn = Box<dyn Fn(&str) -> ValidationResult + Send + Sync>;

/// Result of validating text input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// The input is valid.
    Valid,
    /// The input is invalid with an error message.
    Invalid(String),
    /// The input has a warning (valid but with a note).
    Warning(String),
}

impl ValidationResult {
    /// Returns true if the validation passed (Valid or Warning).
    pub fn is_valid(&self) -> bool {
        !matches!(self, ValidationResult::Invalid(_))
    }

    /// Returns the message if there is one.
    pub fn message(&self) -> Option<&str> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::Invalid(msg) | ValidationResult::Warning(msg) => Some(msg),
        }
    }
}

/// Messages that the TextInput component can handle.
#[derive(Debug, Clone)]
pub enum TextInputMsg {
    /// Insert a character at the cursor position.
    InsertChar(char),
    /// Delete the character before the cursor (backspace).
    Backspace,
    /// Delete the character at the cursor (delete key).
    Delete,
    /// Move cursor left.
    CursorLeft,
    /// Move cursor right.
    CursorRight,
    /// Move cursor to start of text.
    CursorHome,
    /// Move cursor to end of text.
    CursorEnd,
    /// Move cursor to previous word boundary.
    CursorWordLeft,
    /// Move cursor to next word boundary.
    CursorWordRight,
    /// Extend selection left.
    SelectLeft,
    /// Extend selection right.
    SelectRight,
    /// Extend selection to start.
    SelectHome,
    /// Extend selection to end.
    SelectEnd,
    /// Extend selection to previous word.
    SelectWordLeft,
    /// Extend selection to next word.
    SelectWordRight,
    /// Select all text.
    SelectAll,
    /// Clear selection without moving cursor.
    ClearSelection,
    /// Cut selected text to clipboard.
    Cut,
    /// Copy selected text to clipboard.
    Copy,
    /// Paste text from clipboard.
    Paste(String),
    /// Set the entire text content.
    SetText(String),
    /// Clear all text.
    Clear,
}

/// Actions emitted by the TextInput component.
#[derive(Debug, Clone)]
pub enum TextInputAction {
    /// The text content changed.
    Changed(String),
    /// Text was cut to clipboard.
    CutToClipboard(String),
    /// Text was copied to clipboard.
    CopiedToClipboard(String),
    /// The user pressed Enter (submit).
    Submit(String),
}

/// A single-line text input component with cursor, selection, and validation.
pub struct TextInput {
    /// The current text content.
    text: String,
    /// Cursor position (byte index in text).
    cursor: usize,
    /// Selection range if any (byte indices).
    selection: Option<Range<usize>>,
    /// Placeholder text shown when empty.
    placeholder: Option<String>,
    /// Maximum number of characters allowed.
    max_length: Option<usize>,
    /// Validation function.
    validator: Option<ValidatorFn>,
    /// Current validation error/warning.
    validation_message: Option<ValidationResult>,
    /// Whether the input is focused.
    focused: bool,
    /// Optional theme for styling.
    theme: Option<Theme>,
}

impl std::fmt::Debug for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("text", &self.text)
            .field("cursor", &self.cursor)
            .field("selection", &self.selection)
            .field("placeholder", &self.placeholder)
            .field("max_length", &self.max_length)
            .field("validator", &self.validator.as_ref().map(|_| "<fn>"))
            .field("validation_message", &self.validation_message)
            .field("focused", &self.focused)
            .field("theme", &self.theme.as_ref().map(|t| t.name()))
            .finish()
    }
}

impl Clone for TextInput {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            cursor: self.cursor,
            selection: self.selection.clone(),
            placeholder: self.placeholder.clone(),
            max_length: self.max_length,
            validator: None, // Validators cannot be cloned
            validation_message: self.validation_message.clone(),
            focused: self.focused,
            theme: self.theme.clone(),
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Creates a new empty TextInput.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection: None,
            placeholder: None,
            max_length: None,
            validator: None,
            validation_message: None,
            focused: false,
            theme: None,
        }
    }

    /// Sets the placeholder text shown when the input is empty.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the maximum character length.
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Sets a validation function.
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> ValidationResult + Send + Sync + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Sets the theme for styling.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the current text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Sets the text content and resets cursor/selection.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.text.len();
        self.selection = None;
        self.validate();
    }

    /// Returns the cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the current selection range if any.
    pub fn selection(&self) -> Option<Range<usize>> {
        self.selection.clone()
    }

    /// Returns the selected text if any.
    pub fn selected_text(&self) -> Option<&str> {
        self.selection.as_ref().map(|r| &self.text[r.clone()])
    }

    /// Returns whether the input is valid.
    pub fn is_valid(&self) -> bool {
        self.validation_message
            .as_ref()
            .map(|v| v.is_valid())
            .unwrap_or(true)
    }

    /// Returns the validation message if any.
    pub fn validation_message(&self) -> Option<&ValidationResult> {
        self.validation_message.as_ref()
    }

    /// Runs validation and updates the validation message.
    fn validate(&mut self) {
        if let Some(ref validator) = self.validator {
            let result = validator(&self.text);
            self.validation_message = if matches!(result, ValidationResult::Valid) {
                None
            } else {
                Some(result)
            };
        }
    }

    /// Returns the character count (not byte count).
    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// Converts a character index to a byte index.
    fn char_to_byte_index(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Converts a byte index to a character index.
    fn byte_to_char_index(&self, byte_idx: usize) -> usize {
        self.text[..byte_idx].chars().count()
    }

    /// Finds the previous word boundary from the current cursor.
    fn prev_word_boundary(&self) -> usize {
        let chars: Vec<(usize, char)> = self.text.char_indices().collect();
        let char_pos = self.byte_to_char_index(self.cursor);

        if char_pos == 0 {
            return 0;
        }

        let mut pos = char_pos - 1;

        // Skip whitespace
        while pos > 0 && chars[pos].1.is_whitespace() {
            pos -= 1;
        }

        // Skip word characters
        while pos > 0 && !chars[pos - 1].1.is_whitespace() {
            pos -= 1;
        }

        self.char_to_byte_index(pos)
    }

    /// Finds the next word boundary from the current cursor.
    fn next_word_boundary(&self) -> usize {
        let chars: Vec<(usize, char)> = self.text.char_indices().collect();
        let char_pos = self.byte_to_char_index(self.cursor);
        let len = chars.len();

        if char_pos >= len {
            return self.text.len();
        }

        let mut pos = char_pos;

        // Skip current word characters
        while pos < len && !chars[pos].1.is_whitespace() {
            pos += 1;
        }

        // Skip whitespace
        while pos < len && chars[pos].1.is_whitespace() {
            pos += 1;
        }

        self.char_to_byte_index(pos)
    }

    /// Moves cursor left by one character.
    fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            let char_pos = self.byte_to_char_index(self.cursor);
            self.cursor = self.char_to_byte_index(char_pos.saturating_sub(1));
        }
        self.selection = None;
    }

    /// Moves cursor right by one character.
    fn move_cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            let char_pos = self.byte_to_char_index(self.cursor);
            self.cursor = self.char_to_byte_index(char_pos + 1);
        }
        self.selection = None;
    }

    /// Extends selection in the given direction.
    fn extend_selection(&mut self, new_cursor: usize) {
        let (anchor, _) = match &self.selection {
            Some(range) => {
                // Determine which end is the anchor
                if self.cursor == range.start {
                    (range.end, range.start)
                } else {
                    (range.start, range.end)
                }
            }
            None => (self.cursor, self.cursor),
        };

        self.cursor = new_cursor;

        if anchor == new_cursor {
            self.selection = None;
        } else if anchor < new_cursor {
            self.selection = Some(anchor..new_cursor);
        } else {
            self.selection = Some(new_cursor..anchor);
        }
    }

    /// Deletes the current selection and returns the deleted text.
    fn delete_selection(&mut self) -> Option<String> {
        if let Some(range) = self.selection.take() {
            let deleted = self.text[range.clone()].to_string();
            self.text.replace_range(range.clone(), "");
            self.cursor = range.start;
            self.validate();
            Some(deleted)
        } else {
            None
        }
    }

    /// Inserts text at the cursor position.
    fn insert_text(&mut self, text: &str) -> bool {
        // Check max length
        if let Some(max) = self.max_length {
            let current_chars = self.char_count();
            let insert_chars = text.chars().count();
            let selection_chars = self
                .selection
                .as_ref()
                .map(|r| self.text[r.clone()].chars().count())
                .unwrap_or(0);

            if current_chars - selection_chars + insert_chars > max {
                return false;
            }
        }

        // Delete selection first if any
        self.delete_selection();

        // Insert new text
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.validate();
        true
    }
}

impl Component for TextInput {
    type Message = TextInputMsg;
    type Action = TextInputAction;

    fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
        match msg {
            TextInputMsg::InsertChar(c) => {
                if self.insert_text(&c.to_string()) {
                    Some(TextInputAction::Changed(self.text.clone()))
                } else {
                    None
                }
            }
            TextInputMsg::Backspace => {
                if self.selection.is_some() {
                    self.delete_selection();
                    Some(TextInputAction::Changed(self.text.clone()))
                } else if self.cursor > 0 {
                    let char_pos = self.byte_to_char_index(self.cursor);
                    let new_cursor = self.char_to_byte_index(char_pos - 1);
                    self.text.drain(new_cursor..self.cursor);
                    self.cursor = new_cursor;
                    self.validate();
                    Some(TextInputAction::Changed(self.text.clone()))
                } else {
                    None
                }
            }
            TextInputMsg::Delete => {
                if self.selection.is_some() {
                    self.delete_selection();
                    Some(TextInputAction::Changed(self.text.clone()))
                } else if self.cursor < self.text.len() {
                    let char_pos = self.byte_to_char_index(self.cursor);
                    let end = self.char_to_byte_index(char_pos + 1);
                    self.text.drain(self.cursor..end);
                    self.validate();
                    Some(TextInputAction::Changed(self.text.clone()))
                } else {
                    None
                }
            }
            TextInputMsg::CursorLeft => {
                self.move_cursor_left();
                None
            }
            TextInputMsg::CursorRight => {
                self.move_cursor_right();
                None
            }
            TextInputMsg::CursorHome => {
                self.cursor = 0;
                self.selection = None;
                None
            }
            TextInputMsg::CursorEnd => {
                self.cursor = self.text.len();
                self.selection = None;
                None
            }
            TextInputMsg::CursorWordLeft => {
                self.cursor = self.prev_word_boundary();
                self.selection = None;
                None
            }
            TextInputMsg::CursorWordRight => {
                self.cursor = self.next_word_boundary();
                self.selection = None;
                None
            }
            TextInputMsg::SelectLeft => {
                if self.cursor > 0 {
                    let char_pos = self.byte_to_char_index(self.cursor);
                    let new_cursor = self.char_to_byte_index(char_pos.saturating_sub(1));
                    self.extend_selection(new_cursor);
                }
                None
            }
            TextInputMsg::SelectRight => {
                if self.cursor < self.text.len() {
                    let char_pos = self.byte_to_char_index(self.cursor);
                    let new_cursor = self.char_to_byte_index(char_pos + 1);
                    self.extend_selection(new_cursor);
                }
                None
            }
            TextInputMsg::SelectHome => {
                self.extend_selection(0);
                None
            }
            TextInputMsg::SelectEnd => {
                self.extend_selection(self.text.len());
                None
            }
            TextInputMsg::SelectWordLeft => {
                let new_cursor = self.prev_word_boundary();
                self.extend_selection(new_cursor);
                None
            }
            TextInputMsg::SelectWordRight => {
                let new_cursor = self.next_word_boundary();
                self.extend_selection(new_cursor);
                None
            }
            TextInputMsg::SelectAll => {
                if !self.text.is_empty() {
                    self.selection = Some(0..self.text.len());
                    self.cursor = self.text.len();
                }
                None
            }
            TextInputMsg::ClearSelection => {
                self.selection = None;
                None
            }
            TextInputMsg::Cut => self.delete_selection().map(TextInputAction::CutToClipboard),
            TextInputMsg::Copy => self
                .selected_text()
                .map(|t| TextInputAction::CopiedToClipboard(t.to_string())),
            TextInputMsg::Paste(text) => {
                if self.insert_text(&text) {
                    Some(TextInputAction::Changed(self.text.clone()))
                } else {
                    None
                }
            }
            TextInputMsg::SetText(text) => {
                self.set_text(text);
                Some(TextInputAction::Changed(self.text.clone()))
            }
            TextInputMsg::Clear => {
                self.text.clear();
                self.cursor = 0;
                self.selection = None;
                self.validate();
                Some(TextInputAction::Changed(String::new()))
            }
        }
    }
}

impl Focusable for TextInput {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn on_focus(&mut self) {
        // Optionally select all text on focus
    }

    fn on_blur(&mut self) {
        // Validate on blur
        self.validate();
        // Clear selection on blur
        self.selection = None;
    }
}

impl Renderable for TextInput {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme.as_ref().cloned().unwrap_or_default();

        // Determine styles
        let text_style = if self.focused {
            theme.input_focused_style()
        } else {
            theme.input_normal_style()
        };

        let border_style = if self.focused {
            theme.border_focused_style()
        } else {
            theme.border_style()
        };

        // Build block with border
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(theme.components().input.border_type)
            .border_style(border_style);

        // Add error indicator to title if validation failed
        if let Some(ref validation) = self.validation_message {
            if let Some(msg) = validation.message() {
                let title_style = match validation {
                    ValidationResult::Invalid(_) => theme.error_text_style(),
                    ValidationResult::Warning(_) => theme.warning_text_style(),
                    ValidationResult::Valid => text_style,
                };
                block = block.title(Span::styled(msg, title_style));
            }
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Render text content
        if self.text.is_empty() {
            // Show placeholder
            if let Some(ref placeholder) = self.placeholder {
                let placeholder_style = theme.input_placeholder_style();
                let paragraph = Paragraph::new(placeholder.as_str()).style(placeholder_style);
                frame.render_widget(paragraph, inner_area);
            }
        } else {
            // Build spans with selection highlighting
            let spans = self.build_text_spans(&theme);
            let paragraph = Paragraph::new(Line::from(spans)).style(text_style);
            frame.render_widget(paragraph, inner_area);
        }

        // Render cursor if focused
        if self.focused && inner_area.width > 0 {
            let cursor_char_pos = self.byte_to_char_index(self.cursor);
            let cursor_x = inner_area.x + cursor_char_pos as u16;

            if cursor_x < inner_area.x + inner_area.width {
                // Get character at cursor or space if at end
                let cursor_char = if self.cursor < self.text.len() {
                    self.text[self.cursor..].chars().next().unwrap_or(' ')
                } else {
                    ' '
                };

                let cursor_style = theme.input_cursor_style();
                let cursor_span = Span::styled(cursor_char.to_string(), cursor_style);
                let cursor_area = Rect::new(cursor_x, inner_area.y, 1, 1);
                frame.render_widget(Paragraph::new(cursor_span), cursor_area);
            }
        }
    }
}

impl TextInput {
    /// Builds text spans with selection highlighting.
    fn build_text_spans(&self, theme: &Theme) -> Vec<Span<'_>> {
        let mut spans = Vec::new();
        let selection_style = Style::default()
            .add_modifier(theme.components().input.selection_modifier)
            .bg(theme.colors().primary);

        match &self.selection {
            Some(range) => {
                // Before selection
                if range.start > 0 {
                    spans.push(Span::raw(&self.text[..range.start]));
                }
                // Selection
                spans.push(Span::styled(&self.text[range.clone()], selection_style));
                // After selection
                if range.end < self.text.len() {
                    spans.push(Span::raw(&self.text[range.end..]));
                }
            }
            None => {
                spans.push(Span::raw(&self.text));
            }
        }

        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_text_input() {
        let input = TextInput::new();
        assert!(input.text().is_empty());
        assert_eq!(input.cursor(), 0);
        assert!(input.selection().is_none());
        assert!(input.is_valid());
    }

    #[test]
    fn test_insert_char() {
        let mut input = TextInput::new();
        input.update(TextInputMsg::InsertChar('a'));
        assert_eq!(input.text(), "a");
        assert_eq!(input.cursor(), 1);

        input.update(TextInputMsg::InsertChar('b'));
        assert_eq!(input.text(), "ab");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TextInput::new();
        input.set_text("hello");

        input.update(TextInputMsg::CursorHome);
        assert_eq!(input.cursor(), 0);

        input.update(TextInputMsg::CursorEnd);
        assert_eq!(input.cursor(), 5);

        input.update(TextInputMsg::CursorLeft);
        assert_eq!(input.cursor(), 4);

        input.update(TextInputMsg::CursorRight);
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_word_navigation() {
        let mut input = TextInput::new();
        input.set_text("hello world test");

        // Start at end
        assert_eq!(input.cursor(), 16);

        input.update(TextInputMsg::CursorWordLeft);
        assert_eq!(input.cursor(), 12); // Before "test"

        input.update(TextInputMsg::CursorWordLeft);
        assert_eq!(input.cursor(), 6); // Before "world"

        input.update(TextInputMsg::CursorWordRight);
        assert_eq!(input.cursor(), 12); // Before "test"
    }

    #[test]
    fn test_backspace() {
        let mut input = TextInput::new();
        input.set_text("hello");

        input.update(TextInputMsg::Backspace);
        assert_eq!(input.text(), "hell");
        assert_eq!(input.cursor(), 4);
    }

    #[test]
    fn test_delete() {
        let mut input = TextInput::new();
        input.set_text("hello");
        input.cursor = 0;

        input.update(TextInputMsg::Delete);
        assert_eq!(input.text(), "ello");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_selection() {
        let mut input = TextInput::new();
        input.set_text("hello");
        input.cursor = 0;

        input.update(TextInputMsg::SelectRight);
        assert_eq!(input.selection(), Some(0..1));

        input.update(TextInputMsg::SelectRight);
        assert_eq!(input.selection(), Some(0..2));

        input.update(TextInputMsg::SelectAll);
        assert_eq!(input.selection(), Some(0..5));
    }

    #[test]
    fn test_delete_selection() {
        let mut input = TextInput::new();
        input.set_text("hello world");
        input.selection = Some(0..6); // "hello "
        input.cursor = 6;

        input.update(TextInputMsg::Backspace);
        assert_eq!(input.text(), "world");
        assert!(input.selection().is_none());
    }

    #[test]
    fn test_cut_copy() {
        let mut input = TextInput::new();
        input.set_text("hello");
        input.selection = Some(0..5);
        input.cursor = 5;

        // Test copy
        let action = input.update(TextInputMsg::Copy);
        assert!(matches!(
            action,
            Some(TextInputAction::CopiedToClipboard(ref s)) if s == "hello"
        ));
        assert_eq!(input.text(), "hello"); // Text unchanged

        // Test cut
        let action = input.update(TextInputMsg::Cut);
        assert!(matches!(
            action,
            Some(TextInputAction::CutToClipboard(ref s)) if s == "hello"
        ));
        assert!(input.text().is_empty());
    }

    #[test]
    fn test_paste() {
        let mut input = TextInput::new();
        input.update(TextInputMsg::Paste("hello".to_string()));
        assert_eq!(input.text(), "hello");

        input.update(TextInputMsg::Paste(" world".to_string()));
        assert_eq!(input.text(), "hello world");
    }

    #[test]
    fn test_max_length() {
        let mut input = TextInput::new().with_max_length(5);

        input.update(TextInputMsg::Paste("hello".to_string()));
        assert_eq!(input.text(), "hello");

        // Should not insert more
        let action = input.update(TextInputMsg::InsertChar('!'));
        assert!(action.is_none());
        assert_eq!(input.text(), "hello");
    }

    #[test]
    fn test_validation() {
        let mut input = TextInput::new().with_validator(|text| {
            if text.is_empty() {
                ValidationResult::Invalid("Required".to_string())
            } else {
                ValidationResult::Valid
            }
        });

        // Initially valid (empty, no validation run yet)
        assert!(input.is_valid());

        // After setting text and validating
        input.set_text("");
        assert!(!input.is_valid());

        input.set_text("hello");
        assert!(input.is_valid());
    }

    #[test]
    fn test_unicode_handling() {
        let mut input = TextInput::new();
        input.set_text("héllo wörld");

        input.update(TextInputMsg::CursorHome);
        input.update(TextInputMsg::CursorRight);
        // Should be at character position 1, not byte position 1
        assert_eq!(input.byte_to_char_index(input.cursor()), 1);

        input.update(TextInputMsg::Delete);
        // Should delete 'é' (2 bytes) not just 1 byte
        assert_eq!(input.text(), "hllo wörld");
    }

    #[test]
    fn test_placeholder() {
        let input = TextInput::new().with_placeholder("Enter text...");
        assert!(input.text().is_empty());
        // Placeholder is used during rendering, not stored in text
    }

    #[test]
    fn test_focusable() {
        let mut input = TextInput::new();
        assert!(!input.is_focused());

        input.set_focused(true);
        assert!(input.is_focused());

        input.set_focused(false);
        assert!(!input.is_focused());
    }
}
