//! Action router for dispatching actions through the component hierarchy.
//!
//! The [`ActionRouter`] manages action dispatch with support for:
//! - Capture and bubble phases (similar to DOM events)
//! - Stop propagation
//! - Middleware for logging and transformation
//! - Async handler support
//!
//! # Event Propagation Model
//!
//! Actions flow through the component tree in two phases:
//!
//! 1. **Capture Phase**: Action travels from root → target
//! 2. **Bubble Phase**: Action travels from target → root
//!
//! ```text
//!                    ┌─────────┐
//!                    │  Root   │ ←───┐
//!                    └────┬────┘     │
//!        Capture ↓        │          │ Bubble ↑
//!                    ┌────▼────┐     │
//!                    │ Parent  │ ────┤
//!                    └────┬────┘     │
//!        Capture ↓        │          │ Bubble ↑
//!                    ┌────▼────┐     │
//!                    │ Target  │ ────┘
//!                    └─────────┘
//! ```
//!
//! # Examples
//!
//! ## Basic Dispatch
//!
//! ```rust
//! use tuilib::input::{Action, ActionRouter, ActionHandler, Phase, HandleResult};
//!
//! struct App {
//!     id: String,
//!     children: Vec<Box<dyn ActionHandler>>,
//! }
//!
//! impl ActionHandler for App {
//!     fn handle(&mut self, action: &Action, phase: Phase) -> HandleResult {
//!         if phase == Phase::Bubble && action.name() == "quit" {
//!             return HandleResult::Handled;
//!         }
//!         HandleResult::Continue
//!     }
//!
//!     fn id(&self) -> &str { &self.id }
//!     fn children(&self) -> &[Box<dyn ActionHandler>] { &self.children }
//!     fn children_mut(&mut self) -> &mut [Box<dyn ActionHandler>] { &mut self.children }
//! }
//!
//! let mut router = ActionRouter::new();
//! let mut app = App { id: "app".to_string(), children: vec![] };
//! let result = router.dispatch(&mut app, Action::new("quit"));
//! ```
//!
//! ## With Middleware
//!
//! ```rust
//! use tuilib::input::{ActionRouter, TracingMiddleware};
//!
//! let mut router = ActionRouter::new();
//! router.add_middleware(TracingMiddleware::debug());
//! ```

use std::future::Future;
use std::pin::Pin;

use super::handler::{HandleResult, Phase};
use super::middleware::{ActionMiddleware, MiddlewareChain};
use super::Action;
use super::ActionHandler;

/// Result of dispatching an action through the router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchResult {
    /// The final handle result from the dispatch.
    pub result: HandleResult,
    /// The handler ID that handled the action, if any.
    pub handled_by: Option<String>,
    /// The phase in which the action was handled.
    pub handled_in: Option<Phase>,
    /// Whether propagation was stopped.
    pub propagation_stopped: bool,
}

impl DispatchResult {
    /// Creates a new dispatch result indicating the action was not handled.
    pub fn not_handled() -> Self {
        Self {
            result: HandleResult::Ignored,
            handled_by: None,
            handled_in: None,
            propagation_stopped: false,
        }
    }

    /// Creates a dispatch result indicating the action was handled.
    pub fn handled(by: &str, phase: Phase) -> Self {
        Self {
            result: HandleResult::Handled,
            handled_by: Some(by.to_string()),
            handled_in: Some(phase),
            propagation_stopped: true,
        }
    }

    /// Returns true if the action was handled.
    pub fn was_handled(&self) -> bool {
        self.result.is_handled()
    }
}

/// Routes actions through a component hierarchy with capture/bubble propagation.
///
/// The router manages:
/// - Two-phase propagation (capture and bubble)
/// - Middleware chain for action transformation
/// - Handler registration and dispatch
///
/// # Example
///
/// ```rust
/// use tuilib::input::{ActionRouter, TracingMiddleware};
///
/// let mut router = ActionRouter::new();
///
/// // Add middleware
/// router.add_middleware(TracingMiddleware::debug());
/// ```
pub struct ActionRouter {
    middleware: MiddlewareChain,
}

impl ActionRouter {
    /// Creates a new action router.
    pub fn new() -> Self {
        Self {
            middleware: MiddlewareChain::new(),
        }
    }

    /// Adds middleware to the router.
    ///
    /// Middleware is executed in the order it was added.
    pub fn add_middleware<M: ActionMiddleware + 'static>(&mut self, middleware: M) {
        self.middleware.add(middleware);
    }

    /// Dispatches an action to a handler tree.
    ///
    /// The action flows through the tree in two phases:
    /// 1. Capture: Root → Target (following focus path)
    /// 2. Bubble: Target → Root
    ///
    /// # Arguments
    ///
    /// * `root` - The root handler of the component tree
    /// * `action` - The action to dispatch
    ///
    /// # Returns
    ///
    /// A `DispatchResult` containing information about how the action was handled.
    pub fn dispatch(&mut self, root: &mut dyn ActionHandler, action: Action) -> DispatchResult {
        // Process through middleware
        let action = match self.middleware.process_before(action) {
            Some(a) => a,
            None => return DispatchResult::not_handled(),
        };

        // Find the focus path
        let focus_path = root.find_focus_path().unwrap_or_default();

        // Perform the dispatch
        let result = self.dispatch_internal(root, &action, &focus_path);

        // Process through middleware after
        self.middleware.process_after(&action, &result.result);

        result
    }

    /// Internal dispatch implementation that handles the two-phase propagation.
    fn dispatch_internal(
        &self,
        root: &mut dyn ActionHandler,
        action: &Action,
        focus_path: &[usize],
    ) -> DispatchResult {
        // Capture phase: root → target
        if let Some(result) = self.capture_phase(root, action, focus_path, 0) {
            return result;
        }

        // Bubble phase: target → root
        self.bubble_phase(root, action, focus_path, 0)
    }

    /// Capture phase: dispatches from root toward target.
    fn capture_phase(
        &self,
        handler: &mut dyn ActionHandler,
        action: &Action,
        focus_path: &[usize],
        depth: usize,
    ) -> Option<DispatchResult> {
        // Handle at current node
        let result = handler.handle(action, Phase::Capture);
        if result.should_stop() {
            return Some(DispatchResult::handled(handler.id(), Phase::Capture));
        }

        // Continue to child if we have more path to follow
        if depth < focus_path.len() {
            let child_idx = focus_path[depth];
            let children = handler.children_mut();
            if child_idx < children.len() {
                return self.capture_phase(
                    &mut *children[child_idx],
                    action,
                    focus_path,
                    depth + 1,
                );
            }
        }

        None
    }

    /// Bubble phase: dispatches from target back to root.
    fn bubble_phase(
        &self,
        handler: &mut dyn ActionHandler,
        action: &Action,
        focus_path: &[usize],
        depth: usize,
    ) -> DispatchResult {
        // First, recurse to child if we have more path
        if depth < focus_path.len() {
            let child_idx = focus_path[depth];
            let children = handler.children_mut();
            if child_idx < children.len() {
                let result =
                    self.bubble_phase(&mut *children[child_idx], action, focus_path, depth + 1);
                if result.was_handled() {
                    return result;
                }
            }
        }

        // Handle at current node
        let result = handler.handle(action, Phase::Bubble);
        if result.should_stop() {
            return DispatchResult::handled(handler.id(), Phase::Bubble);
        }

        DispatchResult::not_handled()
    }

    /// Dispatches an action asynchronously.
    ///
    /// This version supports async handlers and returns a future that
    /// resolves when all handlers have completed.
    ///
    /// # Arguments
    ///
    /// * `root` - The root handler of the component tree
    /// * `action` - The action to dispatch
    ///
    /// # Returns
    ///
    /// A future that resolves to a `DispatchResult`.
    pub fn dispatch_async<'a>(
        &'a mut self,
        root: &'a mut dyn ActionHandler,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = DispatchResult> + Send + 'a>> {
        Box::pin(async move {
            // For now, the async version just wraps the sync version
            // In the future, this could support truly async handlers
            self.dispatch(root, action)
        })
    }

    /// Dispatches an action to a specific handler path without focus-based routing.
    ///
    /// This method allows dispatching to a specific path in the tree,
    /// useful for programmatic action dispatch.
    ///
    /// # Arguments
    ///
    /// * `root` - The root handler
    /// * `action` - The action to dispatch
    /// * `path` - The path to the target handler (indices into children)
    pub fn dispatch_to_path(
        &mut self,
        root: &mut dyn ActionHandler,
        action: Action,
        path: &[usize],
    ) -> DispatchResult {
        // Process through middleware
        let action = match self.middleware.process_before(action) {
            Some(a) => a,
            None => return DispatchResult::not_handled(),
        };

        // Dispatch using the provided path
        let result = self.dispatch_internal(root, &action, path);

        // Process through middleware after
        self.middleware.process_after(&action, &result.result);

        result
    }
}

impl Default for ActionRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ActionRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionRouter")
            .field("middleware_count", &self.middleware.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestHandler {
        id: String,
        focused: bool,
        children: Vec<Box<dyn ActionHandler>>,
        handle_in_capture: Option<String>,
        handle_in_bubble: Option<String>,
        calls: Arc<Mutex<Vec<(String, Phase)>>>,
    }

    impl TestHandler {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                focused: false,
                children: Vec::new(),
                handle_in_capture: None,
                handle_in_bubble: None,
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn focused(mut self) -> Self {
            self.focused = true;
            self
        }

        fn handles_capture(mut self, action: &str) -> Self {
            self.handle_in_capture = Some(action.to_string());
            self
        }

        fn handles_bubble(mut self, action: &str) -> Self {
            self.handle_in_bubble = Some(action.to_string());
            self
        }

        fn with_child(mut self, child: TestHandler) -> Self {
            self.children.push(Box::new(child));
            self
        }

        #[allow(dead_code)]
        fn calls(&self) -> Vec<(String, Phase)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl ActionHandler for TestHandler {
        fn handle(&mut self, action: &Action, phase: Phase) -> HandleResult {
            self.calls.lock().unwrap().push((self.id.clone(), phase));

            match phase {
                Phase::Capture => {
                    if let Some(ref handle_action) = self.handle_in_capture {
                        if action.name() == handle_action {
                            return HandleResult::Handled;
                        }
                    }
                }
                Phase::Bubble => {
                    if let Some(ref handle_action) = self.handle_in_bubble {
                        if action.name() == handle_action {
                            return HandleResult::Handled;
                        }
                    }
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
    fn test_dispatch_result_helpers() {
        let not_handled = DispatchResult::not_handled();
        assert!(!not_handled.was_handled());
        assert!(not_handled.handled_by.is_none());
        assert!(!not_handled.propagation_stopped);

        let handled = DispatchResult::handled("test", Phase::Bubble);
        assert!(handled.was_handled());
        assert_eq!(handled.handled_by.as_deref(), Some("test"));
        assert_eq!(handled.handled_in, Some(Phase::Bubble));
        assert!(handled.propagation_stopped);
    }

    #[test]
    fn test_router_dispatch_to_root() {
        let mut router = ActionRouter::new();
        let mut handler = TestHandler::new("root").handles_bubble("click");

        let result = router.dispatch(&mut handler, Action::new("click"));

        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("root"));
        assert_eq!(result.handled_in, Some(Phase::Bubble));
    }

    #[test]
    fn test_router_capture_phase() {
        let mut router = ActionRouter::new();

        let child = TestHandler::new("child").focused();
        let calls = child.calls.clone();
        let mut handler = TestHandler::new("root")
            .handles_capture("intercept")
            .with_child(child);

        let result = router.dispatch(&mut handler, Action::new("intercept"));

        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("root"));
        assert_eq!(result.handled_in, Some(Phase::Capture));

        // The child should not have been called since root captured it
        let recorded_calls = calls.lock().unwrap();
        assert!(!recorded_calls.iter().any(|(id, _)| id == "child"));
    }

    #[test]
    fn test_router_bubble_phase() {
        let mut router = ActionRouter::new();

        let child = TestHandler::new("child").focused().handles_bubble("click");
        let child_calls = child.calls.clone();
        let mut handler = TestHandler::new("root").with_child(child);

        let result = router.dispatch(&mut handler, Action::new("click"));

        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("child"));
        assert_eq!(result.handled_in, Some(Phase::Bubble));

        // Both should have been called during capture, but bubble stopped at child
        let recorded = child_calls.lock().unwrap();
        assert!(recorded
            .iter()
            .any(|(id, phase)| id == "child" && *phase == Phase::Bubble));
    }

    #[test]
    fn test_router_propagation_order() {
        let mut router = ActionRouter::new();

        // Create a shared call tracker for all handlers
        let shared_calls: Arc<Mutex<Vec<(String, Phase)>>> = Arc::new(Mutex::new(Vec::new()));

        let mut grandchild = TestHandler::new("grandchild").focused();
        grandchild.calls = shared_calls.clone();

        let mut child = TestHandler::new("child");
        child.calls = shared_calls.clone();
        child.children.push(Box::new(grandchild));

        let mut root = TestHandler::new("root");
        root.calls = shared_calls.clone();
        root.children.push(Box::new(child));

        router.dispatch(&mut root, Action::new("test"));

        let recorded = shared_calls.lock().unwrap();
        let phases: Vec<_> = recorded
            .iter()
            .map(|(id, phase)| (id.as_str(), *phase))
            .collect();

        // Should be: root(capture), child(capture), grandchild(capture),
        //            grandchild(bubble), child(bubble), root(bubble)
        assert!(
            phases
                .iter()
                .position(|x| x == &("root", Phase::Capture))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("root", Phase::Bubble))
                    .unwrap()
        );
        assert!(
            phases
                .iter()
                .position(|x| x == &("child", Phase::Capture))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("child", Phase::Bubble))
                    .unwrap()
        );
        assert!(
            phases
                .iter()
                .position(|x| x == &("grandchild", Phase::Capture))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("grandchild", Phase::Bubble))
                    .unwrap()
        );

        // Verify capture order: root -> child -> grandchild
        assert!(
            phases
                .iter()
                .position(|x| x == &("root", Phase::Capture))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("child", Phase::Capture))
                    .unwrap()
        );
        assert!(
            phases
                .iter()
                .position(|x| x == &("child", Phase::Capture))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("grandchild", Phase::Capture))
                    .unwrap()
        );

        // Verify bubble order: grandchild -> child -> root
        assert!(
            phases
                .iter()
                .position(|x| x == &("grandchild", Phase::Bubble))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("child", Phase::Bubble))
                    .unwrap()
        );
        assert!(
            phases
                .iter()
                .position(|x| x == &("child", Phase::Bubble))
                .unwrap()
                < phases
                    .iter()
                    .position(|x| x == &("root", Phase::Bubble))
                    .unwrap()
        );
    }

    #[test]
    fn test_router_no_focus() {
        let mut router = ActionRouter::new();
        let mut handler = TestHandler::new("root").handles_bubble("click");

        // No focus set, so should still dispatch to root
        let result = router.dispatch(&mut handler, Action::new("click"));

        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("root"));
    }

    #[test]
    fn test_router_with_middleware() {
        use super::super::middleware::PassthroughMiddleware;

        let mut router = ActionRouter::new();
        router.add_middleware(PassthroughMiddleware);

        let mut handler = TestHandler::new("root").handles_bubble("click");
        let result = router.dispatch(&mut handler, Action::new("click"));

        assert!(result.was_handled());
    }

    #[test]
    fn test_dispatch_to_path() {
        let mut router = ActionRouter::new();

        let child = TestHandler::new("child").handles_bubble("click");
        let mut root = TestHandler::new("root").with_child(child);

        // Dispatch directly to child via path
        let result = router.dispatch_to_path(&mut root, Action::new("click"), &[0]);

        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("child"));
    }

    #[test]
    fn test_router_default() {
        let router = ActionRouter::default();
        assert_eq!(router.middleware.len(), 0);
    }

    #[test]
    fn test_router_debug() {
        let mut router = ActionRouter::new();
        router.add_middleware(super::super::middleware::PassthroughMiddleware);

        let debug_str = format!("{:?}", router);
        assert!(debug_str.contains("ActionRouter"));
        assert!(debug_str.contains("middleware_count"));
    }

    #[tokio::test]
    async fn test_dispatch_async() {
        let mut router = ActionRouter::new();
        let mut handler = TestHandler::new("root").handles_bubble("click");

        let result = router
            .dispatch_async(&mut handler, Action::new("click"))
            .await;

        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("root"));
    }

    #[test]
    fn test_nested_hierarchy() {
        let mut router = ActionRouter::new();

        // Create a 3-level hierarchy with focus on deepest node
        let leaf = TestHandler::new("leaf")
            .focused()
            .handles_bubble("leaf_action");
        let middle = TestHandler::new("middle")
            .handles_bubble("middle_action")
            .with_child(leaf);
        let mut root = TestHandler::new("root")
            .handles_bubble("root_action")
            .with_child(middle);

        // Action handled at leaf
        let result = router.dispatch(&mut root, Action::new("leaf_action"));
        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("leaf"));

        // Rebuild tree for next test
        let leaf = TestHandler::new("leaf").focused();
        let middle = TestHandler::new("middle")
            .handles_bubble("middle_action")
            .with_child(leaf);
        let mut root = TestHandler::new("root")
            .handles_bubble("root_action")
            .with_child(middle);

        // Action handled at middle (bubbles up from leaf)
        let result = router.dispatch(&mut root, Action::new("middle_action"));
        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("middle"));

        // Rebuild tree for next test
        let leaf = TestHandler::new("leaf").focused();
        let middle = TestHandler::new("middle").with_child(leaf);
        let mut root = TestHandler::new("root")
            .handles_bubble("root_action")
            .with_child(middle);

        // Action handled at root (bubbles all the way up)
        let result = router.dispatch(&mut root, Action::new("root_action"));
        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("root"));
    }

    #[test]
    fn test_capture_intercepts_before_bubble() {
        let mut router = ActionRouter::new();

        let leaf = TestHandler::new("leaf").focused().handles_bubble("test");
        let mut root = TestHandler::new("root")
            .handles_capture("test")
            .with_child(leaf);

        let result = router.dispatch(&mut root, Action::new("test"));

        // Root should capture it before leaf can bubble-handle it
        assert!(result.was_handled());
        assert_eq!(result.handled_by.as_deref(), Some("root"));
        assert_eq!(result.handled_in, Some(Phase::Capture));
    }
}
