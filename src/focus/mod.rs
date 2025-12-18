//! Focus management module.
//!
//! This module handles focus navigation between components.
//! It provides a focus system that enables keyboard navigation
//! and tracks which component currently has input focus.
//!
//! # Overview
//!
//! The focus system consists of several key types:
//!
//! - [`FocusId`]: Unique identifier for focusable components
//! - [`FocusRing`]: Ordered collection of focusable components with navigation
//! - [`FocusManager`]: Main interface for focus management
//! - [`FocusTrap`]: Focus restriction for modal dialogs
//!
//! # Basic Usage
//!
//! ```rust
//! use tuilib::focus::{FocusId, FocusManager, FocusDirection};
//!
//! // Create a focus manager
//! let mut manager = FocusManager::new();
//!
//! // Register focusable components
//! manager.register(FocusId::new("username-input"), 0);
//! manager.register(FocusId::new("password-input"), 0);
//! manager.register(FocusId::new("submit-button"), 0);
//!
//! // Navigate forward (Tab)
//! manager.focus_next();
//! assert_eq!(manager.current(), Some(&FocusId::new("username-input")));
//!
//! // Navigate forward again
//! manager.focus_next();
//! assert_eq!(manager.current(), Some(&FocusId::new("password-input")));
//!
//! // Navigate backward (Shift+Tab)
//! manager.focus_prev();
//! assert_eq!(manager.current(), Some(&FocusId::new("username-input")));
//!
//! // Direct focus by ID
//! manager.focus(&FocusId::new("submit-button"));
//! assert_eq!(manager.current(), Some(&FocusId::new("submit-button")));
//! ```
//!
//! # Focus Order
//!
//! Components are navigated based on their focus order. Lower order values
//! are focused first. Components with equal order values maintain their
//! registration order.
//!
//! ```rust
//! use tuilib::focus::{FocusId, FocusManager};
//!
//! let mut manager = FocusManager::new();
//!
//! // Register with custom order
//! manager.register(FocusId::new("normal"), 0);
//! manager.register(FocusId::new("first"), -100);  // First in tab order
//! manager.register(FocusId::new("last"), 100);    // Last in tab order
//!
//! // Navigation follows order: first -> normal -> last
//! manager.focus_next();
//! assert_eq!(manager.current(), Some(&FocusId::new("first")));
//! ```
//!
//! # Focus Traps (Modal Dialogs)
//!
//! Focus traps restrict keyboard navigation to a specific set of components,
//! preventing users from tabbing out of modal dialogs.
//!
//! ```rust
//! use tuilib::focus::{FocusId, FocusManager, FocusTrap};
//!
//! let mut manager = FocusManager::new();
//!
//! // Main page components
//! manager.register(FocusId::new("main-content"), 0);
//! manager.focus_next();
//!
//! // Open modal with focus trap
//! let mut trap = FocusTrap::new();
//! trap.register(FocusId::new("modal-input"), 0);
//! trap.register(FocusId::new("modal-ok"), 0);
//! trap.register(FocusId::new("modal-cancel"), 0);
//! manager.push_trap(trap);
//!
//! // Navigation is now restricted to modal components
//! manager.focus_next();
//! manager.focus_next();
//! manager.focus_next();
//! // Still within modal (wraps around)
//! assert!(manager.current().map(|id| id.as_str().starts_with("modal")).unwrap_or(false));
//!
//! // Close modal and restore previous focus
//! manager.pop_trap();
//! assert_eq!(manager.current(), Some(&FocusId::new("main-content")));
//! ```
//!
//! # Integration with Focusable Trait
//!
//! The focus system works with the [`Focusable`](crate::components::Focusable) trait
//! from the components module. Components implementing `Focusable` can be registered
//! with the focus manager.
//!
//! ```rust
//! use tuilib::components::Focusable;
//! use tuilib::focus::{FocusId, FocusManager};
//!
//! struct MyButton {
//!     id: FocusId,
//!     focused: bool,
//! }
//!
//! impl Focusable for MyButton {
//!     fn is_focused(&self) -> bool { self.focused }
//!     fn set_focused(&mut self, focused: bool) { self.focused = focused; }
//! }
//!
//! // Register the button with the focus manager
//! let mut manager = FocusManager::new();
//! let button = MyButton {
//!     id: FocusId::new("my-button"),
//!     focused: false,
//! };
//! manager.register(button.id.clone(), 0);
//! ```

mod id;
mod manager;
mod ring;
mod trap;

pub use id::FocusId;
pub use manager::{FocusDirection, FocusManager, FocusResult};
pub use ring::FocusRing;
pub use trap::FocusTrap;
