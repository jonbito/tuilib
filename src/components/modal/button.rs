//! Button component for modal dialogs.
//!
//! A simple button component with focus support, used within modal dialogs
//! for actions like OK, Cancel, Yes, No, etc.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::components::{Component, Focusable, Renderable};
use crate::focus::FocusId;
use crate::theme::Theme;

/// Visual variant for buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Default button style.
    #[default]
    Default,
    /// Primary action button (e.g., OK, Submit).
    Primary,
    /// Destructive action button (e.g., Delete).
    Danger,
}

/// Messages that the Button can handle.
#[derive(Debug, Clone)]
pub enum ButtonMsg {
    /// The button was pressed.
    Press,
}

/// Actions that the Button can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ButtonAction {
    /// The button was pressed.
    Pressed,
}

/// A simple button component with focus support.
///
/// # Example
///
/// ```rust
/// use tuilib::components::modal::{Button, ButtonVariant};
///
/// let button = Button::new("ok", "OK")
///     .with_variant(ButtonVariant::Primary);
/// ```
#[derive(Debug, Clone)]
pub struct Button {
    /// Unique identifier for this button.
    id: FocusId,
    /// Button label text.
    label: String,
    /// Visual variant.
    variant: ButtonVariant,
    /// Whether the button is focused.
    focused: bool,
    /// Whether the button is disabled.
    disabled: bool,
    /// Optional theme for styling.
    theme: Option<Theme>,
}

impl Button {
    /// Creates a new button with the given ID and label.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for focus management
    /// * `label` - Text displayed on the button
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: FocusId::from(id.into()),
            label: label.into(),
            variant: ButtonVariant::Default,
            focused: false,
            disabled: false,
            theme: None,
        }
    }

    /// Sets the button variant.
    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the theme for styling.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets whether the button is disabled.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the button's focus ID.
    pub fn id(&self) -> &FocusId {
        &self.id
    }

    /// Returns the button's label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the button's variant.
    pub fn variant(&self) -> ButtonVariant {
        self.variant
    }

    /// Returns whether the button is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

impl Component for Button {
    type Message = ButtonMsg;
    type Action = ButtonAction;

    fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
        if self.disabled {
            return None;
        }

        match msg {
            ButtonMsg::Press => Some(ButtonAction::Pressed),
        }
    }
}

impl Focusable for Button {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        !self.disabled
    }
}

impl Renderable for Button {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme.as_ref().cloned().unwrap_or_default();

        // Determine style based on state
        let (text_style, border_style) = if self.disabled {
            (theme.button_disabled_style(), theme.border_style())
        } else if self.focused {
            let text_style = match self.variant {
                ButtonVariant::Primary => theme.button_focused_style(),
                ButtonVariant::Danger => theme.button_focused_style().fg(theme.colors().error),
                ButtonVariant::Default => theme.button_focused_style(),
            };
            (text_style, theme.border_focused_style())
        } else {
            let text_style = match self.variant {
                ButtonVariant::Primary => theme.button_normal_style(),
                ButtonVariant::Danger => theme.button_normal_style().fg(theme.colors().error),
                ButtonVariant::Default => theme.button_normal_style(),
            };
            (text_style, theme.border_style())
        };

        // Build block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(theme.components().button.border_type)
            .border_style(border_style);

        // Create paragraph with centered text
        let paragraph = Paragraph::new(self.label.as_str())
            .style(text_style)
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("test-btn", "Click Me");
        assert_eq!(button.id(), &FocusId::new("test-btn"));
        assert_eq!(button.label(), "Click Me");
        assert_eq!(button.variant(), ButtonVariant::Default);
        assert!(!button.is_focused());
        assert!(!button.is_disabled());
    }

    #[test]
    fn test_button_with_variant() {
        let button = Button::new("btn", "OK").with_variant(ButtonVariant::Primary);
        assert_eq!(button.variant(), ButtonVariant::Primary);
    }

    #[test]
    fn test_button_with_disabled() {
        let button = Button::new("btn", "Submit").with_disabled(true);
        assert!(button.is_disabled());
    }

    #[test]
    fn test_button_update() {
        let mut button = Button::new("btn", "OK");
        let action = button.update(ButtonMsg::Press);
        assert_eq!(action, Some(ButtonAction::Pressed));
    }

    #[test]
    fn test_disabled_button_no_action() {
        let mut button = Button::new("btn", "OK").with_disabled(true);
        let action = button.update(ButtonMsg::Press);
        assert!(action.is_none());
    }

    #[test]
    fn test_button_focus() {
        let mut button = Button::new("btn", "OK");
        assert!(!button.is_focused());
        assert!(button.can_focus());

        button.set_focused(true);
        assert!(button.is_focused());
    }

    #[test]
    fn test_disabled_button_cannot_focus() {
        let button = Button::new("btn", "OK").with_disabled(true);
        assert!(!button.can_focus());
    }

    #[test]
    fn test_button_set_disabled() {
        let mut button = Button::new("btn", "OK");
        assert!(!button.is_disabled());

        button.set_disabled(true);
        assert!(button.is_disabled());
    }
}
