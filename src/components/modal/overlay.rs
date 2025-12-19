//! Modal overlay component.
//!
//! Provides a semi-transparent overlay behind modal dialogs to
//! visually indicate that the underlying content is not interactive.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Widget};

use crate::theme::Theme;

/// A semi-transparent overlay rendered behind modal dialogs.
///
/// The overlay dims the background content to focus attention on the modal.
/// It also optionally renders a shadow effect for depth perception.
///
/// # Example
///
/// ```rust
/// use tuilib::components::modal::Overlay;
/// use ratatui::prelude::*;
///
/// let overlay = Overlay::new()
///     .with_shadow(true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Overlay {
    /// Whether to show a shadow effect.
    show_shadow: bool,
    /// Optional theme for styling.
    theme: Option<Theme>,
}

impl Overlay {
    /// Creates a new overlay with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to show a shadow effect.
    pub fn with_shadow(mut self, show_shadow: bool) -> Self {
        self.show_shadow = show_shadow;
        self
    }

    /// Sets the theme for styling.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns whether shadow is enabled.
    pub fn has_shadow(&self) -> bool {
        self.show_shadow
    }

    /// Renders the overlay to the given frame covering the full area.
    ///
    /// Call this before rendering the modal content.
    pub fn render(&self, frame: &mut Frame, full_area: Rect) {
        // Clear the area to prepare for overlay
        frame.render_widget(Clear, full_area);

        let theme = self.theme.as_ref().cloned().unwrap_or_default();

        // Render a dimmed background
        // Using a block with a semi-transparent style
        let overlay_style = Style::default().bg(theme.colors().background);

        let block = Block::default().style(overlay_style);
        frame.render_widget(block, full_area);
    }

    /// Renders a shadow effect for the modal.
    ///
    /// Call this after `render` but before rendering the modal content.
    /// The shadow is rendered offset from the modal area.
    pub fn render_shadow(&self, frame: &mut Frame, modal_area: Rect) {
        if !self.show_shadow {
            return;
        }

        let theme = self.theme.as_ref().cloned().unwrap_or_default();

        // Shadow offset (2 right, 1 down)
        let shadow_offset_x = 2u16;
        let shadow_offset_y = 1u16;

        // Calculate shadow area
        let shadow_x = modal_area.x.saturating_add(shadow_offset_x);
        let shadow_y = modal_area.y.saturating_add(shadow_offset_y);

        // Render shadow characters
        // Right edge shadow
        if shadow_x < frame.area().width {
            let right_shadow = Rect::new(
                modal_area.x + modal_area.width,
                modal_area.y + shadow_offset_y,
                shadow_offset_x.min(
                    frame
                        .area()
                        .width
                        .saturating_sub(modal_area.x + modal_area.width),
                ),
                modal_area.height.saturating_sub(shadow_offset_y),
            );

            if right_shadow.width > 0 && right_shadow.height > 0 {
                let shadow_style = Style::default().fg(theme.colors().text_disabled);
                frame.render_widget(ShadowBlock::new(shadow_style), right_shadow);
            }
        }

        // Bottom edge shadow
        if shadow_y < frame.area().height {
            let bottom_shadow = Rect::new(
                modal_area.x + shadow_offset_x,
                modal_area.y + modal_area.height,
                modal_area.width,
                shadow_offset_y.min(
                    frame
                        .area()
                        .height
                        .saturating_sub(modal_area.y + modal_area.height),
                ),
            );

            if bottom_shadow.width > 0 && bottom_shadow.height > 0 {
                let shadow_style = Style::default().fg(theme.colors().text_disabled);
                frame.render_widget(ShadowBlock::new(shadow_style), bottom_shadow);
            }
        }
    }
}

/// A simple widget that renders shadow characters.
struct ShadowBlock {
    style: Style,
}

impl ShadowBlock {
    fn new(style: Style) -> Self {
        Self { style }
    }
}

impl Widget for ShadowBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if x < buf.area.width && y < buf.area.height {
                    let cell = buf.cell_mut((x, y));
                    if let Some(cell) = cell {
                        cell.set_symbol("░");
                        cell.set_style(self.style);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_default() {
        let overlay = Overlay::new();
        assert!(!overlay.has_shadow());
    }

    #[test]
    fn test_overlay_with_shadow() {
        let overlay = Overlay::new().with_shadow(true);
        assert!(overlay.has_shadow());
    }

    #[test]
    fn test_overlay_with_theme() {
        let theme = Theme::dark();
        let overlay = Overlay::new().with_theme(theme);
        assert!(overlay.theme.is_some());
    }
}
