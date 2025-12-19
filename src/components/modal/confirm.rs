//! Confirm modal dialog.
//!
//! A modal for getting yes/no confirmation from the user.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{
    calculate_modal_area, Button, ButtonAction, ButtonVariant, Modal, ModalAction, ModalConfig,
    ModalMsg, Overlay,
};
use crate::components::{Component, Focusable, Renderable};
use crate::focus::FocusId;
use crate::theme::Theme;

/// A confirm modal dialog with Yes/No buttons.
///
/// Confirm modals are used to get user confirmation before performing
/// an action. Returns `true` if the user confirms, `false` if they cancel.
///
/// # Example
///
/// ```rust
/// use tuilib::components::Component;
/// use tuilib::components::modal::{ConfirmModal, ModalMsg, ModalAction};
///
/// let mut modal = ConfirmModal::new("Confirm Delete", "Are you sure you want to delete this file?");
///
/// // Handle user confirmation
/// match modal.update(ModalMsg::Confirm) {
///     Some(ModalAction::Confirm(true)) => {
///         // User confirmed (pressed Yes)
///     }
///     Some(ModalAction::Confirm(false)) | Some(ModalAction::Close) => {
///         // User cancelled
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConfirmModal {
    /// Modal configuration.
    config: ModalConfig,
    /// The message to display.
    message: String,
    /// The Yes button.
    yes_button: Button,
    /// The No button.
    no_button: Button,
    /// Index of the currently focused button (0 = Yes, 1 = No).
    focused_button: usize,
    /// Optional theme for styling.
    theme: Option<Theme>,
    /// Overlay for background dimming.
    overlay: Overlay,
    /// Custom labels for buttons.
    yes_label: String,
    no_label: String,
}

impl ConfirmModal {
    /// Creates a new confirm modal with the given title and message.
    ///
    /// # Arguments
    ///
    /// * `title` - Title displayed at the top of the modal
    /// * `message` - Message content displayed in the modal body
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let config = ModalConfig::new(title);

        let mut yes_button = Button::new("confirm-yes", "Yes").with_variant(ButtonVariant::Primary);
        yes_button.set_focused(true);

        let no_button = Button::new("confirm-no", "No").with_variant(ButtonVariant::Default);

        Self {
            config,
            message: message.into(),
            yes_button,
            no_button,
            focused_button: 0,
            theme: None,
            overlay: Overlay::new().with_shadow(true),
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
        }
    }

    /// Sets the theme for styling.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.yes_button = self.yes_button.with_theme(theme.clone());
        self.no_button = self.no_button.with_theme(theme.clone());
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

    /// Sets custom button labels.
    ///
    /// # Arguments
    ///
    /// * `yes_label` - Label for the confirmation button (default: "Yes")
    /// * `no_label` - Label for the cancellation button (default: "No")
    pub fn with_labels(
        mut self,
        yes_label: impl Into<String>,
        no_label: impl Into<String>,
    ) -> Self {
        self.yes_label = yes_label.into();
        self.no_label = no_label.into();
        self.yes_button =
            Button::new("confirm-yes", self.yes_label.clone()).with_variant(ButtonVariant::Primary);
        self.no_button =
            Button::new("confirm-no", self.no_label.clone()).with_variant(ButtonVariant::Default);

        // Preserve theme and focus state
        if let Some(ref theme) = self.theme {
            self.yes_button = self.yes_button.with_theme(theme.clone());
            self.no_button = self.no_button.with_theme(theme.clone());
        }
        self.update_focus();
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

    /// Returns a reference to the Yes button.
    pub fn yes_button(&self) -> &Button {
        &self.yes_button
    }

    /// Returns a reference to the No button.
    pub fn no_button(&self) -> &Button {
        &self.no_button
    }

    /// Returns the index of the currently focused button.
    pub fn focused_button_index(&self) -> usize {
        self.focused_button
    }

    /// Returns the modal configuration.
    pub fn config(&self) -> &ModalConfig {
        &self.config
    }

    /// Updates the focus state of buttons based on focused_button index.
    fn update_focus(&mut self) {
        self.yes_button.set_focused(self.focused_button == 0);
        self.no_button.set_focused(self.focused_button == 1);
    }

    /// Focuses the next button.
    fn focus_next(&mut self) {
        self.focused_button = (self.focused_button + 1) % 2;
        self.update_focus();
    }

    /// Focuses the previous button.
    fn focus_prev(&mut self) {
        self.focused_button = if self.focused_button == 0 { 1 } else { 0 };
        self.update_focus();
    }
}

impl Modal for ConfirmModal {
    fn focus_ids(&self) -> Vec<FocusId> {
        vec![self.yes_button.id().clone(), self.no_button.id().clone()]
    }
}

impl Component for ConfirmModal {
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
                // Confirm the currently focused button
                if self.focused_button == 0 {
                    Some(ModalAction::Confirm(true))
                } else {
                    Some(ModalAction::Confirm(false))
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
            ModalMsg::ButtonPressed(0) => Some(ModalAction::Confirm(true)),
            ModalMsg::ButtonPressed(1) => Some(ModalAction::Confirm(false)),
            ModalMsg::ButtonMsg(0, button_msg) => {
                if let Some(ButtonAction::Pressed) = self.yes_button.update(button_msg) {
                    Some(ModalAction::Confirm(true))
                } else {
                    None
                }
            }
            ModalMsg::ButtonMsg(1, button_msg) => {
                if let Some(ButtonAction::Pressed) = self.no_button.update(button_msg) {
                    Some(ModalAction::Confirm(false))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Focusable for ConfirmModal {
    fn is_focused(&self) -> bool {
        self.yes_button.is_focused() || self.no_button.is_focused()
    }

    fn set_focused(&mut self, focused: bool) {
        if focused {
            // Focus the first button when modal gains focus
            self.focused_button = 0;
            self.update_focus();
        } else {
            self.yes_button.set_focused(false);
            self.no_button.set_focused(false);
        }
    }
}

impl Renderable for ConfirmModal {
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

        // Render buttons (centered, side by side)
        let yes_width = (self.yes_label.len() + 4) as u16;
        let no_width = (self.no_label.len() + 4) as u16;
        let button_spacing = 2u16;
        let total_button_width = yes_width + button_spacing + no_width;

        let buttons_x = chunks[1].x + (chunks[1].width.saturating_sub(total_button_width)) / 2;

        let yes_area = Rect::new(buttons_x, chunks[1].y, yes_width, 3);
        let no_area = Rect::new(
            buttons_x + yes_width + button_spacing,
            chunks[1].y,
            no_width,
            3,
        );

        self.yes_button.render(frame, yes_area);
        self.no_button.render(frame, no_area);
    }
}

#[cfg(test)]
mod tests {
    use super::super::ButtonMsg;
    use super::*;

    #[test]
    fn test_confirm_modal_creation() {
        let modal = ConfirmModal::new("Confirm", "Are you sure?");
        assert_eq!(modal.title(), "Confirm");
        assert_eq!(modal.message(), "Are you sure?");
        assert!(modal.config().close_on_escape);
        assert_eq!(modal.focused_button_index(), 0);
    }

    #[test]
    fn test_confirm_modal_with_theme() {
        let theme = Theme::dark();
        let modal = ConfirmModal::new("Test", "Message").with_theme(theme);
        assert!(modal.theme.is_some());
    }

    #[test]
    fn test_confirm_modal_with_labels() {
        let modal = ConfirmModal::new("Delete", "Delete file?").with_labels("Delete", "Keep");
        assert_eq!(modal.yes_button().label(), "Delete");
        assert_eq!(modal.no_button().label(), "Keep");
    }

    #[test]
    fn test_confirm_modal_close_on_escape() {
        let mut modal = ConfirmModal::new("Test", "Message");
        let action = modal.update(ModalMsg::Close);
        assert_eq!(action, Some(ModalAction::Close));
    }

    #[test]
    fn test_confirm_modal_close_on_escape_disabled() {
        let mut modal = ConfirmModal::new("Test", "Message").with_close_on_escape(false);
        let action = modal.update(ModalMsg::Close);
        assert!(action.is_none());
    }

    #[test]
    fn test_confirm_modal_confirm_yes() {
        let mut modal = ConfirmModal::new("Test", "Message");
        // Yes button is focused by default
        let action = modal.update(ModalMsg::Confirm);
        assert_eq!(action, Some(ModalAction::Confirm(true)));
    }

    #[test]
    fn test_confirm_modal_confirm_no() {
        let mut modal = ConfirmModal::new("Test", "Message");
        // Focus the No button
        modal.update(ModalMsg::FocusNext);
        let action = modal.update(ModalMsg::Confirm);
        assert_eq!(action, Some(ModalAction::Confirm(false)));
    }

    #[test]
    fn test_confirm_modal_button_pressed() {
        let mut modal = ConfirmModal::new("Test", "Message");

        let action = modal.update(ModalMsg::ButtonPressed(0));
        assert_eq!(action, Some(ModalAction::Confirm(true)));

        let action = modal.update(ModalMsg::ButtonPressed(1));
        assert_eq!(action, Some(ModalAction::Confirm(false)));
    }

    #[test]
    fn test_confirm_modal_focus_navigation() {
        let mut modal = ConfirmModal::new("Test", "Message");

        // Initially Yes is focused
        assert_eq!(modal.focused_button_index(), 0);
        assert!(modal.yes_button().is_focused());
        assert!(!modal.no_button().is_focused());

        // Focus next (to No)
        modal.update(ModalMsg::FocusNext);
        assert_eq!(modal.focused_button_index(), 1);
        assert!(!modal.yes_button().is_focused());
        assert!(modal.no_button().is_focused());

        // Focus next again (wraps to Yes)
        modal.update(ModalMsg::FocusNext);
        assert_eq!(modal.focused_button_index(), 0);
        assert!(modal.yes_button().is_focused());

        // Focus prev (to No)
        modal.update(ModalMsg::FocusPrev);
        assert_eq!(modal.focused_button_index(), 1);
        assert!(modal.no_button().is_focused());
    }

    #[test]
    fn test_confirm_modal_focus_ids() {
        let modal = ConfirmModal::new("Test", "Message");
        let ids = modal.focus_ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], FocusId::new("confirm-yes"));
        assert_eq!(ids[1], FocusId::new("confirm-no"));
    }

    #[test]
    fn test_confirm_modal_create_focus_trap() {
        let modal = ConfirmModal::new("Test", "Message");
        let trap = modal.create_focus_trap();
        assert!(!trap.is_empty());
        assert_eq!(trap.len(), 2);
    }

    #[test]
    fn test_confirm_modal_focusable() {
        let mut modal = ConfirmModal::new("Test", "Message");
        assert!(modal.is_focused()); // Yes button is focused by default

        modal.set_focused(false);
        assert!(!modal.is_focused());

        modal.set_focused(true);
        assert!(modal.is_focused());
        assert_eq!(modal.focused_button_index(), 0); // Reset to first button
    }

    #[test]
    fn test_confirm_modal_button_msg() {
        let mut modal = ConfirmModal::new("Test", "Message");

        let action = modal.update(ModalMsg::ButtonMsg(0, ButtonMsg::Press));
        assert_eq!(action, Some(ModalAction::Confirm(true)));

        let action = modal.update(ModalMsg::ButtonMsg(1, ButtonMsg::Press));
        assert_eq!(action, Some(ModalAction::Confirm(false)));
    }
}
