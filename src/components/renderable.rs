//! Renderable trait for TUI components.
//!
//! The [`Renderable`] trait defines the core interface for anything that can be
//! rendered to a terminal frame. This is the most fundamental trait in the component
//! system - all visual elements must implement it.
//!
//! # Design Principles
//!
//! - **Simplicity**: Single method interface for rendering
//! - **Composability**: Renderables can contain other renderables
//! - **Integration**: Works directly with ratatui's Frame and Rect types
//!
//! # Examples
//!
//! ```rust
//! use tuilib::components::Renderable;
//! use ratatui::prelude::*;
//!
//! struct Label {
//!     text: String,
//! }
//!
//! impl Renderable for Label {
//!     fn render(&self, frame: &mut Frame, area: Rect) {
//!         let paragraph = ratatui::widgets::Paragraph::new(self.text.as_str());
//!         frame.render_widget(paragraph, area);
//!     }
//! }
//! ```

use ratatui::prelude::*;

/// Base trait for anything that can render to a terminal frame.
///
/// This trait provides the fundamental rendering capability for TUI components.
/// Any type that can be displayed in the terminal should implement this trait.
///
/// # Implementation Notes
///
/// - The `area` parameter defines the rectangular region where the component
///   should render itself. Components should respect these boundaries.
/// - Components are responsible for handling cases where the area is too small.
/// - The frame provides access to the underlying buffer for rendering.
///
/// # Examples
///
/// ## Basic Implementation
///
/// ```rust
/// use tuilib::components::Renderable;
/// use ratatui::prelude::*;
///
/// struct StatusBar {
///     message: String,
/// }
///
/// impl Renderable for StatusBar {
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         let text = ratatui::widgets::Paragraph::new(self.message.as_str())
///             .style(Style::default().fg(Color::White).bg(Color::Blue));
///         frame.render_widget(text, area);
///     }
/// }
/// ```
///
/// ## Composing Renderables
///
/// ```rust
/// use tuilib::components::Renderable;
/// use ratatui::prelude::*;
///
/// struct Container<R: Renderable> {
///     child: R,
/// }
///
/// impl<R: Renderable> Renderable for Container<R> {
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         // Add a border, then render child in the inner area
///         let block = ratatui::widgets::Block::bordered();
///         let inner = block.inner(area);
///         frame.render_widget(block, area);
///         self.child.render(frame, inner);
///     }
/// }
/// ```
pub trait Renderable {
    /// Renders this component to the given frame within the specified area.
    ///
    /// # Arguments
    ///
    /// * `frame` - The terminal frame to render to
    /// * `area` - The rectangular area within which to render
    fn render(&self, frame: &mut Frame, area: Rect);
}

/// Blanket implementation for boxed renderables.
///
/// This allows storing heterogeneous renderables in collections.
impl<R: Renderable + ?Sized> Renderable for Box<R> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        (**self).render(frame, area);
    }
}

/// Blanket implementation for referenced renderables.
impl<R: Renderable + ?Sized> Renderable for &R {
    fn render(&self, frame: &mut Frame, area: Rect) {
        (**self).render(frame, area);
    }
}

/// Blanket implementation for mutably referenced renderables.
impl<R: Renderable + ?Sized> Renderable for &mut R {
    fn render(&self, frame: &mut Frame, area: Rect) {
        (**self).render(frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        rendered: std::cell::Cell<bool>,
    }

    impl TestWidget {
        fn new() -> Self {
            Self {
                rendered: std::cell::Cell::new(false),
            }
        }

        #[allow(dead_code)]
        fn was_rendered(&self) -> bool {
            self.rendered.get()
        }
    }

    impl Renderable for TestWidget {
        fn render(&self, _frame: &mut Frame, _area: Rect) {
            self.rendered.set(true);
        }
    }

    #[test]
    fn test_box_renderable() {
        // Test that Box<dyn Renderable> works
        let widget = TestWidget::new();
        let _boxed: Box<dyn Renderable> = Box::new(widget);
        // Compilation success is the test
    }

    #[test]
    fn test_ref_renderable() {
        // Test that &impl Renderable works
        let widget = TestWidget::new();
        let _ref_widget: &dyn Renderable = &widget;
        // Compilation success is the test
    }
}
