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
//! - **Tracing Integration**: Structured logging with lifecycle spans
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
//! ## Tracing
//!
//! The library includes comprehensive tracing integration for debugging TUI applications.
//! Since TUI applications use stdout for rendering, logs are directed to files.
//!
//! To enable tracing setup helpers, add the `tracing-setup` feature:
//!
//! ```toml
//! [dependencies]
//! tuilib = { version = "0.1", features = ["tracing-setup"] }
//! ```
//!
//! Then initialize tracing in your application:
//!
//! ```rust,ignore
//! use tuilib::tracing::{TracingConfig, init_tracing};
//!
//! let config = TracingConfig::new()
//!     .with_log_file("debug.log")
//!     .with_level(tracing::Level::DEBUG);
//!
//! let _guard = init_tracing(config)?;
//! ```
//!
//! ## Modules
//!
//! - [`components`]: UI components (buttons, inputs, etc.)
//! - [`input`]: Input action mapping and keyboard handling
//! - [`focus`]: Focus management and navigation
//! - [`theme`]: Theming and design tokens
//! - [`event`]: Async event loop infrastructure
//! - [`tracing`]: Structured logging and debugging (requires `tracing-setup` feature for setup helpers)

pub mod components;
pub mod event;
pub mod focus;
pub mod input;
pub mod theme;
pub mod tracing;

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
