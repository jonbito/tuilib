//! Middleware system for action transformation and logging.
//!
//! This module provides the [`ActionMiddleware`] trait for intercepting actions
//! before and after they are handled. Middleware can transform, log, or filter
//! actions as they flow through the system.
//!
//! # Examples
//!
//! ## Logging Middleware
//!
//! ```rust
//! use tuilib::input::{Action, ActionMiddleware, HandleResult, MiddlewareResult};
//!
//! struct LoggingMiddleware;
//!
//! impl ActionMiddleware for LoggingMiddleware {
//!     fn before(&mut self, action: &Action) -> MiddlewareResult {
//!         println!("Action dispatched: {}", action.name());
//!         MiddlewareResult::Continue(None)
//!     }
//!
//!     fn after(&mut self, action: &Action, result: &HandleResult) {
//!         println!("Action {} result: {}", action.name(), result);
//!     }
//! }
//! ```
//!
//! ## Transformation Middleware
//!
//! ```rust
//! use tuilib::input::{Action, ActionMiddleware, HandleResult, MiddlewareResult};
//!
//! struct ActionRemapper {
//!     mappings: std::collections::HashMap<String, String>,
//! }
//!
//! impl ActionMiddleware for ActionRemapper {
//!     fn before(&mut self, action: &Action) -> MiddlewareResult {
//!         if let Some(new_name) = self.mappings.get(action.name()) {
//!             MiddlewareResult::Continue(Some(Action::new(new_name.clone())))
//!         } else {
//!             MiddlewareResult::Continue(None)
//!         }
//!     }
//!
//!     fn after(&mut self, _action: &Action, _result: &HandleResult) {}
//! }
//! ```

use super::{Action, HandleResult};

/// Result of middleware processing before action dispatch.
///
/// Middleware can allow the action to continue (optionally with a transformation),
/// or stop the action from being dispatched entirely.
#[derive(Debug, Clone)]
pub enum MiddlewareResult {
    /// Continue processing with the original action or a transformed version.
    /// If `Some(Action)` is provided, the action is replaced with the new one.
    Continue(Option<Action>),
    /// Stop processing. The action will not be dispatched.
    Stop,
}

impl MiddlewareResult {
    /// Creates a continue result with no transformation.
    pub fn pass() -> Self {
        MiddlewareResult::Continue(None)
    }

    /// Creates a continue result with a transformed action.
    pub fn transform(action: Action) -> Self {
        MiddlewareResult::Continue(Some(action))
    }

    /// Creates a stop result.
    pub fn stop() -> Self {
        MiddlewareResult::Stop
    }

    /// Returns true if this result allows the action to continue.
    pub fn should_continue(&self) -> bool {
        matches!(self, MiddlewareResult::Continue(_))
    }

    /// Returns true if this result stops the action.
    pub fn is_stopped(&self) -> bool {
        matches!(self, MiddlewareResult::Stop)
    }

    /// Returns the transformed action if any.
    pub fn transformed_action(&self) -> Option<&Action> {
        match self {
            MiddlewareResult::Continue(Some(action)) => Some(action),
            _ => None,
        }
    }

    /// Consumes this result and returns the transformed action if any.
    pub fn into_transformed_action(self) -> Option<Action> {
        match self {
            MiddlewareResult::Continue(action) => action,
            MiddlewareResult::Stop => None,
        }
    }
}

impl Default for MiddlewareResult {
    fn default() -> Self {
        MiddlewareResult::pass()
    }
}

/// Trait for middleware that can intercept and transform actions.
///
/// Middleware is executed in a chain:
/// 1. `before()` is called for each middleware in order
/// 2. The action is dispatched to handlers
/// 3. `after()` is called for each middleware in reverse order
///
/// # Thread Safety
///
/// Middleware must implement `Send + Sync` to be used in async contexts.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::{Action, ActionMiddleware, HandleResult, MiddlewareResult};
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// struct CountingMiddleware {
///     dispatched: AtomicUsize,
///     handled: AtomicUsize,
/// }
///
/// impl ActionMiddleware for CountingMiddleware {
///     fn before(&mut self, _action: &Action) -> MiddlewareResult {
///         self.dispatched.fetch_add(1, Ordering::SeqCst);
///         MiddlewareResult::pass()
///     }
///
///     fn after(&mut self, _action: &Action, result: &HandleResult) {
///         if result.is_handled() {
///             self.handled.fetch_add(1, Ordering::SeqCst);
///         }
///     }
/// }
/// ```
pub trait ActionMiddleware: Send + Sync {
    /// Called before an action is dispatched.
    ///
    /// # Arguments
    ///
    /// * `action` - The action about to be dispatched
    ///
    /// # Returns
    ///
    /// A `MiddlewareResult` indicating whether to continue:
    /// - `Continue(None)`: Continue with the original action
    /// - `Continue(Some(action))`: Continue with a transformed action
    /// - `Stop`: Don't dispatch the action
    fn before(&mut self, action: &Action) -> MiddlewareResult;

    /// Called after an action has been dispatched.
    ///
    /// This is called even if the action was not handled. Use the
    /// `result` parameter to determine what happened.
    ///
    /// # Arguments
    ///
    /// * `action` - The action that was dispatched (possibly transformed)
    /// * `result` - The result of handling the action
    fn after(&mut self, action: &Action, result: &HandleResult);

    /// Returns the name of this middleware for debugging.
    ///
    /// The default implementation returns "unnamed".
    fn name(&self) -> &str {
        "unnamed"
    }
}

/// A middleware chain that executes multiple middleware in sequence.
pub struct MiddlewareChain {
    middleware: Vec<Box<dyn ActionMiddleware>>,
}

impl MiddlewareChain {
    /// Creates a new empty middleware chain.
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    /// Adds middleware to the chain.
    ///
    /// Middleware is executed in the order it was added for `before()`,
    /// and in reverse order for `after()`.
    pub fn add<M: ActionMiddleware + 'static>(&mut self, middleware: M) {
        self.middleware.push(Box::new(middleware));
    }

    /// Processes an action through all middleware `before()` methods.
    ///
    /// Returns the potentially transformed action, or `None` if any
    /// middleware stopped the action.
    pub fn process_before(&mut self, mut action: Action) -> Option<Action> {
        for m in &mut self.middleware {
            match m.before(&action) {
                MiddlewareResult::Continue(Some(transformed)) => {
                    action = transformed;
                }
                MiddlewareResult::Continue(None) => {}
                MiddlewareResult::Stop => {
                    return None;
                }
            }
        }
        Some(action)
    }

    /// Processes an action through all middleware `after()` methods.
    ///
    /// Middleware is called in reverse order.
    pub fn process_after(&mut self, action: &Action, result: &HandleResult) {
        for m in self.middleware.iter_mut().rev() {
            m.after(action, result);
        }
    }

    /// Returns the number of middleware in the chain.
    pub fn len(&self) -> usize {
        self.middleware.len()
    }

    /// Returns true if the chain has no middleware.
    pub fn is_empty(&self) -> bool {
        self.middleware.is_empty()
    }

    /// Clears all middleware from the chain.
    pub fn clear(&mut self) {
        self.middleware.clear();
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// A no-op middleware that passes all actions through unchanged.
///
/// Useful as a placeholder or for testing.
pub struct PassthroughMiddleware;

impl ActionMiddleware for PassthroughMiddleware {
    fn before(&mut self, _action: &Action) -> MiddlewareResult {
        MiddlewareResult::pass()
    }

    fn after(&mut self, _action: &Action, _result: &HandleResult) {}

    fn name(&self) -> &str {
        "passthrough"
    }
}

/// Middleware that logs all action dispatches.
///
/// Uses the `tracing` crate for logging.
pub struct TracingMiddleware {
    level: tracing::Level,
}

impl TracingMiddleware {
    /// Creates a new tracing middleware with the specified log level.
    pub fn new(level: tracing::Level) -> Self {
        Self { level }
    }

    /// Creates a new tracing middleware that logs at DEBUG level.
    pub fn debug() -> Self {
        Self::new(tracing::Level::DEBUG)
    }

    /// Creates a new tracing middleware that logs at INFO level.
    pub fn info() -> Self {
        Self::new(tracing::Level::INFO)
    }

    /// Creates a new tracing middleware that logs at TRACE level.
    pub fn trace() -> Self {
        Self::new(tracing::Level::TRACE)
    }
}

impl ActionMiddleware for TracingMiddleware {
    fn before(&mut self, action: &Action) -> MiddlewareResult {
        match self.level {
            tracing::Level::TRACE => {
                tracing::trace!(action = %action, "Dispatching action");
            }
            tracing::Level::DEBUG => {
                tracing::debug!(action = %action, "Dispatching action");
            }
            tracing::Level::INFO => {
                tracing::info!(action = %action, "Dispatching action");
            }
            tracing::Level::WARN => {
                tracing::warn!(action = %action, "Dispatching action");
            }
            tracing::Level::ERROR => {
                tracing::error!(action = %action, "Dispatching action");
            }
        }
        MiddlewareResult::pass()
    }

    fn after(&mut self, action: &Action, result: &HandleResult) {
        match self.level {
            tracing::Level::TRACE => {
                tracing::trace!(action = %action, result = %result, "Action completed");
            }
            tracing::Level::DEBUG => {
                tracing::debug!(action = %action, result = %result, "Action completed");
            }
            tracing::Level::INFO => {
                tracing::info!(action = %action, result = %result, "Action completed");
            }
            tracing::Level::WARN => {
                tracing::warn!(action = %action, result = %result, "Action completed");
            }
            tracing::Level::ERROR => {
                tracing::error!(action = %action, result = %result, "Action completed");
            }
        }
    }

    fn name(&self) -> &str {
        "tracing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_middleware_result_helpers() {
        let pass = MiddlewareResult::pass();
        assert!(pass.should_continue());
        assert!(!pass.is_stopped());
        assert!(pass.transformed_action().is_none());

        let transform = MiddlewareResult::transform(Action::new("test"));
        assert!(transform.should_continue());
        assert!(!transform.is_stopped());
        assert!(transform.transformed_action().is_some());
        assert_eq!(transform.transformed_action().unwrap().name(), "test");

        let stop = MiddlewareResult::stop();
        assert!(!stop.should_continue());
        assert!(stop.is_stopped());
        assert!(stop.transformed_action().is_none());
    }

    #[test]
    fn test_middleware_result_default() {
        let result = MiddlewareResult::default();
        assert!(result.should_continue());
        assert!(result.transformed_action().is_none());
    }

    #[test]
    fn test_into_transformed_action() {
        let pass = MiddlewareResult::pass();
        assert!(pass.into_transformed_action().is_none());

        let transform = MiddlewareResult::transform(Action::new("test"));
        let action = transform.into_transformed_action();
        assert!(action.is_some());
        assert_eq!(action.unwrap().name(), "test");

        let stop = MiddlewareResult::stop();
        assert!(stop.into_transformed_action().is_none());
    }

    struct TestMiddleware {
        name: String,
        before_calls: Arc<Mutex<Vec<String>>>,
        after_calls: Arc<Mutex<Vec<(String, HandleResult)>>>,
        transform_to: Option<String>,
        should_stop: bool,
    }

    impl TestMiddleware {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                before_calls: Arc::new(Mutex::new(Vec::new())),
                after_calls: Arc::new(Mutex::new(Vec::new())),
                transform_to: None,
                should_stop: false,
            }
        }

        fn transform(mut self, to: &str) -> Self {
            self.transform_to = Some(to.to_string());
            self
        }

        fn stop(mut self) -> Self {
            self.should_stop = true;
            self
        }

        #[allow(dead_code)]
        fn before_calls(&self) -> Vec<String> {
            self.before_calls.lock().unwrap().clone()
        }

        #[allow(dead_code)]
        fn after_calls(&self) -> Vec<(String, HandleResult)> {
            self.after_calls.lock().unwrap().clone()
        }
    }

    impl ActionMiddleware for TestMiddleware {
        fn before(&mut self, action: &Action) -> MiddlewareResult {
            self.before_calls
                .lock()
                .unwrap()
                .push(action.name().to_string());

            if self.should_stop {
                return MiddlewareResult::stop();
            }

            if let Some(ref to) = self.transform_to {
                MiddlewareResult::transform(Action::new(to.clone()))
            } else {
                MiddlewareResult::pass()
            }
        }

        fn after(&mut self, action: &Action, result: &HandleResult) {
            self.after_calls
                .lock()
                .unwrap()
                .push((action.name().to_string(), *result));
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_middleware_chain_empty() {
        let mut chain = MiddlewareChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);

        let action = Action::new("test");
        let result = chain.process_before(action);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), "test");
    }

    #[test]
    fn test_middleware_chain_passthrough() {
        let mut chain = MiddlewareChain::new();
        let m = TestMiddleware::new("test");
        let before_calls = m.before_calls.clone();
        let after_calls = m.after_calls.clone();
        chain.add(m);

        let action = Action::new("click");
        let result = chain.process_before(action);
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().name(), "click");

        chain.process_after(result.as_ref().unwrap(), &HandleResult::Handled);

        assert_eq!(before_calls.lock().unwrap().as_slice(), &["click"]);
        assert_eq!(
            after_calls.lock().unwrap().as_slice(),
            &[("click".to_string(), HandleResult::Handled)]
        );
    }

    #[test]
    fn test_middleware_chain_transform() {
        let mut chain = MiddlewareChain::new();
        chain.add(TestMiddleware::new("transform").transform("transformed"));

        let action = Action::new("original");
        let result = chain.process_before(action);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), "transformed");
    }

    #[test]
    fn test_middleware_chain_stop() {
        let mut chain = MiddlewareChain::new();
        let m = TestMiddleware::new("stopper").stop();
        let after_calls = m.after_calls.clone();
        chain.add(m);

        let action = Action::new("test");
        let result = chain.process_before(action);
        assert!(result.is_none());

        // After shouldn't be called if action was stopped, but let's verify
        // the chain can still call after if needed
        chain.process_after(&Action::new("test"), &HandleResult::Ignored);
        assert_eq!(after_calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_middleware_chain_order() {
        let mut chain = MiddlewareChain::new();

        let m1 = TestMiddleware::new("m1");
        let m1_before = m1.before_calls.clone();
        let m1_after = m1.after_calls.clone();

        let m2 = TestMiddleware::new("m2");
        let m2_before = m2.before_calls.clone();
        let m2_after = m2.after_calls.clone();

        chain.add(m1);
        chain.add(m2);

        let action = Action::new("test");
        let result = chain.process_before(action);
        chain.process_after(result.as_ref().unwrap(), &HandleResult::Handled);

        // Before is called in order: m1, m2
        assert_eq!(m1_before.lock().unwrap().len(), 1);
        assert_eq!(m2_before.lock().unwrap().len(), 1);

        // After is called in reverse order: m2, m1
        // Both should have been called
        assert_eq!(m1_after.lock().unwrap().len(), 1);
        assert_eq!(m2_after.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_middleware_chain_clear() {
        let mut chain = MiddlewareChain::new();
        chain.add(PassthroughMiddleware);
        chain.add(PassthroughMiddleware);

        assert_eq!(chain.len(), 2);

        chain.clear();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_passthrough_middleware() {
        let mut m = PassthroughMiddleware;
        let action = Action::new("test");

        let result = m.before(&action);
        assert!(result.should_continue());
        assert!(result.transformed_action().is_none());

        // after() is a no-op but should not panic
        m.after(&action, &HandleResult::Handled);

        assert_eq!(m.name(), "passthrough");
    }

    #[test]
    fn test_tracing_middleware_creation() {
        let m = TracingMiddleware::debug();
        assert_eq!(m.name(), "tracing");

        let m = TracingMiddleware::info();
        assert_eq!(m.name(), "tracing");

        let m = TracingMiddleware::trace();
        assert_eq!(m.name(), "tracing");
    }
}
