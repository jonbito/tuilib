//! Component-specific style definitions.
//!
//! This module provides style configurations for individual UI components,
//! allowing themes to customize the appearance of specific elements.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// Style configuration for all themed components.
///
/// Each component type has its own style struct that defines
/// how it should appear in different states (normal, focused, disabled, etc.).
///
/// # Example
///
/// ```rust
/// use tuilib::theme::ComponentStyles;
///
/// let styles = ComponentStyles::default();
/// assert!(styles.button.use_border);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComponentStyles {
    /// Style for button components
    pub button: ButtonStyle,
    /// Style for text input components
    pub input: InputStyle,
    /// Style for table components
    pub table: TableStyle,
    /// Style for modal dialog components
    pub modal: ModalStyle,
    /// Style for list components
    pub list: ListStyle,
    /// Style for tab components
    pub tabs: TabsStyle,
}

impl ComponentStyles {
    /// Creates a new component styles configuration.
    pub fn new(
        button: ButtonStyle,
        input: InputStyle,
        table: TableStyle,
        modal: ModalStyle,
        list: ListStyle,
        tabs: TabsStyle,
    ) -> Self {
        Self {
            button,
            input,
            table,
            modal,
            list,
            tabs,
        }
    }
}

/// Style configuration for button components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonStyle {
    /// Whether to show a border around buttons
    pub use_border: bool,
    /// Border type for buttons
    pub border_type: BorderType,
    /// Additional padding around button text
    pub padding: u16,
    /// Text modifier for focused buttons
    pub focused_modifier: Modifier,
    /// Text modifier for pressed buttons
    pub pressed_modifier: Modifier,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            use_border: true,
            border_type: BorderType::Rounded,
            padding: 1,
            focused_modifier: Modifier::BOLD,
            pressed_modifier: Modifier::REVERSED,
        }
    }
}

/// Style configuration for text input components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputStyle {
    /// Whether to show a border around inputs
    pub use_border: bool,
    /// Border type for inputs
    pub border_type: BorderType,
    /// Cursor style modifier
    pub cursor_modifier: Modifier,
    /// Placeholder text modifier
    pub placeholder_modifier: Modifier,
    /// Selection highlight modifier
    pub selection_modifier: Modifier,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            use_border: true,
            border_type: BorderType::Rounded,
            cursor_modifier: Modifier::REVERSED,
            placeholder_modifier: Modifier::DIM,
            selection_modifier: Modifier::REVERSED,
        }
    }
}

/// Style configuration for table components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableStyle {
    /// Whether to show row separators
    pub show_row_separators: bool,
    /// Whether to show column separators
    pub show_column_separators: bool,
    /// Header text modifier
    pub header_modifier: Modifier,
    /// Selected row modifier
    pub selected_modifier: Modifier,
    /// Whether to highlight rows on hover/selection
    pub highlight_rows: bool,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            show_row_separators: false,
            show_column_separators: true,
            header_modifier: Modifier::BOLD,
            selected_modifier: Modifier::REVERSED,
            highlight_rows: true,
        }
    }
}

/// Style configuration for modal dialog components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalStyle {
    /// Border type for modals
    pub border_type: BorderType,
    /// Whether to show a shadow effect
    pub show_shadow: bool,
    /// Title text modifier
    pub title_modifier: Modifier,
    /// Whether to center the modal content
    pub center_content: bool,
}

impl Default for ModalStyle {
    fn default() -> Self {
        Self {
            border_type: BorderType::Double,
            show_shadow: true,
            title_modifier: Modifier::BOLD,
            center_content: true,
        }
    }
}

/// Style configuration for list components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListStyle {
    /// Whether to show bullets/markers
    pub show_markers: bool,
    /// Marker character for unselected items
    pub marker: char,
    /// Marker character for selected items
    pub selected_marker: char,
    /// Selected item modifier
    pub selected_modifier: Modifier,
    /// Whether to highlight the entire row or just the marker
    pub highlight_full_row: bool,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            show_markers: true,
            marker: ' ',
            selected_marker: '>',
            selected_modifier: Modifier::BOLD,
            highlight_full_row: true,
        }
    }
}

/// Style configuration for tab components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsStyle {
    /// Separator between tabs
    pub separator: String,
    /// Active tab modifier
    pub active_modifier: Modifier,
    /// Inactive tab modifier
    pub inactive_modifier: Modifier,
    /// Whether to show a border around the tab bar
    pub use_border: bool,
}

impl Default for TabsStyle {
    fn default() -> Self {
        Self {
            separator: " │ ".to_string(),
            active_modifier: Modifier::BOLD | Modifier::UNDERLINED,
            inactive_modifier: Modifier::empty(),
            use_border: false,
        }
    }
}

/// A computed style that can be applied directly to ratatui widgets.
///
/// This is a convenience struct that combines colors and modifiers
/// into a single applicable style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComputedStyle {
    /// Foreground color
    pub fg: Option<Color>,
    /// Background color
    pub bg: Option<Color>,
    /// Style modifiers
    pub modifiers: Modifier,
}

impl ComputedStyle {
    /// Creates a new computed style.
    pub fn new(fg: Option<Color>, bg: Option<Color>, modifiers: Modifier) -> Self {
        Self { fg, bg, modifiers }
    }

    /// Creates an empty computed style.
    pub fn empty() -> Self {
        Self {
            fg: None,
            bg: None,
            modifiers: Modifier::empty(),
        }
    }

    /// Sets the foreground color.
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Sets the background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Adds modifiers to the style.
    pub fn add_modifiers(mut self, modifiers: Modifier) -> Self {
        self.modifiers |= modifiers;
        self
    }

    /// Converts to a ratatui [`Style`].
    pub fn to_style(self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }
        style.add_modifier(self.modifiers)
    }
}

impl From<ComputedStyle> for Style {
    fn from(computed: ComputedStyle) -> Self {
        computed.to_style()
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_styles_default() {
        let styles = ComponentStyles::default();
        assert!(styles.button.use_border);
        assert!(styles.input.use_border);
        assert!(styles.modal.show_shadow);
    }

    #[test]
    fn test_button_style_default() {
        let style = ButtonStyle::default();
        assert!(style.use_border);
        assert_eq!(style.border_type, BorderType::Rounded);
        assert_eq!(style.padding, 1);
    }

    #[test]
    fn test_input_style_default() {
        let style = InputStyle::default();
        assert!(style.use_border);
        assert!(style.cursor_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_table_style_default() {
        let style = TableStyle::default();
        assert!(!style.show_row_separators);
        assert!(style.show_column_separators);
        assert!(style.highlight_rows);
    }

    #[test]
    fn test_modal_style_default() {
        let style = ModalStyle::default();
        assert_eq!(style.border_type, BorderType::Double);
        assert!(style.show_shadow);
    }

    #[test]
    fn test_list_style_default() {
        let style = ListStyle::default();
        assert!(style.show_markers);
        assert_eq!(style.marker, ' ');
        assert_eq!(style.selected_marker, '>');
    }

    #[test]
    fn test_tabs_style_default() {
        let style = TabsStyle::default();
        assert_eq!(style.separator, " │ ");
        assert!(!style.use_border);
    }

    #[test]
    fn test_computed_style_creation() {
        let style = ComputedStyle::empty()
            .fg(Color::Red)
            .bg(Color::Blue)
            .add_modifiers(Modifier::BOLD);

        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
        assert!(style.modifiers.contains(Modifier::BOLD));
    }

    #[test]
    fn test_computed_style_to_ratatui() {
        let computed = ComputedStyle::new(
            Some(Color::Red),
            Some(Color::Blue),
            Modifier::BOLD | Modifier::ITALIC,
        );
        let style: Style = computed.into();

        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }
}
