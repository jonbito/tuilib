//! Style definitions for borders and text.
//!
//! This module provides [`BorderStyles`] and [`TextStyles`] structs for
//! configuring visual styling of borders and text across components.

use ratatui::style::{Modifier, Style};
use ratatui::widgets::BorderType;

/// Border style configuration for themed components.
///
/// Defines the border types used in different component states and contexts.
///
/// # Example
///
/// ```rust
/// use tuilib::theme::BorderStyles;
/// use ratatui::widgets::BorderType;
///
/// let borders = BorderStyles::default();
/// assert_eq!(borders.default, BorderType::Rounded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderStyles {
    /// Default border style for unfocused elements
    pub default: BorderType,
    /// Border style for focused elements
    pub focused: BorderType,
    /// Border style for modal dialogs
    pub modal: BorderType,
    /// Border style for disabled elements
    pub disabled: BorderType,
}

impl BorderStyles {
    /// Creates a new border styles configuration.
    pub fn new(
        default: BorderType,
        focused: BorderType,
        modal: BorderType,
        disabled: BorderType,
    ) -> Self {
        Self {
            default,
            focused,
            modal,
            disabled,
        }
    }

    /// Creates a modern border style configuration with rounded corners.
    pub fn modern() -> Self {
        Self {
            default: BorderType::Rounded,
            focused: BorderType::Rounded,
            modal: BorderType::Double,
            disabled: BorderType::Plain,
        }
    }

    /// Creates a classic border style configuration with plain corners.
    pub fn classic() -> Self {
        Self {
            default: BorderType::Plain,
            focused: BorderType::Plain,
            modal: BorderType::Double,
            disabled: BorderType::Plain,
        }
    }

    /// Creates a minimal border style configuration.
    pub fn minimal() -> Self {
        Self {
            default: BorderType::Plain,
            focused: BorderType::Plain,
            modal: BorderType::Plain,
            disabled: BorderType::Plain,
        }
    }
}

impl Default for BorderStyles {
    fn default() -> Self {
        Self::modern()
    }
}

/// Text style configuration for themed components.
///
/// Provides pre-configured styles for different text elements
/// like headings, body text, and emphasis.
///
/// # Example
///
/// ```rust
/// use tuilib::theme::TextStyles;
/// use ratatui::style::{Modifier, Style};
///
/// let text = TextStyles::default();
/// assert!(text.heading.add_modifier.contains(Modifier::BOLD));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyles {
    /// Style for main headings
    pub heading: TextStyle,
    /// Style for subheadings
    pub subheading: TextStyle,
    /// Style for body text
    pub body: TextStyle,
    /// Style for emphasized text
    pub emphasis: TextStyle,
    /// Style for strong/important text
    pub strong: TextStyle,
    /// Style for code/monospace text
    pub code: TextStyle,
    /// Style for muted/secondary text
    pub muted: TextStyle,
}

impl TextStyles {
    /// Creates a new text styles configuration.
    pub fn new(
        heading: TextStyle,
        subheading: TextStyle,
        body: TextStyle,
        emphasis: TextStyle,
        strong: TextStyle,
        code: TextStyle,
        muted: TextStyle,
    ) -> Self {
        Self {
            heading,
            subheading,
            body,
            emphasis,
            strong,
            code,
            muted,
        }
    }
}

impl Default for TextStyles {
    fn default() -> Self {
        Self {
            heading: TextStyle::new().bold(),
            subheading: TextStyle::new().bold(),
            body: TextStyle::new(),
            emphasis: TextStyle::new().italic(),
            strong: TextStyle::new().bold(),
            code: TextStyle::new(),
            muted: TextStyle::new().dim(),
        }
    }
}

/// Individual text style configuration.
///
/// Represents text modifiers that can be applied to text elements.
/// This is a simplified representation that can be converted to
/// ratatui's [`Style`].
///
/// # Example
///
/// ```rust
/// use tuilib::theme::TextStyle;
/// use ratatui::style::{Modifier, Style};
///
/// let style = TextStyle::new().bold().italic();
/// let ratatui_style: Style = style.into();
/// assert!(ratatui_style.add_modifier.contains(Modifier::BOLD));
/// assert!(ratatui_style.add_modifier.contains(Modifier::ITALIC));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    /// Modifiers to add to the text
    pub add_modifier: Modifier,
}

impl TextStyle {
    /// Creates a new empty text style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds bold modifier to the style.
    pub fn bold(mut self) -> Self {
        self.add_modifier |= Modifier::BOLD;
        self
    }

    /// Adds italic modifier to the style.
    pub fn italic(mut self) -> Self {
        self.add_modifier |= Modifier::ITALIC;
        self
    }

    /// Adds underline modifier to the style.
    pub fn underline(mut self) -> Self {
        self.add_modifier |= Modifier::UNDERLINED;
        self
    }

    /// Adds dim modifier to the style.
    pub fn dim(mut self) -> Self {
        self.add_modifier |= Modifier::DIM;
        self
    }

    /// Adds crossed out modifier to the style.
    pub fn crossed_out(mut self) -> Self {
        self.add_modifier |= Modifier::CROSSED_OUT;
        self
    }

    /// Adds reversed (inverted colors) modifier to the style.
    pub fn reversed(mut self) -> Self {
        self.add_modifier |= Modifier::REVERSED;
        self
    }

    /// Converts this text style to a ratatui [`Style`].
    pub fn to_style(self) -> Style {
        Style::default().add_modifier(self.add_modifier)
    }
}

impl From<TextStyle> for Style {
    fn from(text_style: TextStyle) -> Self {
        text_style.to_style()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_styles_default() {
        let borders = BorderStyles::default();
        assert_eq!(borders.default, BorderType::Rounded);
        assert_eq!(borders.focused, BorderType::Rounded);
        assert_eq!(borders.modal, BorderType::Double);
    }

    #[test]
    fn test_border_styles_modern() {
        let borders = BorderStyles::modern();
        assert_eq!(borders, BorderStyles::default());
    }

    #[test]
    fn test_border_styles_classic() {
        let borders = BorderStyles::classic();
        assert_eq!(borders.default, BorderType::Plain);
        assert_eq!(borders.modal, BorderType::Double);
    }

    #[test]
    fn test_border_styles_minimal() {
        let borders = BorderStyles::minimal();
        assert_eq!(borders.default, BorderType::Plain);
        assert_eq!(borders.modal, BorderType::Plain);
    }

    #[test]
    fn test_text_style_modifiers() {
        let style = TextStyle::new().bold().italic().underline();
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn test_text_style_to_ratatui_style() {
        let text_style = TextStyle::new().bold();
        let style: Style = text_style.into();
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_text_styles_default() {
        let styles = TextStyles::default();
        assert!(styles.heading.add_modifier.contains(Modifier::BOLD));
        assert!(styles.emphasis.add_modifier.contains(Modifier::ITALIC));
        assert!(styles.muted.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn test_text_style_dim() {
        let style = TextStyle::new().dim();
        assert!(style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn test_text_style_chaining() {
        let style = TextStyle::new()
            .bold()
            .italic()
            .underline()
            .dim()
            .crossed_out()
            .reversed();

        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
        assert!(style.add_modifier.contains(Modifier::DIM));
        assert!(style.add_modifier.contains(Modifier::CROSSED_OUT));
        assert!(style.add_modifier.contains(Modifier::REVERSED));
    }
}
