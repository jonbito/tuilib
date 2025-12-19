//! Alert modal dialog.
//!
//! A simple modal for displaying information to the user with an OK button.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{
    calculate_modal_area, Button, ButtonAction, ButtonVariant, Modal, ModalAction, ModalConfig,
    ModalMsg, Overlay,
};
use crate::components::{Component, Focusable, Renderable};
use crate::focus::FocusId;
use crate::theme::Theme;

/// An alert modal dialog with a message and OK button.
///
/// Alert modals are used to display information to the user that requires
/// acknowledgment. The modal can be dismissed by clicking OK, pressing Enter,
/// or pressing Escape (if configured).
///
/// # Example
///
/// ```rust
/// use tuilib::components::Component;
/// use tuilib::components::modal::{AlertModal, ModalMsg, ModalAction};
///
/// let mut modal = AlertModal::new("Success", "Your changes have been saved.");
///
/// // Handle escape key
/// if let Some(ModalAction::Close) = modal.update(ModalMsg::Close) {
///     // Modal was dismissed
/// }
///
/// // Handle enter key (confirm)
/// if let Some(ModalAction::Close) = modal.update(ModalMsg::Confirm) {
///     // Modal was confirmed
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AlertModal {
    /// Modal configuration.
    config: ModalConfig,
    /// The message to display.
    message: String,
    /// The OK button.
    ok_button: Button,
    /// Optional theme for styling.
    theme: Option<Theme>,
    /// Overlay for background dimming.
    overlay: Overlay,
}

impl AlertModal {
    /// Creates a new alert modal with the given title and message.
    ///
    /// # Arguments
    ///
    /// * `title` - Title displayed at the top of the modal
    /// * `message` - Message content displayed in the modal body
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let config = ModalConfig::new(title);

        let mut ok_button = Button::new("alert-ok", "OK").with_variant(ButtonVariant::Primary);
        ok_button.set_focused(true);

        Self {
            config,
            message: message.into(),
            ok_button,
            theme: None,
            overlay: Overlay::new().with_shadow(true),
        }
    }

    /// Sets the theme for styling.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.ok_button = self.ok_button.with_theme(theme.clone());
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

    /// Returns the modal title.
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Returns the modal message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns a reference to the OK button.
    pub fn ok_button(&self) -> &Button {
        &self.ok_button
    }

    /// Returns the modal configuration.
    pub fn config(&self) -> &ModalConfig {
        &self.config
    }
}

impl Modal for AlertModal {
    fn focus_ids(&self) -> Vec<FocusId> {
        vec![self.ok_button.id().clone()]
    }
}

impl Component for AlertModal {
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
            ModalMsg::Confirm | ModalMsg::ButtonPressed(0) => Some(ModalAction::Close),
            ModalMsg::ButtonMsg(0, button_msg) => {
                if let Some(ButtonAction::Pressed) = self.ok_button.update(button_msg) {
                    Some(ModalAction::Close)
                } else {
                    None
                }
            }
            // AlertModal has only one button, focus navigation is a no-op
            ModalMsg::FocusNext | ModalMsg::FocusPrev => None,
            _ => None,
        }
    }
}

impl Focusable for AlertModal {
    fn is_focused(&self) -> bool {
        self.ok_button.is_focused()
    }

    fn set_focused(&mut self, focused: bool) {
        self.ok_button.set_focused(focused);
    }
}

impl Renderable for AlertModal {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme.as_ref().cloned().unwrap_or_default();

        // Calculate content height: message lines + button row + spacing
        let message_width = (area.width as f32 * self.config.width_percent) as u16;
        let message_width = message_width.saturating_sub(4); // Account for borders/padding
        let message_lines = (self.message.len() as u16 / message_width.max(1)) + 1;
        let content_height = message_lines + 4; // message + spacing + button (3 high) + spacing

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

        // Layout: message area and button area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Message area
                Constraint::Length(3), // Button area
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

        // Render button (centered)
        let button_width = (self.ok_button.label().len() + 4) as u16;
        let button_x = chunks[1].x + (chunks[1].width.saturating_sub(button_width)) / 2;
        let button_area = Rect::new(button_x, chunks[1].y, button_width, 3);

        self.ok_button.render(frame, button_area);
    }
}

#[cfg(test)]
mod tests {
    use super::super::ButtonMsg;
    use super::*;

    #[test]
    fn test_alert_modal_creation() {
        let modal = AlertModal::new("Test Title", "Test message");
        assert_eq!(modal.title(), "Test Title");
        assert_eq!(modal.message(), "Test message");
        assert!(modal.config().close_on_escape);
    }

    #[test]
    fn test_alert_modal_with_theme() {
        let theme = Theme::dark();
        let modal = AlertModal::new("Test", "Message").with_theme(theme);
        assert!(modal.theme.is_some());
    }

    #[test]
    fn test_alert_modal_close_on_escape() {
        let mut modal = AlertModal::new("Test", "Message");
        let action = modal.update(ModalMsg::Close);
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_alert_modal_close_on_escape_disabled() {
        let mut modal = AlertModal::new("Test", "Message").with_close_on_escape(false);
        let action = modal.update(ModalMsg::Close);
        assert!(action.is_none());
    }

    #[test]
    fn test_alert_modal_confirm() {
        let mut modal = AlertModal::new("Test", "Message");
        let action = modal.update(ModalMsg::Confirm);
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_alert_modal_button_pressed() {
        let mut modal = AlertModal::new("Test", "Message");
        let action = modal.update(ModalMsg::ButtonPressed(0));
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_alert_modal_button_msg() {
        let mut modal = AlertModal::new("Test", "Message");
        let action = modal.update(ModalMsg::ButtonMsg(0, ButtonMsg::Press));
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_alert_modal_focus_ids() {
        let modal = AlertModal::new("Test", "Message");
        let ids = modal.focus_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], FocusId::new("alert-ok"));
    }

    #[test]
    fn test_alert_modal_create_focus_trap() {
        let modal = AlertModal::new("Test", "Message");
        let trap = modal.create_focus_trap();
        assert!(!trap.is_empty());
        assert_eq!(trap.len(), 1);
    }

    #[test]
    fn test_alert_modal_focusable() {
        let mut modal = AlertModal::new("Test", "Message");
        assert!(modal.is_focused()); // OK button is focused by default

        modal.set_focused(false);
        assert!(!modal.is_focused());
    }

    #[test]
    fn test_alert_modal_builder_methods() {
        let modal = AlertModal::new("Test", "Message")
            .with_width_percent(0.8)
            .with_overlay(false)
            .with_shadow(false);

        assert!((modal.config().width_percent - 0.8).abs() < 0.01);
        assert!(!modal.config().show_overlay);
        assert!(!modal.config().show_shadow);
    }
}
