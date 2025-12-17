//! # tuilib
//!
//! An opinionated TUI component library with input action mapping built on ratatui.
//!
//! ## Features
//!
//! - **Component System**: Reusable, composable UI components
//! - **Input Action Mapping**: Semantic input handling using terminput
//! - **Focus Management**: Built-in focus navigation between components
//! - **Theming**: Consistent styling with design tokens
//! - **Async Event Loop**: Tokio-powered event handling
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use tuilib::prelude::*;
//!
//! // Components and application code will be documented
//! // as they are implemented in subsequent tasks.
//! ```
//!
//! ## Modules
//!
//! - [`components`]: UI components (buttons, inputs, etc.)
//! - [`input`]: Input action mapping and keyboard handling
//! - [`focus`]: Focus management and navigation
//! - [`theme`]: Theming and design tokens
//! - [`event`]: Async event loop infrastructure

pub mod components;
pub mod event;
pub mod focus;
pub mod input;
pub mod theme;

pub mod prelude;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_exports_modules() {
        // Verify that prelude re-exports the main modules
        let _ = std::any::TypeId::of::<prelude::Rect>();
    }
}
