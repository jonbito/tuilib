//! Convenience re-exports for common types and traits.
//!
//! This prelude module provides a single import to access the most commonly
//! used types and traits from tuilib:
//!
//! ```rust
//! use tuilib::prelude::*;
//! ```
//!
//! # Included Types and Traits
//!
//! This prelude includes:
//!
//! - All types from `ratatui::prelude::*`
//! - Core component traits: [`Component`], [`Focusable`], [`Renderable`]
//! - Convenience types: [`FocusWrapper`], [`FocusableComponent`]
//! - Input types: [`Action`], [`KeyBinding`], [`KeyBindings`], [`KeySequence`], [`InputMatcher`]
//! - Focus types: [`FocusId`], [`FocusManager`], [`FocusRing`], [`FocusTrap`]
//! - Theme types: [`Theme`], [`ThemeBuilder`], [`ColorPalette`]
//! - Event loop types: [`EventLoop`], [`EventLoopConfig`], [`AppEvent`], [`ControlFlow`]
//! - Timing utilities: [`Debouncer`], [`Throttle`]
//! - Tracing types: [`TracingConfig`], and with `tracing-setup` feature: [`init_tracing`], [`TracingGuard`]
//! - Tracing macros: [`component_update_span!`], [`component_render_span!`], [`focus_span!`]

// Re-export ratatui prelude for convenience
pub use ratatui::prelude::*;

// Core component traits
pub use crate::components::{
    Component, FocusWrapper, Focusable, FocusableComponent, Renderable, StatelessComponent,
};

// Input types
pub use crate::input::{
    Action, InputMatcher, KeyBinding, KeyBindings, KeyBindingsBuilder, KeySequence, MatchResult,
};

// Focus types
pub use crate::focus::{FocusDirection, FocusId, FocusManager, FocusResult, FocusRing, FocusTrap};

// Theme types
pub use crate::theme::{ColorPalette, Theme, ThemeBuilder};

// Event loop types
pub use crate::event::{AppEvent, ControlFlow, Debouncer, EventLoop, EventLoopConfig, Throttle};

// Tracing types
pub use crate::tracing::TracingConfig;
#[cfg(feature = "tracing-setup")]
pub use crate::tracing::{init_tracing, TracingError, TracingGuard};

// Re-export tracing macros for convenience
pub use crate::{component_render_span, component_update_span, focus_span};

// Module re-exports
pub use crate::components;
pub use crate::event;
pub use crate::focus;
pub use crate::input;
pub use crate::theme;
pub use crate::tracing;
