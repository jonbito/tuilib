//! UI Components module
//!
//! This module contains all reusable TUI components built on top of ratatui.
//! Components are designed to be composable and follow consistent patterns
//! for rendering, input handling, and focus management.
//!
//! # Core Traits
//!
//! The component system is built on three core traits:
//!
//! - [`Renderable`]: Base trait for anything that can render to a terminal frame
//! - [`Focusable`]: Trait for components that can receive and manage focus
//! - [`Component`]: Main trait following the Elm architecture pattern
//!
//! # Architecture
//!
//! Components in tuilib follow the Elm architecture:
//!
//! 1. **State**: Components maintain their own internal state
//! 2. **Messages**: Events that describe what happened (user input, etc.)
//! 3. **Update**: Process messages and optionally emit actions
//! 4. **Render**: Display the component based on current state
//!
//! # Example
//!
//! ```rust
//! use tuilib::components::{Component, Focusable, Renderable};
//! use ratatui::prelude::*;
//!
//! struct Button {
//!     label: String,
//!     focused: bool,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum ButtonMsg {
//!     Press,
//! }
//!
//! #[derive(Debug)]
//! enum ButtonAction {
//!     Pressed,
//! }
//!
//! impl Component for Button {
//!     type Message = ButtonMsg;
//!     type Action = ButtonAction;
//!
//!     fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
//!         match msg {
//!             ButtonMsg::Press => Some(ButtonAction::Pressed),
//!         }
//!     }
//! }
//!
//! impl Focusable for Button {
//!     fn is_focused(&self) -> bool { self.focused }
//!     fn set_focused(&mut self, focused: bool) { self.focused = focused; }
//! }
//!
//! impl Renderable for Button {
//!     fn render(&self, frame: &mut Frame, area: Rect) {
//!         let style = if self.focused {
//!             Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
//!         } else {
//!             Style::default()
//!         };
//!         let paragraph = ratatui::widgets::Paragraph::new(self.label.as_str())
//!             .style(style);
//!         frame.render_widget(paragraph, area);
//!     }
//! }
//! ```

mod component;
mod focusable;
mod renderable;
mod text_input;

pub use component::{Component, FocusableComponent, StatelessComponent};
pub use focusable::{FocusWrapper, Focusable};
pub use renderable::Renderable;
pub use text_input::{TextInput, TextInputAction, TextInputMsg, ValidationResult};
