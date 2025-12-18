//! Color palette definitions for themes.
//!
//! This module provides the [`ColorPalette`] struct which defines semantic color roles
//! that can be used consistently across all components.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// A complete color palette for a theme.
///
/// The palette defines semantic color roles rather than specific colors,
/// allowing components to use colors consistently regardless of the
/// active theme.
///
/// # Semantic Colors
///
/// - **Primary**: The main brand/accent color for interactive elements
/// - **Secondary**: A complementary color for secondary actions
/// - **Background**: The main background color
/// - **Surface**: Elevated surface color (cards, modals, etc.)
/// - **Error/Warning/Success/Info**: Status colors for feedback
/// - **Text variants**: Different text emphasis levels
/// - **Border variants**: Default and focused border colors
///
/// # Example
///
/// ```rust
/// use tuilib::theme::ColorPalette;
/// use ratatui::style::Color;
///
/// let palette = ColorPalette::dark();
/// assert_eq!(palette.background, Color::Rgb(30, 30, 46));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Primary accent color for interactive elements
    pub primary: Color,
    /// Secondary accent color for complementary actions
    pub secondary: Color,
    /// Main background color
    pub background: Color,
    /// Elevated surface color (cards, modals)
    pub surface: Color,
    /// Error state color
    pub error: Color,
    /// Warning state color
    pub warning: Color,
    /// Success state color
    pub success: Color,
    /// Information state color
    pub info: Color,
    /// Primary text color (highest contrast)
    pub text_primary: Color,
    /// Secondary text color (medium contrast)
    pub text_secondary: Color,
    /// Disabled text color (lowest contrast)
    pub text_disabled: Color,
    /// Default border color
    pub border: Color,
    /// Focused element border color
    pub border_focused: Color,
}

impl ColorPalette {
    /// Creates a new color palette with custom colors.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        primary: Color,
        secondary: Color,
        background: Color,
        surface: Color,
        error: Color,
        warning: Color,
        success: Color,
        info: Color,
        text_primary: Color,
        text_secondary: Color,
        text_disabled: Color,
        border: Color,
        border_focused: Color,
    ) -> Self {
        Self {
            primary,
            secondary,
            background,
            surface,
            error,
            warning,
            success,
            info,
            text_primary,
            text_secondary,
            text_disabled,
            border,
            border_focused,
        }
    }

    /// Creates the default dark color palette.
    ///
    /// Uses a modern dark theme inspired by popular color schemes
    /// with good contrast ratios for accessibility.
    pub fn dark() -> Self {
        Self {
            // Catppuccin Mocha-inspired palette
            primary: Color::Rgb(137, 180, 250),        // Blue
            secondary: Color::Rgb(203, 166, 247),      // Mauve
            background: Color::Rgb(30, 30, 46),        // Base
            surface: Color::Rgb(49, 50, 68),           // Surface0
            error: Color::Rgb(243, 139, 168),          // Red
            warning: Color::Rgb(249, 226, 175),        // Yellow
            success: Color::Rgb(166, 227, 161),        // Green
            info: Color::Rgb(137, 220, 235),           // Sky
            text_primary: Color::Rgb(205, 214, 244),   // Text
            text_secondary: Color::Rgb(166, 173, 200), // Subtext0
            text_disabled: Color::Rgb(108, 112, 134),  // Overlay0
            border: Color::Rgb(69, 71, 90),            // Surface1
            border_focused: Color::Rgb(137, 180, 250), // Blue
        }
    }

    /// Creates the default light color palette.
    ///
    /// Uses a modern light theme with good contrast ratios
    /// for accessibility and readability.
    pub fn light() -> Self {
        Self {
            // Catppuccin Latte-inspired palette
            primary: Color::Rgb(30, 102, 245),         // Blue
            secondary: Color::Rgb(136, 57, 239),       // Mauve
            background: Color::Rgb(239, 241, 245),     // Base
            surface: Color::Rgb(220, 224, 232),        // Surface0
            error: Color::Rgb(210, 15, 57),            // Red
            warning: Color::Rgb(223, 142, 29),         // Yellow
            success: Color::Rgb(64, 160, 43),          // Green
            info: Color::Rgb(4, 165, 229),             // Sky
            text_primary: Color::Rgb(76, 79, 105),     // Text
            text_secondary: Color::Rgb(108, 111, 133), // Subtext0
            text_disabled: Color::Rgb(156, 160, 176),  // Overlay0
            border: Color::Rgb(188, 192, 204),         // Surface1
            border_focused: Color::Rgb(30, 102, 245),  // Blue
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_palette_creation() {
        let palette = ColorPalette::dark();
        assert_eq!(palette.background, Color::Rgb(30, 30, 46));
        assert_eq!(palette.text_primary, Color::Rgb(205, 214, 244));
    }

    #[test]
    fn test_light_palette_creation() {
        let palette = ColorPalette::light();
        assert_eq!(palette.background, Color::Rgb(239, 241, 245));
        assert_eq!(palette.text_primary, Color::Rgb(76, 79, 105));
    }

    #[test]
    fn test_default_is_dark() {
        let default = ColorPalette::default();
        let dark = ColorPalette::dark();
        assert_eq!(default, dark);
    }

    #[test]
    fn test_custom_palette() {
        let palette = ColorPalette::new(
            Color::Red,
            Color::Blue,
            Color::Black,
            Color::DarkGray,
            Color::LightRed,
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::White,
            Color::Gray,
            Color::DarkGray,
            Color::Gray,
            Color::White,
        );
        assert_eq!(palette.primary, Color::Red);
        assert_eq!(palette.background, Color::Black);
    }

    #[test]
    fn test_palette_clone() {
        let palette1 = ColorPalette::dark();
        let palette2 = palette1.clone();
        assert_eq!(palette1, palette2);
    }
}
