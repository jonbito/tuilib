//! Theme builder for creating custom themes.
//!
//! The [`ThemeBuilder`] provides a fluent API for creating themes with
//! custom colors, border styles, text styles, and component configurations.

use ratatui::style::Color;
use ratatui::widgets::BorderType;

use super::colors::ColorPalette;
use super::component::{
    ButtonStyle, ComponentStyles, InputStyle, ListStyle, ModalStyle, TableStyle, TabsStyle,
};
use super::core::Theme;
use super::styles::{BorderStyles, TextStyles};

/// Builder for creating custom themes.
///
/// Provides a fluent API for constructing themes with custom configurations.
/// Start with a base theme (light or dark) and customize as needed.
///
/// # Example
///
/// ```rust
/// use tuilib::theme::ThemeBuilder;
/// use ratatui::style::Color;
///
/// let theme = ThemeBuilder::new()
///     .dark_base()
///     .primary_color(Color::Cyan)
///     .secondary_color(Color::Magenta)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ThemeBuilder {
    name: String,
    colors: ColorPalette,
    borders: BorderStyles,
    text: TextStyles,
    components: ComponentStyles,
}

impl ThemeBuilder {
    /// Creates a new theme builder with dark defaults.
    pub fn new() -> Self {
        Self {
            name: "Custom".to_string(),
            colors: ColorPalette::dark(),
            borders: BorderStyles::default(),
            text: TextStyles::default(),
            components: ComponentStyles::default(),
        }
    }

    /// Sets the theme name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Uses the dark color palette as the base.
    pub fn dark_base(mut self) -> Self {
        self.colors = ColorPalette::dark();
        self
    }

    /// Uses the light color palette as the base.
    pub fn light_base(mut self) -> Self {
        self.colors = ColorPalette::light();
        self
    }

    /// Sets the entire color palette.
    pub fn colors(mut self, palette: ColorPalette) -> Self {
        self.colors = palette;
        self
    }

    /// Sets the primary accent color.
    pub fn primary_color(mut self, color: Color) -> Self {
        self.colors.primary = color;
        self
    }

    /// Sets the secondary accent color.
    pub fn secondary_color(mut self, color: Color) -> Self {
        self.colors.secondary = color;
        self
    }

    /// Sets the background color.
    pub fn background_color(mut self, color: Color) -> Self {
        self.colors.background = color;
        self
    }

    /// Sets the surface color.
    pub fn surface_color(mut self, color: Color) -> Self {
        self.colors.surface = color;
        self
    }

    /// Sets the error color.
    pub fn error_color(mut self, color: Color) -> Self {
        self.colors.error = color;
        self
    }

    /// Sets the warning color.
    pub fn warning_color(mut self, color: Color) -> Self {
        self.colors.warning = color;
        self
    }

    /// Sets the success color.
    pub fn success_color(mut self, color: Color) -> Self {
        self.colors.success = color;
        self
    }

    /// Sets the info color.
    pub fn info_color(mut self, color: Color) -> Self {
        self.colors.info = color;
        self
    }

    /// Sets the primary text color.
    pub fn text_primary_color(mut self, color: Color) -> Self {
        self.colors.text_primary = color;
        self
    }

    /// Sets the secondary text color.
    pub fn text_secondary_color(mut self, color: Color) -> Self {
        self.colors.text_secondary = color;
        self
    }

    /// Sets the disabled text color.
    pub fn text_disabled_color(mut self, color: Color) -> Self {
        self.colors.text_disabled = color;
        self
    }

    /// Sets the default border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.colors.border = color;
        self
    }

    /// Sets the focused border color.
    pub fn border_focused_color(mut self, color: Color) -> Self {
        self.colors.border_focused = color;
        self
    }

    /// Sets the entire border styles configuration.
    pub fn borders(mut self, borders: BorderStyles) -> Self {
        self.borders = borders;
        self
    }

    /// Sets the default border type.
    pub fn default_border_type(mut self, border_type: BorderType) -> Self {
        self.borders.default = border_type;
        self
    }

    /// Sets the focused border type.
    pub fn focused_border_type(mut self, border_type: BorderType) -> Self {
        self.borders.focused = border_type;
        self
    }

    /// Sets the modal border type.
    pub fn modal_border_type(mut self, border_type: BorderType) -> Self {
        self.borders.modal = border_type;
        self
    }

    /// Uses modern border styles (rounded corners).
    pub fn modern_borders(mut self) -> Self {
        self.borders = BorderStyles::modern();
        self
    }

    /// Uses classic border styles (plain corners).
    pub fn classic_borders(mut self) -> Self {
        self.borders = BorderStyles::classic();
        self
    }

    /// Uses minimal border styles.
    pub fn minimal_borders(mut self) -> Self {
        self.borders = BorderStyles::minimal();
        self
    }

    /// Sets the entire text styles configuration.
    pub fn text_styles(mut self, text: TextStyles) -> Self {
        self.text = text;
        self
    }

    /// Sets the entire component styles configuration.
    pub fn component_styles(mut self, components: ComponentStyles) -> Self {
        self.components = components;
        self
    }

    /// Sets the button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.components.button = style;
        self
    }

    /// Sets the input style.
    pub fn input_style(mut self, style: InputStyle) -> Self {
        self.components.input = style;
        self
    }

    /// Sets the table style.
    pub fn table_style(mut self, style: TableStyle) -> Self {
        self.components.table = style;
        self
    }

    /// Sets the modal style.
    pub fn modal_style(mut self, style: ModalStyle) -> Self {
        self.components.modal = style;
        self
    }

    /// Sets the list style.
    pub fn list_style(mut self, style: ListStyle) -> Self {
        self.components.list = style;
        self
    }

    /// Sets the tabs style.
    pub fn tabs_style(mut self, style: TabsStyle) -> Self {
        self.components.tabs = style;
        self
    }

    /// Builds the theme.
    pub fn build(self) -> Theme {
        Theme::new(
            self.name,
            self.colors,
            self.borders,
            self.text,
            self.components,
        )
    }
}

impl Default for ThemeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = ThemeBuilder::new();
        let theme = builder.build();
        assert_eq!(theme.name(), "Custom");
    }

    #[test]
    fn test_builder_name() {
        let theme = ThemeBuilder::new().name("My Theme").build();
        assert_eq!(theme.name(), "My Theme");
    }

    #[test]
    fn test_builder_dark_base() {
        let theme = ThemeBuilder::new().dark_base().build();
        assert_eq!(theme.colors(), &ColorPalette::dark());
    }

    #[test]
    fn test_builder_light_base() {
        let theme = ThemeBuilder::new().light_base().build();
        assert_eq!(theme.colors(), &ColorPalette::light());
    }

    #[test]
    fn test_builder_primary_color() {
        let theme = ThemeBuilder::new().primary_color(Color::Red).build();
        assert_eq!(theme.colors().primary, Color::Red);
    }

    #[test]
    fn test_builder_secondary_color() {
        let theme = ThemeBuilder::new().secondary_color(Color::Blue).build();
        assert_eq!(theme.colors().secondary, Color::Blue);
    }

    #[test]
    fn test_builder_background_color() {
        let theme = ThemeBuilder::new().background_color(Color::Black).build();
        assert_eq!(theme.colors().background, Color::Black);
    }

    #[test]
    fn test_builder_modern_borders() {
        let theme = ThemeBuilder::new().modern_borders().build();
        assert_eq!(theme.borders().default, BorderType::Rounded);
    }

    #[test]
    fn test_builder_classic_borders() {
        let theme = ThemeBuilder::new().classic_borders().build();
        assert_eq!(theme.borders().default, BorderType::Plain);
    }

    #[test]
    fn test_builder_chaining() {
        let theme = ThemeBuilder::new()
            .name("Chained Theme")
            .light_base()
            .primary_color(Color::Cyan)
            .secondary_color(Color::Magenta)
            .modern_borders()
            .build();

        assert_eq!(theme.name(), "Chained Theme");
        assert_eq!(theme.colors().primary, Color::Cyan);
        assert_eq!(theme.colors().secondary, Color::Magenta);
        assert_eq!(theme.borders().default, BorderType::Rounded);
    }

    #[test]
    fn test_builder_custom_colors() {
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

        let theme = ThemeBuilder::new().colors(palette.clone()).build();
        assert_eq!(theme.colors(), &palette);
    }

    #[test]
    fn test_builder_component_style() {
        let button_style = ButtonStyle {
            padding: 5,
            ..Default::default()
        };

        let theme = ThemeBuilder::new().button_style(button_style).build();
        assert_eq!(theme.components().button.padding, 5);
    }
}
