//! Action handler trait for components to handle routed actions.
//!
//! This module provides the [`ActionHandler`] trait which defines how components
//! handle actions during propagation. It supports both capture and bubble phases,
//! similar to DOM event propagation.
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::{Action, ActionHandler, Phase, HandleResult};
//!
//! struct Button {
//!     id: String,
//!     focused: bool,
//! }
//!
//! impl ActionHandler for Button {
//!     fn handle(&mut self, action: &Action, phase: Phase) -> HandleResult {
//!         // Only handle actions during bubble phase when focused
//!         if phase == Phase::Bubble && self.focused {
//!             if action.name() == "activate" {
//!                 println!("Button {} activated!", self.id);
//!                 return HandleResult::Handled;
//!             }
//!         }
//!         HandleResult::Continue
//!     }
//!
//!     fn id(&self) -> &str {
//!         &self.id
//!     }
//! }
//! ```

use std::fmt;

use super::Action;

/// The propagation phase for action handling.
///
/// Similar to DOM events, actions can be handled in two phases:
///
/// - **Capture**: Actions propagate from the root toward the target.
///   Handlers can intercept actions before they reach the target.
/// - **Bubble**: Actions propagate from the target back up to the root.
///   This is where most handling typically occurs.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::Phase;
///
/// let phase = Phase::Capture;
/// assert!(phase.is_capture());
///
/// let phase = Phase::Bubble;
/// assert!(phase.is_bubble());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// The capture phase - action propagates from root toward target.
    Capture,
    /// The bubble phase - action propagates from target back to root.
    Bubble,
}

impl Phase {
    /// Returns true if this is the capture phase.
    pub fn is_capture(&self) -> bool {
        matches!(self, Phase::Capture)
    }

    /// Returns true if this is the bubble phase.
    pub fn is_bubble(&self) -> bool {
        matches!(self, Phase::Bubble)
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Capture => write!(f, "Capture"),
            Phase::Bubble => write!(f, "Bubble"),
        }
    }
}

/// The result of handling an action.
///
/// Handlers return this enum to indicate what should happen next
/// in the propagation chain.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::HandleResult;
///
/// let result = HandleResult::Handled;
/// assert!(result.should_stop());
///
/// let result = HandleResult::Continue;
/// assert!(!result.should_stop());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HandleResult {
    /// Continue propagation to the next handler.
    /// The action was not handled by this handler.
    Continue,
    /// The action was handled. Stop propagation.
    Handled,
    /// This handler doesn't care about this action.
    /// Continue propagation to the next handler.
    #[default]
    Ignored,
}

impl HandleResult {
    /// Returns true if propagation should stop after this result.
    pub fn should_stop(&self) -> bool {
        matches!(self, HandleResult::Handled)
    }

    /// Returns true if this result indicates the action was handled.
    pub fn is_handled(&self) -> bool {
        matches!(self, HandleResult::Handled)
    }

    /// Returns true if this result indicates the handler ignored the action.
    pub fn is_ignored(&self) -> bool {
        matches!(self, HandleResult::Ignored)
    }

    /// Returns true if propagation should continue.
    pub fn should_continue(&self) -> bool {
        !self.should_stop()
    }
}

impl fmt::Display for HandleResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandleResult::Continue => write!(f, "Continue"),
            HandleResult::Handled => write!(f, "Handled"),
            HandleResult::Ignored => write!(f, "Ignored"),
        }
    }
}

/// Trait for components that can handle routed actions.
///
/// Components implementing this trait can participate in action propagation.
/// Actions flow through the component hierarchy in two phases:
///
/// 1. **Capture phase**: From root to target (parent → child)
/// 2. **Bubble phase**: From target back to root (child → parent)
///
/// # Implementation Notes
///
/// - Return `HandleResult::Handled` to stop propagation
/// - Return `HandleResult::Continue` to pass to the next handler
/// - Return `HandleResult::Ignored` if the handler doesn't care about the action
/// - Check the phase to handle actions only during capture or bubble
///
/// # Examples
///
/// ```rust
/// use tuilib::input::{Action, ActionHandler, Phase, HandleResult};
///
/// struct Container {
///     children: Vec<Box<dyn ActionHandler>>,
/// }
///
/// impl ActionHandler for Container {
///     fn handle(&mut self, action: &Action, phase: Phase) -> HandleResult {
///         if phase == Phase::Capture && action.name() == "close_all" {
///             // Intercept close_all during capture to close children first
///             return HandleResult::Handled;
///         }
///         HandleResult::Continue
///     }
///
///     fn id(&self) -> &str {
///         "container"
///     }
///
///     fn children(&self) -> &[Box<dyn ActionHandler>] {
///         &self.children
///     }
///
///     fn children_mut(&mut self) -> &mut [Box<dyn ActionHandler>] {
///         &mut self.children
///     }
/// }
/// ```
pub trait ActionHandler: Send + Sync {
    /// Handles an action during the specified propagation phase.
    ///
    /// # Arguments
    ///
    /// * `action` - The action being dispatched
    /// * `phase` - The current propagation phase (Capture or Bubble)
    ///
    /// # Returns
    ///
    /// A `HandleResult` indicating what should happen next:
    /// - `Handled`: Stop propagation, action was handled
    /// - `Continue`: Pass to the next handler
    /// - `Ignored`: Handler doesn't care about this action
    fn handle(&mut self, action: &Action, phase: Phase) -> HandleResult;

    /// Returns a unique identifier for this handler.
    ///
    /// This is used for debugging, logging, and potentially for
    /// targeting specific handlers in the hierarchy.
    fn id(&self) -> &str;

    /// Returns the child handlers of this handler.
    ///
    /// Override this to enable hierarchical action routing.
    /// The default implementation returns an empty slice.
    fn children(&self) -> &[Box<dyn ActionHandler>] {
        &[]
    }

    /// Returns a mutable reference to child handlers.
    ///
    /// Override this to enable hierarchical action routing.
    /// The default implementation returns an empty slice.
    fn children_mut(&mut self) -> &mut [Box<dyn ActionHandler>] {
        &mut []
    }

    /// Returns whether this handler is the current focus target.
    ///
    /// The focused handler is the "target" of the action propagation.
    /// The default implementation returns `false`.
    fn is_focused(&self) -> bool {
        false
    }

    /// Finds the path to the focused handler in the hierarchy.
    ///
    /// Returns a vector of indices representing the path from this handler
    /// to the focused descendant. Returns `None` if no descendant is focused.
    fn find_focus_path(&self) -> Option<Vec<usize>> {
        if self.is_focused() {
            return Some(vec![]);
        }

        for (i, child) in self.children().iter().enumerate() {
            if let Some(mut path) = child.find_focus_path() {
                path.insert(0, i);
                return Some(path);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_helpers() {
        assert!(Phase::Capture.is_capture());
        assert!(!Phase::Capture.is_bubble());

        assert!(Phase::Bubble.is_bubble());
        assert!(!Phase::Bubble.is_capture());
    }

    #[test]
    fn test_phase_display() {
        assert_eq!(format!("{}", Phase::Capture), "Capture");
        assert_eq!(format!("{}", Phase::Bubble), "Bubble");
    }

    #[test]
    fn test_handle_result_helpers() {
        assert!(HandleResult::Handled.should_stop());
        assert!(HandleResult::Handled.is_handled());
        assert!(!HandleResult::Handled.should_continue());

        assert!(!HandleResult::Continue.should_stop());
        assert!(!HandleResult::Continue.is_handled());
        assert!(HandleResult::Continue.should_continue());

        assert!(!HandleResult::Ignored.should_stop());
        assert!(HandleResult::Ignored.is_ignored());
        assert!(HandleResult::Ignored.should_continue());
    }

    #[test]
    fn test_handle_result_default() {
        assert_eq!(HandleResult::default(), HandleResult::Ignored);
    }

    #[test]
    fn test_handle_result_display() {
        assert_eq!(format!("{}", HandleResult::Continue), "Continue");
        assert_eq!(format!("{}", HandleResult::Handled), "Handled");
        assert_eq!(format!("{}", HandleResult::Ignored), "Ignored");
    }

    struct TestHandler {
        id: String,
        focused: bool,
        children: Vec<Box<dyn ActionHandler>>,
        handle_action: Option<String>,
    }

    impl TestHandler {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                focused: false,
                children: Vec::new(),
                handle_action: None,
            }
        }

        fn focused(mut self) -> Self {
            self.focused = true;
            self
        }

        fn handles(mut self, action: &str) -> Self {
            self.handle_action = Some(action.to_string());
            self
        }

        fn with_child(mut self, child: TestHandler) -> Self {
            self.children.push(Box::new(child));
            self
        }
    }

    impl ActionHandler for TestHandler {
        fn handle(&mut self, action: &Action, _phase: Phase) -> HandleResult {
            if let Some(ref handle_action) = self.handle_action {
                if action.name() == handle_action {
                    return HandleResult::Handled;
                }
            }
            HandleResult::Continue
        }

        fn id(&self) -> &str {
            &self.id
        }

        fn children(&self) -> &[Box<dyn ActionHandler>] {
            &self.children
        }

        fn children_mut(&mut self) -> &mut [Box<dyn ActionHandler>] {
            &mut self.children
        }

        fn is_focused(&self) -> bool {
            self.focused
        }
    }

    #[test]
    fn test_action_handler_trait() {
        let mut handler = TestHandler::new("test").handles("click");

        let click = Action::new("click");
        let other = Action::new("other");

        assert_eq!(handler.handle(&click, Phase::Bubble), HandleResult::Handled);
        assert_eq!(
            handler.handle(&other, Phase::Bubble),
            HandleResult::Continue
        );
    }

    #[test]
    fn test_find_focus_path_self() {
        let handler = TestHandler::new("root").focused();
        let path = handler.find_focus_path();
        assert_eq!(path, Some(vec![]));
    }

    #[test]
    fn test_find_focus_path_child() {
        let child = TestHandler::new("child").focused();
        let handler = TestHandler::new("root").with_child(child);

        let path = handler.find_focus_path();
        assert_eq!(path, Some(vec![0]));
    }

    #[test]
    fn test_find_focus_path_nested() {
        let grandchild = TestHandler::new("grandchild").focused();
        let child = TestHandler::new("child").with_child(grandchild);
        let handler = TestHandler::new("root").with_child(child);

        let path = handler.find_focus_path();
        assert_eq!(path, Some(vec![0, 0]));
    }

    #[test]
    fn test_find_focus_path_none() {
        let handler = TestHandler::new("root");
        let path = handler.find_focus_path();
        assert_eq!(path, None);
    }

    #[test]
    fn test_default_children() {
        struct SimpleHandler;

        impl ActionHandler for SimpleHandler {
            fn handle(&mut self, _action: &Action, _phase: Phase) -> HandleResult {
                HandleResult::Ignored
            }

            fn id(&self) -> &str {
                "simple"
            }
        }

        let handler = SimpleHandler;
        assert!(handler.children().is_empty());
    }
}
