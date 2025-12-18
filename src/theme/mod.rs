//! Theme and styling module.
//!
//! This module provides theming capabilities including color palettes,
//! border styles, text styles, and component-specific styling for
//! consistent visual appearance across all components.
//!
//! # Overview
//!
//! The theming system consists of:
//!
//! - [`Theme`]: The main theme struct combining all styling elements
//! - [`ThemeBuilder`]: A fluent builder API for creating custom themes
//! - [`ColorPalette`]: Semantic color definitions (primary, secondary, etc.)
//! - [`BorderStyles`]: Border type configurations for different states
//! - [`TextStyles`] and [`TextStyle`]: Text modifier configurations
//! - [`ComponentStyles`]: Component-specific style configurations
//!
//! # Quick Start
//!
//! Use the built-in themes:
//!
//! ```rust
//! use tuilib::theme::Theme;
//!
//! // Use the dark theme (default)
//! let theme = Theme::dark();
//!
//! // Or the light theme
//! let theme = Theme::light();
//!
//! // Access computed styles
//! let text_style = theme.primary_text_style();
//! let button_style = theme.button_focused_style();
//! ```
//!
//! # Custom Themes
//!
//! Create custom themes using the builder:
//!
//! ```rust
//! use tuilib::theme::Theme;
//! use ratatui::style::Color;
//!
//! let theme = Theme::builder()
//!     .name("My Custom Theme")
//!     .dark_base()
//!     .primary_color(Color::Cyan)
//!     .secondary_color(Color::Magenta)
//!     .modern_borders()
//!     .build();
//! ```
//!
//! # Color Palette
//!
//! The [`ColorPalette`] provides semantic color roles:
//!
//! ```rust
//! use tuilib::theme::ColorPalette;
//!
//! let palette = ColorPalette::dark();
//! let primary = palette.primary;       // Main accent color
//! let background = palette.background; // Background color
//! let error = palette.error;           // Error state color
//! ```
//!
//! # Component Styles
//!
//! Each component type has dedicated style configuration:
//!
//! ```rust
//! use tuilib::theme::{Theme, ButtonStyle, InputStyle};
//!
//! let theme = Theme::dark();
//!
//! // Access component-specific configurations
//! let button_uses_border = theme.components().button.use_border;
//! let input_uses_border = theme.components().input.use_border;
//! ```
//!
//! # Example: Themed Button
//!
//! ```rust
//! use tuilib::theme::Theme;
//! use ratatui::prelude::*;
//! use ratatui::widgets::{Block, Borders, Paragraph};
//!
//! fn render_button(theme: &Theme, label: &str, focused: bool, frame: &mut Frame, area: Rect) {
//!     let style = if focused {
//!         theme.button_focused_style()
//!     } else {
//!         theme.button_normal_style()
//!     };
//!
//!     let border_style = if focused {
//!         theme.border_focused_style()
//!     } else {
//!         theme.border_style()
//!     };
//!
//!     let block = Block::default()
//!         .borders(Borders::ALL)
//!         .border_type(theme.borders().default)
//!         .border_style(border_style);
//!
//!     let paragraph = Paragraph::new(label)
//!         .style(style)
//!         .block(block);
//!
//!     frame.render_widget(paragraph, area);
//! }
//! ```

mod builder;
mod colors;
mod component;
mod core;
mod styles;

// Main types
pub use builder::ThemeBuilder;
pub use colors::ColorPalette;
pub use component::{
    ButtonStyle, ComponentStyles, ComputedStyle, InputStyle, ListStyle, ModalStyle, TableStyle,
    TabsStyle,
};
pub use core::Theme;
pub use styles::{BorderStyles, TextStyle, TextStyles};
