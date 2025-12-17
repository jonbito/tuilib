//! Input handling module with action mapping.
//!
//! This module provides input action mapping functionality using terminput.
//! It allows mapping keyboard and mouse events to semantic actions,
//! enabling consistent input handling across components.
//!
//! # Overview
//!
//! The input system consists of several key components:
//!
//! - [`Action`]: Named semantic actions like "quit", "save", "navigate_up"
//! - [`KeyBinding`]: A single key with optional modifiers (e.g., "Ctrl+S")
//! - [`KeySequence`]: One or more keys in sequence (e.g., "Ctrl+X Ctrl+S")
//! - [`KeyBindings`]: Container for keybindings with context support
//! - [`KeyBindingsBuilder`]: Fluent API for declarative keybinding configuration
//! - [`InputMatcher`]: Matches input events against registered bindings
//!
//! # Quick Start
//!
//! ## Using KeyBindings Builder (Recommended)
//!
//! ```rust
//! use tuilib::input::{KeyBindings, Action};
//!
//! // Create bindings with the builder API
//! let bindings = KeyBindings::builder()
//!     .bind("quit", "Ctrl+q")
//!     .bind("save", "Ctrl+s")
//!     .bind_multi("navigate_up", &["k", "Up"])
//!     .context("modal", |ctx| {
//!         ctx.bind("close", "Escape")
//!            .bind("confirm", "Enter")
//!     })
//!     .build();
//!
//! // Look up actions
//! use tuilib::input::parser::parse_key_binding;
//! let ctrl_q = parse_key_binding("Ctrl+q").unwrap();
//! if let Some(action) = bindings.lookup(None, &ctrl_q.into()) {
//!     println!("Action: {}", action.name());
//! }
//! ```
//!
//! ## Using InputMatcher (Low-level)
//!
//! ```rust
//! use tuilib::input::{Action, InputMatcher, KeyBinding, KeySequence, ctrl, char_key};
//! use terminput::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind, KeyEventState};
//! use std::time::Duration;
//!
//! // Create a matcher with 1 second sequence timeout
//! let mut matcher = InputMatcher::new(Duration::from_secs(1));
//!
//! // Register simple key binding
//! matcher.register_key(char_key('q'), Action::new("quit"));
//!
//! // Register binding with modifier
//! matcher.register_key(ctrl('s'), Action::new("save"));
//!
//! // Register multi-key sequence (Emacs-style)
//! matcher.register(
//!     KeySequence::new(vec![ctrl('x'), ctrl('c')]),
//!     Action::new("exit")
//! );
//!
//! // Register multiple keys for same action (vim-style)
//! matcher.register_multiple(
//!     vec![
//!         KeySequence::single(char_key('k')),
//!         KeySequence::single(KeyBinding::new(KeyCode::Up)),
//!     ],
//!     Action::new("move_up")
//! );
//!
//! // Process input events
//! let event = KeyEvent {
//!     code: KeyCode::Char('q'),
//!     modifiers: KeyModifiers::NONE,
//!     kind: KeyEventKind::Press,
//!     state: KeyEventState::NONE,
//! };
//!
//! match matcher.process(&event) {
//!     tuilib::input::MatchResult::Matched(action) => {
//!         println!("Action: {}", action.name());
//!     }
//!     tuilib::input::MatchResult::Pending => {
//!         println!("Waiting for more keys...");
//!     }
//!     tuilib::input::MatchResult::NoMatch => {
//!         println!("No binding matched");
//!     }
//! }
//! ```
//!
//! # Key Sequences
//!
//! Key sequences allow Emacs-style multi-key bindings:
//!
//! ```rust
//! use tuilib::input::{KeySequenceBuilder, KeySequenceBuilderExt};
//!
//! // Build a sequence fluently
//! let save_seq = KeySequenceBuilder::new()
//!     .ctrl('x')
//!     .ctrl('s')
//!     .build();
//!
//! // Or use helper functions
//! use tuilib::input::{seq2, ctrl};
//! let exit_seq = seq2(ctrl('x'), ctrl('c'));
//! ```
//!
//! # String Parsing
//!
//! The [`parser`] module provides string parsing for key combinations:
//!
//! ```rust
//! use tuilib::input::parser::{parse_key_binding, parse_key_sequence};
//!
//! // Parse single binding
//! let binding = parse_key_binding("Ctrl+Shift+s").unwrap();
//!
//! // Parse key sequence
//! let sequence = parse_key_sequence("Ctrl+x Ctrl+s").unwrap();
//! ```
//!
//! # Common Bindings
//!
//! The [`common`] module provides pre-defined bindings for typical keys:
//!
//! ```rust
//! use tuilib::input::common;
//!
//! let esc = common::escape();
//! let enter = common::enter();
//! let ctrl_c = common::ctrl_c();
//! let up = common::up();
//! ```
//!
//! # Configuration Support
//!
//! Keybindings can be loaded from configuration files using serde:
//!
//! ```rust
//! use tuilib::input::KeyBindingsConfig;
//!
//! // Parse from TOML, JSON, or YAML
//! let config: KeyBindingsConfig = toml::from_str(r#"
//! [global]
//! quit = "Ctrl+q"
//! save = "Ctrl+s"
//! navigate_up = ["k", "Up"]
//!
//! [contexts.modal]
//! close = "Escape"
//! confirm = "Enter"
//! "#).unwrap();
//!
//! let bindings = config.into_key_bindings().unwrap();
//! ```

mod action;
mod binding;
pub mod bindings;
mod matcher;
pub mod parser;
mod sequence;
mod terminput_ext;

// Core types
pub use action::Action;
pub use binding::KeyBinding;
pub use bindings::{ContextBuilder, KeyBindings, KeyBindingsBuilder, KeyBindingsConfig, KeyOrKeys};
pub use matcher::{InputMatcher, MatchResult};
pub use sequence::{KeySequence, KeySequenceBuilder};

// Terminput helpers
pub use terminput_ext::{
    alt, alt_key, char_key, common, ctrl, ctrl_key, extract_key_press, f_key, is_key_press,
    is_key_release, key, seq, seq2, shift, shift_key, test_key_event, KeySequenceBuilderExt,
};

// Re-export terminput types that users will need
pub use terminput::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
