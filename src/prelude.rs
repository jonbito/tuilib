//! Convenience re-exports for common types and traits.
//!
//! This prelude module provides a single import to access the most commonly
//! used types and traits from tuilib:
//!
//! ```rust
//! use tuilib::prelude::*;
//! ```

// Re-export ratatui prelude for convenience
pub use ratatui::prelude::*;

// Component exports will be added as components are implemented
pub use crate::components;
pub use crate::event;
pub use crate::focus;
pub use crate::input;
pub use crate::theme;
