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

// Module re-exports
pub use crate::components;
pub use crate::event;
pub use crate::focus;
pub use crate::input;
pub use crate::theme;
