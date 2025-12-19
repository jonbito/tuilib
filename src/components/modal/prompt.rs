//! Prompt modal dialog.
//!
//! A modal for getting text input from the user.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{
    calculate_modal_area, Button, ButtonAction, ButtonVariant, Modal, ModalAction, ModalConfig,
    ModalMsg, Overlay,
};
use crate::components::{Component, Focusable, Renderable, TextInput};
use crate::focus::FocusId;
use crate::theme::Theme;

/// A prompt modal dialog with a text input and OK/Cancel buttons.
///
/// Prompt modals are used to get text input from the user. Returns the
/// entered text if the user confirms, or `None` if they cancel.
///
/// # Example
///
/// ```rust
/// use tuilib::components::Component;
/// use tuilib::components::modal::{PromptModal, ModalMsg, ModalAction};
///
/// let mut modal = PromptModal::new("Rename File", "Enter new filename:")
///     .with_default("untitled.txt");
///
/// // Handle user submission
/// match modal.update(ModalMsg::Confirm) {
///     Some(ModalAction::Submit(text)) => {
///         // User submitted the text
///         println!("New filename: {}", text);
///     }
///     Some(ModalAction::Close) => {
///         // User cancelled
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PromptModal {
    /// Modal configuration.
    config: ModalConfig,
    /// The message/prompt to display.
    message: String,
    /// The text input component.
    input: TextInput,
    /// The OK button.
    ok_button: Button,
    /// The Cancel button.
    cancel_button: Button,
    /// Index of the currently focused element (0 = input, 1 = OK, 2 = Cancel).
    focused_element: usize,
    /// Optional theme for styling.
    theme: Option<Theme>,
    /// Overlay for background dimming.
    overlay: Overlay,
    /// Custom labels for buttons.
    ok_label: String,
    cancel_label: String,
}

impl PromptModal {
    /// Creates a new prompt modal with the given title and message.
    ///
    /// # Arguments
    ///
    /// * `title` - Title displayed at the top of the modal
    /// * `message` - Prompt message displayed above the input
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let config = ModalConfig::new(title);

        let mut input = TextInput::new();
        input.set_focused(true);

        let ok_button = Button::new("prompt-ok", "OK").with_variant(ButtonVariant::Primary);
        let cancel_button =
            Button::new("prompt-cancel", "Cancel").with_variant(ButtonVariant::Default);

        Self {
            config,
            message: message.into(),
            input,
            ok_button,
            cancel_button,
            focused_element: 0, // Input focused by default
            theme: None,
            overlay: Overlay::new().with_shadow(true),
            ok_label: "OK".to_string(),
            cancel_label: "Cancel".to_string(),
        }
    }

    /// Sets the theme for styling.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.input = self.input.with_theme(theme.clone());
        self.ok_button = self.ok_button.with_theme(theme.clone());
        self.cancel_button = self.cancel_button.with_theme(theme.clone());
        self.overlay = self.overlay.with_theme(theme.clone());
        self.theme = Some(theme);
        self
    }

    /// Sets whether Escape closes the modal.
    pub fn with_close_on_escape(mut self, value: bool) -> Self {
        self.config = self.config.close_on_escape(value);
        self
    }

    /// Sets the width percentage (0.0 to 1.0).
    pub fn with_width_percent(mut self, value: f32) -> Self {
        self.config = self.config.width_percent(value);
        self
    }

    /// Sets whether to show the overlay.
    pub fn with_overlay(mut self, value: bool) -> Self {
        self.config = self.config.show_overlay(value);
        self
    }

    /// Sets whether to show a shadow.
    pub fn with_shadow(mut self, value: bool) -> Self {
        self.config = self.config.show_shadow(value);
        self.overlay = self.overlay.with_shadow(value);
        self
    }

    /// Sets the default text in the input.
    pub fn with_default(mut self, text: impl Into<String>) -> Self {
        self.input.set_text(text);
        self
    }

    /// Sets a placeholder for the input.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input = self.input.with_placeholder(placeholder);
        self
    }

    /// Sets custom button labels.
    ///
    /// # Arguments
    ///
    /// * `ok_label` - Label for the confirmation button (default: "OK")
    /// * `cancel_label` - Label for the cancellation button (default: "Cancel")
    pub fn with_labels(
        mut self,
        ok_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) -> Self {
        self.ok_label = ok_label.into();
        self.cancel_label = cancel_label.into();
        self.ok_button =
            Button::new("prompt-ok", self.ok_label.clone()).with_variant(ButtonVariant::Primary);
        self.cancel_button = Button::new("prompt-cancel", self.cancel_label.clone())
            .with_variant(ButtonVariant::Default);

        // Preserve theme
        if let Some(ref theme) = self.theme {
            self.ok_button = self.ok_button.with_theme(theme.clone());
            self.cancel_button = self.cancel_button.with_theme(theme.clone());
        }
        self
    }

    /// Returns the modal title.
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Returns the modal message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the current input text.
    pub fn text(&self) -> &str {
        self.input.text()
    }

    /// Returns a reference to the text input.
    pub fn input(&self) -> &TextInput {
        &self.input
    }

    /// Returns a reference to the OK button.
    pub fn ok_button(&self) -> &Button {
        &self.ok_button
    }

    /// Returns a reference to the Cancel button.
    pub fn cancel_button(&self) -> &Button {
        &self.cancel_button
    }

    /// Returns the index of the currently focused element.
    /// (0 = input, 1 = OK, 2 = Cancel)
    pub fn focused_element_index(&self) -> usize {
        self.focused_element
    }

    /// Returns the modal configuration.
    pub fn config(&self) -> &ModalConfig {
        &self.config
    }

    /// Updates the focus state of all elements based on focused_element index.
    fn update_focus(&mut self) {
        self.input.set_focused(self.focused_element == 0);
        self.ok_button.set_focused(self.focused_element == 1);
        self.cancel_button.set_focused(self.focused_element == 2);
    }

    /// Focuses the next element.
    fn focus_next(&mut self) {
        self.focused_element = (self.focused_element + 1) % 3;
        self.update_focus();
    }

    /// Focuses the previous element.
    fn focus_prev(&mut self) {
        self.focused_element = if self.focused_element == 0 {
            2
        } else {
            self.focused_element - 1
        };
        self.update_focus();
    }
}

impl Modal for PromptModal {
    fn focus_ids(&self) -> Vec<FocusId> {
        vec![
            FocusId::new("prompt-input"),
            self.ok_button.id().clone(),
            self.cancel_button.id().clone(),
        ]
    }
}

impl Component for PromptModal {
    type Message = ModalMsg;
    type Action = ModalAction;

    fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
        match msg {
            ModalMsg::Close => {
                if self.config.close_on_escape {
                    Some(ModalAction::Close)
                } else {
                    None
                }
            }
            ModalMsg::Confirm => {
                // If input is focused, submit. If button is focused, handle that button.
                match self.focused_element {
                    0 | 1 => {
                        // Input or OK button: submit the text
                        Some(ModalAction::Submit(self.input.text().to_string()))
                    }
                    2 => {
                        // Cancel button: close
                        Some(ModalAction::Close)
                    }
                    _ => None,
                }
            }
            ModalMsg::FocusNext => {
                self.focus_next();
                None
            }
            ModalMsg::FocusPrev => {
                self.focus_prev();
                None
            }
            ModalMsg::ButtonPressed(1) => {
                // OK button
                Some(ModalAction::Submit(self.input.text().to_string()))
            }
            ModalMsg::ButtonPressed(2) => {
                // Cancel button
                Some(ModalAction::Close)
            }
            ModalMsg::ButtonMsg(1, button_msg) => {
                if let Some(ButtonAction::Pressed) = self.ok_button.update(button_msg) {
                    Some(ModalAction::Submit(self.input.text().to_string()))
                } else {
                    None
                }
            }
            ModalMsg::ButtonMsg(2, button_msg) => {
                if let Some(ButtonAction::Pressed) = self.cancel_button.update(button_msg) {
                    Some(ModalAction::Close)
                } else {
                    None
                }
            }
            ModalMsg::InputMsg(input_msg) => {
                self.input.update(input_msg);
                None
            }
            _ => None,
        }
    }
}

impl Focusable for PromptModal {
    fn is_focused(&self) -> bool {
        self.input.is_focused() || self.ok_button.is_focused() || self.cancel_button.is_focused()
    }

    fn set_focused(&mut self, focused: bool) {
        if focused {
            // Focus the input when modal gains focus
            self.focused_element = 0;
            self.update_focus();
        } else {
            self.input.set_focused(false);
            self.ok_button.set_focused(false);
            self.cancel_button.set_focused(false);
        }
    }
}

impl Renderable for PromptModal {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme.as_ref().cloned().unwrap_or_default();

        // Calculate content height: message + input + button row + spacing
        let message_width = (area.width as f32 * self.config.width_percent) as u16;
        let message_width = message_width.saturating_sub(4); // Account for borders/padding
        let message_lines = (self.message.len() as u16 / message_width.max(1)) + 1;
        let content_height = message_lines + 7; // message + input (3) + spacing + button (3)

        // Render overlay if enabled
        if self.config.show_overlay {
            self.overlay.render(frame, area);
        }

        // Calculate modal area
        let modal_area = calculate_modal_area(area, self.config.width_percent, content_height);

        // Render shadow if enabled
        if self.config.show_shadow {
            self.overlay.render_shadow(frame, modal_area);
        }

        // Render modal background and border
        let block = Block::default()
            .title(self.config.title.as_str())
            .title_style(theme.modal_title_style())
            .borders(Borders::ALL)
            .border_type(theme.components().modal.border_type)
            .border_style(theme.border_focused_style())
            .style(theme.modal_content_style());

        let inner_area = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Layout: message, input, buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(message_lines + 1), // Message area
                Constraint::Length(3),                 // Input area
                Constraint::Length(3),                 // Button area
            ])
            .split(inner_area);

        // Render message
        let message = Paragraph::new(self.message.as_str())
            .style(theme.primary_text_style())
            .wrap(Wrap { trim: true })
            .alignment(if theme.components().modal.center_content {
                Alignment::Center
            } else {
                Alignment::Left
            });

        frame.render_widget(message, chunks[0]);

        // Render input
        self.input.render(frame, chunks[1]);

        // Render buttons (centered, side by side)
        let ok_width = (self.ok_label.len() + 4) as u16;
        let cancel_width = (self.cancel_label.len() + 4) as u16;
        let button_spacing = 2u16;
        let total_button_width = ok_width + button_spacing + cancel_width;

        let buttons_x = chunks[2].x + (chunks[2].width.saturating_sub(total_button_width)) / 2;

        let ok_area = Rect::new(buttons_x, chunks[2].y, ok_width, 3);
        let cancel_area = Rect::new(
            buttons_x + ok_width + button_spacing,
            chunks[2].y,
            cancel_width,
            3,
        );

        self.ok_button.render(frame, ok_area);
        self.cancel_button.render(frame, cancel_area);
    }
}

#[cfg(test)]
mod tests {
    use super::super::ButtonMsg;
    use super::*;
    use crate::components::TextInputMsg;

    #[test]
    fn test_prompt_modal_creation() {
        let modal = PromptModal::new("Prompt", "Enter value:");
        assert_eq!(modal.title(), "Prompt");
        assert_eq!(modal.message(), "Enter value:");
        assert!(modal.config().close_on_escape);
        assert_eq!(modal.focused_element_index(), 0); // Input focused
        assert!(modal.text().is_empty());
    }

    #[test]
    fn test_prompt_modal_with_default() {
        let modal = PromptModal::new("Rename", "New name:").with_default("file.txt");
        assert_eq!(modal.text(), "file.txt");
    }

    #[test]
    fn test_prompt_modal_with_theme() {
        let theme = Theme::dark();
        let modal = PromptModal::new("Test", "Message").with_theme(theme);
        assert!(modal.theme.is_some());
    }

    #[test]
    fn test_prompt_modal_with_labels() {
        let modal = PromptModal::new("Save", "Filename:").with_labels("Save", "Discard");
        assert_eq!(modal.ok_button().label(), "Save");
        assert_eq!(modal.cancel_button().label(), "Discard");
    }

    #[test]
    fn test_prompt_modal_close_on_escape() {
        let mut modal = PromptModal::new("Test", "Message");
        let action = modal.update(ModalMsg::Close);
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_prompt_modal_close_on_escape_disabled() {
        let mut modal = PromptModal::new("Test", "Message").with_close_on_escape(false);
        let action = modal.update(ModalMsg::Close);
        assert!(action.is_none());
    }

    #[test]
    fn test_prompt_modal_submit_from_input() {
        let mut modal = PromptModal::new("Test", "Message").with_default("hello");
        // Input is focused by default
        let action = modal.update(ModalMsg::Confirm);
        assert_eq!(action, Some(ModalAction::Submit("hello".to_string())));
    }

    #[test]
    fn test_prompt_modal_submit_from_ok_button() {
        let mut modal = PromptModal::new("Test", "Message").with_default("hello");
        // Focus OK button
        modal.update(ModalMsg::FocusNext);
        let action = modal.update(ModalMsg::Confirm);
        assert_eq!(action, Some(ModalAction::Submit("hello".to_string())));
    }

    #[test]
    fn test_prompt_modal_cancel_from_button() {
        let mut modal = PromptModal::new("Test", "Message");
        // Focus Cancel button (input -> ok -> cancel)
        modal.update(ModalMsg::FocusNext);
        modal.update(ModalMsg::FocusNext);
        let action = modal.update(ModalMsg::Confirm);
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_prompt_modal_focus_navigation() {
        let mut modal = PromptModal::new("Test", "Message");

        // Initially input is focused
        assert_eq!(modal.focused_element_index(), 0);
        assert!(modal.input().is_focused());
        assert!(!modal.ok_button().is_focused());
        assert!(!modal.cancel_button().is_focused());

        // Focus next (to OK)
        modal.update(ModalMsg::FocusNext);
        assert_eq!(modal.focused_element_index(), 1);
        assert!(!modal.input().is_focused());
        assert!(modal.ok_button().is_focused());
        assert!(!modal.cancel_button().is_focused());

        // Focus next (to Cancel)
        modal.update(ModalMsg::FocusNext);
        assert_eq!(modal.focused_element_index(), 2);
        assert!(!modal.input().is_focused());
        assert!(!modal.ok_button().is_focused());
        assert!(modal.cancel_button().is_focused());

        // Focus next (wraps to input)
        modal.update(ModalMsg::FocusNext);
        assert_eq!(modal.focused_element_index(), 0);
        assert!(modal.input().is_focused());

        // Focus prev (to Cancel)
        modal.update(ModalMsg::FocusPrev);
        assert_eq!(modal.focused_element_index(), 2);
        assert!(modal.cancel_button().is_focused());
    }

    #[test]
    fn test_prompt_modal_focus_ids() {
        let modal = PromptModal::new("Test", "Message");
        let ids = modal.focus_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], FocusId::new("prompt-input"));
        assert_eq!(ids[1], FocusId::new("prompt-ok"));
        assert_eq!(ids[2], FocusId::new("prompt-cancel"));
    }

    #[test]
    fn test_prompt_modal_create_focus_trap() {
        let modal = PromptModal::new("Test", "Message");
        let trap = modal.create_focus_trap();
        assert!(!trap.is_empty());
        assert_eq!(trap.len(), 3);
    }

    #[test]
    fn test_prompt_modal_input_msg() {
        let mut modal = PromptModal::new("Test", "Message");
        modal.update(ModalMsg::InputMsg(TextInputMsg::InsertChar('a')));
        assert_eq!(modal.text(), "a");

        modal.update(ModalMsg::InputMsg(TextInputMsg::InsertChar('b')));
        assert_eq!(modal.text(), "ab");
    }

    #[test]
    fn test_prompt_modal_button_pressed() {
        let mut modal = PromptModal::new("Test", "Message").with_default("test");

        let action = modal.update(ModalMsg::ButtonPressed(1));
        assert_eq!(action, Some(ModalAction::Submit("test".to_string())));

        let action = modal.update(ModalMsg::ButtonPressed(2));
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_prompt_modal_button_msg() {
        let mut modal = PromptModal::new("Test", "Message").with_default("test");

        let action = modal.update(ModalMsg::ButtonMsg(1, ButtonMsg::Press));
        assert_eq!(action, Some(ModalAction::Submit("test".to_string())));

        let action = modal.update(ModalMsg::ButtonMsg(2, ButtonMsg::Press));
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_prompt_modal_focusable() {
        let mut modal = PromptModal::new("Test", "Message");
        assert!(modal.is_focused()); // Input is focused by default

        modal.set_focused(false);
        assert!(!modal.is_focused());

        modal.set_focused(true);
        assert!(modal.is_focused());
        assert_eq!(modal.focused_element_index(), 0); // Reset to input
    }

    #[test]
    fn test_prompt_modal_with_placeholder() {
        let modal = PromptModal::new("Test", "Message").with_placeholder("Enter text...");
        // Placeholder is stored in input, verified during render
        assert!(modal.text().is_empty());
    }
}
