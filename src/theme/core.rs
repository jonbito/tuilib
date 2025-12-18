//! Main theme struct and implementation.
//!
//! This module provides the [`Theme`] struct which combines all styling
//! elements into a cohesive theme definition.

use ratatui::style::{Color, Modifier, Style};

use super::builder::ThemeBuilder;
use super::colors::ColorPalette;
use super::component::{ComponentStyles, ComputedStyle};
use super::styles::{BorderStyles, TextStyles};

/// A complete theme definition.
///
/// Themes combine color palettes, border styles, text styles, and
/// component-specific configurations into a cohesive visual identity.
///
/// # Creating Themes
///
/// Use the provided factory methods for built-in themes:
///
/// ```rust
/// use tuilib::theme::Theme;
///
/// let dark = Theme::dark();
/// let light = Theme::light();
/// ```
///
/// Or use the builder for custom themes:
///
/// ```rust
/// use tuilib::theme::Theme;
/// use ratatui::style::Color;
///
/// let custom = Theme::builder()
///     .name("My Theme")
///     .dark_base()
///     .primary_color(Color::Cyan)
///     .build();
/// ```
///
/// # Accessing Styles
///
/// Themes provide convenience methods for computing styles:
///
/// ```rust
/// use tuilib::theme::Theme;
///
/// let theme = Theme::dark();
///
/// // Get primary text style
/// let style = theme.primary_text_style();
///
/// // Get styles for different states
/// let normal = theme.button_normal_style();
/// let focused = theme.button_focused_style();
/// ```
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name for identification
    name: String,
    /// Color palette
    colors: ColorPalette,
    /// Border style configuration
    borders: BorderStyles,
    /// Text style configuration
    text: TextStyles,
    /// Component-specific styles
    components: ComponentStyles,
}

impl Theme {
    /// Creates a new theme with the given configuration.
    pub fn new(
        name: impl Into<String>,
        colors: ColorPalette,
        borders: BorderStyles,
        text: TextStyles,
        components: ComponentStyles,
    ) -> Self {
        Self {
            name: name.into(),
            colors,
            borders,
            text,
            components,
        }
    }

    /// Creates a theme builder for constructing custom themes.
    pub fn builder() -> ThemeBuilder {
        ThemeBuilder::new()
    }

    /// Creates the default dark theme.
    ///
    /// Uses a modern dark color palette with good contrast ratios.
    pub fn dark() -> Self {
        Self::new(
            "Dark",
            ColorPalette::dark(),
            BorderStyles::modern(),
            TextStyles::default(),
            ComponentStyles::default(),
        )
    }

    /// Creates the default light theme.
    ///
    /// Uses a modern light color palette with good contrast ratios.
    pub fn light() -> Self {
        Self::new(
            "Light",
            ColorPalette::light(),
            BorderStyles::modern(),
            TextStyles::default(),
            ComponentStyles::default(),
        )
    }

    /// Returns the theme name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the color palette.
    pub fn colors(&self) -> &ColorPalette {
        &self.colors
    }

    /// Returns the border styles.
    pub fn borders(&self) -> &BorderStyles {
        &self.borders
    }

    /// Returns the text styles.
    pub fn text(&self) -> &TextStyles {
        &self.text
    }

    /// Returns the component styles.
    pub fn components(&self) -> &ComponentStyles {
        &self.components
    }

    // ===== Computed Styles =====

    /// Returns the style for primary text.
    pub fn primary_text_style(&self) -> Style {
        Style::default().fg(self.colors.text_primary)
    }

    /// Returns the style for secondary text.
    pub fn secondary_text_style(&self) -> Style {
        Style::default().fg(self.colors.text_secondary)
    }

    /// Returns the style for disabled text.
    pub fn disabled_text_style(&self) -> Style {
        Style::default().fg(self.colors.text_disabled)
    }

    /// Returns the style for error text.
    pub fn error_text_style(&self) -> Style {
        Style::default().fg(self.colors.error)
    }

    /// Returns the style for warning text.
    pub fn warning_text_style(&self) -> Style {
        Style::default().fg(self.colors.warning)
    }

    /// Returns the style for success text.
    pub fn success_text_style(&self) -> Style {
        Style::default().fg(self.colors.success)
    }

    /// Returns the style for info text.
    pub fn info_text_style(&self) -> Style {
        Style::default().fg(self.colors.info)
    }

    /// Returns the style for heading text.
    pub fn heading_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .add_modifier(self.text.heading.add_modifier)
    }

    /// Returns the style for emphasized text.
    pub fn emphasis_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .add_modifier(self.text.emphasis.add_modifier)
    }

    /// Returns the style for muted text.
    pub fn muted_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_secondary)
            .add_modifier(self.text.muted.add_modifier)
    }

    // ===== Border Styles =====

    /// Returns the style for default borders.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.colors.border)
    }

    /// Returns the style for focused borders.
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.colors.border_focused)
    }

    // ===== Button Styles =====

    /// Returns the style for normal (unfocused) buttons.
    pub fn button_normal_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .bg(self.colors.surface)
    }

    /// Returns the style for focused buttons.
    pub fn button_focused_style(&self) -> Style {
        Style::default()
            .fg(self.colors.primary)
            .bg(self.colors.surface)
            .add_modifier(self.components.button.focused_modifier)
    }

    /// Returns the style for pressed buttons.
    pub fn button_pressed_style(&self) -> Style {
        Style::default()
            .fg(self.colors.primary)
            .bg(self.colors.surface)
            .add_modifier(self.components.button.pressed_modifier)
    }

    /// Returns the style for disabled buttons.
    pub fn button_disabled_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_disabled)
            .bg(self.colors.surface)
    }

    // ===== Input Styles =====

    /// Returns the style for normal (unfocused) text inputs.
    pub fn input_normal_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .bg(self.colors.background)
    }

    /// Returns the style for focused text inputs.
    pub fn input_focused_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .bg(self.colors.background)
    }

    /// Returns the style for input placeholders.
    pub fn input_placeholder_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_disabled)
            .add_modifier(self.components.input.placeholder_modifier)
    }

    /// Returns the style for input cursors.
    pub fn input_cursor_style(&self) -> Style {
        Style::default()
            .fg(self.colors.background)
            .bg(self.colors.text_primary)
            .add_modifier(self.components.input.cursor_modifier)
    }

    // ===== Table Styles =====

    /// Returns the style for table headers.
    pub fn table_header_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .add_modifier(self.components.table.header_modifier)
    }

    /// Returns the style for normal table rows.
    pub fn table_row_style(&self) -> Style {
        Style::default().fg(self.colors.text_primary)
    }

    /// Returns the style for selected table rows.
    pub fn table_selected_style(&self) -> Style {
        Style::default()
            .fg(self.colors.primary)
            .add_modifier(self.components.table.selected_modifier)
    }

    // ===== List Styles =====

    /// Returns the style for normal list items.
    pub fn list_item_style(&self) -> Style {
        Style::default().fg(self.colors.text_primary)
    }

    /// Returns the style for selected list items.
    pub fn list_selected_style(&self) -> Style {
        Style::default()
            .fg(self.colors.primary)
            .add_modifier(self.components.list.selected_modifier)
    }

    // ===== Modal Styles =====

    /// Returns the style for modal titles.
    pub fn modal_title_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .add_modifier(self.components.modal.title_modifier)
    }

    /// Returns the style for modal content.
    pub fn modal_content_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_primary)
            .bg(self.colors.surface)
    }

    // ===== Tab Styles =====

    /// Returns the style for active tabs.
    pub fn tab_active_style(&self) -> Style {
        Style::default()
            .fg(self.colors.primary)
            .add_modifier(self.components.tabs.active_modifier)
    }

    /// Returns the style for inactive tabs.
    pub fn tab_inactive_style(&self) -> Style {
        Style::default()
            .fg(self.colors.text_secondary)
            .add_modifier(self.components.tabs.inactive_modifier)
    }

    // ===== Utility Methods =====

    /// Creates a computed style from colors and modifiers.
    pub fn computed_style(
        &self,
        fg: Option<Color>,
        bg: Option<Color>,
        modifiers: Modifier,
    ) -> ComputedStyle {
        ComputedStyle::new(fg, bg, modifiers)
    }

    /// Returns whether this is a dark theme.
    ///
    /// This is a heuristic based on the background color brightness.
    pub fn is_dark(&self) -> bool {
        match self.colors.background {
            Color::Rgb(r, g, b) => {
                // Calculate relative luminance
                let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                luminance < 128.0
            }
            Color::Black | Color::DarkGray => true,
            Color::White
            | Color::Gray
            | Color::LightRed
            | Color::LightGreen
            | Color::LightBlue
            | Color::LightCyan
            | Color::LightMagenta
            | Color::LightYellow => false,
            _ => true, // Default to dark for other colors
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_dark() {
        let theme = Theme::dark();
        assert_eq!(theme.name(), "Dark");
        assert!(theme.is_dark());
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.name(), "Light");
        assert!(!theme.is_dark());
    }

    #[test]
    fn test_theme_default_is_dark() {
        let default = Theme::default();
        let dark = Theme::dark();
        assert_eq!(default.name(), dark.name());
        assert_eq!(default.colors(), dark.colors());
    }

    #[test]
    fn test_theme_builder() {
        let theme = Theme::builder()
            .name("Custom")
            .primary_color(Color::Cyan)
            .build();

        assert_eq!(theme.name(), "Custom");
        assert_eq!(theme.colors().primary, Color::Cyan);
    }

    #[test]
    fn test_primary_text_style() {
        let theme = Theme::dark();
        let style = theme.primary_text_style();
        assert_eq!(style.fg, Some(theme.colors().text_primary));
    }

    #[test]
    fn test_secondary_text_style() {
        let theme = Theme::dark();
        let style = theme.secondary_text_style();
        assert_eq!(style.fg, Some(theme.colors().text_secondary));
    }

    #[test]
    fn test_error_text_style() {
        let theme = Theme::dark();
        let style = theme.error_text_style();
        assert_eq!(style.fg, Some(theme.colors().error));
    }

    #[test]
    fn test_button_styles() {
        let theme = Theme::dark();

        let normal = theme.button_normal_style();
        assert_eq!(normal.fg, Some(theme.colors().text_primary));
        assert_eq!(normal.bg, Some(theme.colors().surface));

        let focused = theme.button_focused_style();
        assert_eq!(focused.fg, Some(theme.colors().primary));
        assert!(focused.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_input_styles() {
        let theme = Theme::dark();

        let normal = theme.input_normal_style();
        assert_eq!(normal.fg, Some(theme.colors().text_primary));

        let placeholder = theme.input_placeholder_style();
        assert_eq!(placeholder.fg, Some(theme.colors().text_disabled));
    }

    #[test]
    fn test_table_styles() {
        let theme = Theme::dark();

        let header = theme.table_header_style();
        assert!(header.add_modifier.contains(Modifier::BOLD));

        let selected = theme.table_selected_style();
        assert_eq!(selected.fg, Some(theme.colors().primary));
    }

    #[test]
    fn test_is_dark_detection() {
        // Dark background
        let dark_theme = Theme::builder()
            .background_color(Color::Rgb(30, 30, 46))
            .build();
        assert!(dark_theme.is_dark());

        // Light background
        let light_theme = Theme::builder()
            .background_color(Color::Rgb(239, 241, 245))
            .build();
        assert!(!light_theme.is_dark());

        // Named dark colors
        let black_theme = Theme::builder().background_color(Color::Black).build();
        assert!(black_theme.is_dark());

        // Named light colors
        let white_theme = Theme::builder().background_color(Color::White).build();
        assert!(!white_theme.is_dark());
    }

    #[test]
    fn test_heading_style() {
        let theme = Theme::dark();
        let style = theme.heading_style();
        assert_eq!(style.fg, Some(theme.colors().text_primary));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_computed_style() {
        let theme = Theme::dark();
        let computed = theme.computed_style(Some(Color::Red), Some(Color::Blue), Modifier::BOLD);

        assert_eq!(computed.fg, Some(Color::Red));
        assert_eq!(computed.bg, Some(Color::Blue));
        assert!(computed.modifiers.contains(Modifier::BOLD));
    }
}
