//! Focusable trait for focus-aware TUI components.
//!
//! The [`Focusable`] trait defines the interface for components that can receive
//! and manage keyboard focus. This enables keyboard navigation between components
//! and allows components to render differently when focused.
//!
//! # Design Principles
//!
//! - **Explicit State**: Focus state is managed explicitly by the component
//! - **Customizable**: Components can define their own focusability rules
//! - **Ordered Navigation**: Focus order can be customized per component
//!
//! # Examples
//!
//! ```rust
//! use tuilib::components::Focusable;
//!
//! struct Button {
//!     label: String,
//!     focused: bool,
//! }
//!
//! impl Focusable for Button {
//!     fn is_focused(&self) -> bool {
//!         self.focused
//!     }
//!
//!     fn set_focused(&mut self, focused: bool) {
//!         self.focused = focused;
//!     }
//! }
//! ```

/// Trait for components that can receive and manage keyboard focus.
///
/// Implementing this trait allows components to participate in focus navigation.
/// When a component is focused, it typically receives keyboard input and may
/// render with visual focus indicators (e.g., highlighted borders).
///
/// # Default Implementations
///
/// - [`can_focus`](Focusable::can_focus): Returns `true` by default, indicating
///   the component can receive focus
/// - [`focus_order`](Focusable::focus_order): Returns `0` by default, used for
///   ordering focus navigation
///
/// # Examples
///
/// ## Basic Focusable Component
///
/// ```rust
/// use tuilib::components::Focusable;
///
/// struct TextInput {
///     value: String,
///     focused: bool,
/// }
///
/// impl TextInput {
///     fn new() -> Self {
///         Self {
///             value: String::new(),
///             focused: false,
///         }
///     }
/// }
///
/// impl Focusable for TextInput {
///     fn is_focused(&self) -> bool {
///         self.focused
///     }
///
///     fn set_focused(&mut self, focused: bool) {
///         self.focused = focused;
///     }
/// }
/// ```
///
/// ## Non-Focusable Component
///
/// Some components may want to opt out of focus navigation:
///
/// ```rust
/// use tuilib::components::Focusable;
///
/// struct Divider {
///     focused: bool,
/// }
///
/// impl Focusable for Divider {
///     fn is_focused(&self) -> bool {
///         self.focused
///     }
///
///     fn set_focused(&mut self, focused: bool) {
///         self.focused = focused;
///     }
///
///     fn can_focus(&self) -> bool {
///         false // Dividers cannot receive focus
///     }
/// }
/// ```
///
/// ## Custom Focus Order
///
/// ```rust
/// use tuilib::components::Focusable;
///
/// struct PriorityButton {
///     focused: bool,
///     priority: i32,
/// }
///
/// impl Focusable for PriorityButton {
///     fn is_focused(&self) -> bool {
///         self.focused
///     }
///
///     fn set_focused(&mut self, focused: bool) {
///         self.focused = focused;
///     }
///
///     fn focus_order(&self) -> i32 {
///         self.priority // Higher priority = earlier in focus order
///     }
/// }
/// ```
pub trait Focusable {
    /// Returns whether this component currently has focus.
    fn is_focused(&self) -> bool;

    /// Sets the focus state of this component.
    ///
    /// # Arguments
    ///
    /// * `focused` - `true` to give focus to this component, `false` to remove it
    fn set_focused(&mut self, focused: bool);

    /// Returns whether this component can receive focus.
    ///
    /// Components that return `false` will be skipped during focus navigation.
    /// This is useful for decorative or non-interactive components.
    ///
    /// # Default Implementation
    ///
    /// Returns `true`, meaning the component can receive focus.
    fn can_focus(&self) -> bool {
        true
    }

    /// Returns the focus order priority of this component.
    ///
    /// Lower values are focused first during tab navigation.
    /// Components with equal focus order are navigated in their natural order
    /// (typically the order they were added to the layout).
    ///
    /// # Default Implementation
    ///
    /// Returns `0`, giving the component default priority.
    fn focus_order(&self) -> i32 {
        0
    }

    /// Called when this component gains focus.
    ///
    /// Override this method to perform actions when focus is received,
    /// such as selecting text or starting animations.
    ///
    /// # Default Implementation
    ///
    /// Does nothing.
    fn on_focus(&mut self) {}

    /// Called when this component loses focus.
    ///
    /// Override this method to perform cleanup when focus is lost,
    /// such as validating input or stopping animations.
    ///
    /// # Default Implementation
    ///
    /// Does nothing.
    fn on_blur(&mut self) {}
}

/// A wrapper that adds focus functionality to any component.
///
/// This is useful for adding focusability to components that don't
/// inherently track focus state.
///
/// # Examples
///
/// ```rust
/// use tuilib::components::{FocusWrapper, Focusable};
///
/// struct SimpleLabel {
///     text: String,
/// }
///
/// let label = SimpleLabel { text: "Hello".into() };
/// let mut focusable_label = FocusWrapper::new(label);
///
/// assert!(!focusable_label.is_focused());
/// focusable_label.set_focused(true);
/// assert!(focusable_label.is_focused());
/// ```
#[derive(Debug, Clone)]
pub struct FocusWrapper<T> {
    inner: T,
    focused: bool,
    can_focus: bool,
    focus_order: i32,
}

impl<T> FocusWrapper<T> {
    /// Creates a new focus wrapper around the given component.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            focused: false,
            can_focus: true,
            focus_order: 0,
        }
    }

    /// Sets whether this wrapper can receive focus.
    pub fn with_can_focus(mut self, can_focus: bool) -> Self {
        self.can_focus = can_focus;
        self
    }

    /// Sets the focus order for this wrapper.
    pub fn with_focus_order(mut self, order: i32) -> Self {
        self.focus_order = order;
        self
    }

    /// Returns a reference to the inner component.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner component.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consumes the wrapper and returns the inner component.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Focusable for FocusWrapper<T> {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        self.can_focus
    }

    fn focus_order(&self) -> i32 {
        self.focus_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFocusable {
        focused: bool,
    }

    impl TestFocusable {
        fn new() -> Self {
            Self { focused: false }
        }
    }

    impl Focusable for TestFocusable {
        fn is_focused(&self) -> bool {
            self.focused
        }

        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
    }

    #[test]
    fn test_default_can_focus() {
        let f = TestFocusable::new();
        assert!(f.can_focus());
    }

    #[test]
    fn test_default_focus_order() {
        let f = TestFocusable::new();
        assert_eq!(f.focus_order(), 0);
    }

    #[test]
    fn test_focus_state() {
        let mut f = TestFocusable::new();
        assert!(!f.is_focused());

        f.set_focused(true);
        assert!(f.is_focused());

        f.set_focused(false);
        assert!(!f.is_focused());
    }

    #[test]
    fn test_focus_wrapper() {
        struct Inner;

        let mut wrapper = FocusWrapper::new(Inner);
        assert!(!wrapper.is_focused());
        assert!(wrapper.can_focus());
        assert_eq!(wrapper.focus_order(), 0);

        wrapper.set_focused(true);
        assert!(wrapper.is_focused());
    }

    #[test]
    fn test_focus_wrapper_builder() {
        struct Inner;

        let wrapper = FocusWrapper::new(Inner)
            .with_can_focus(false)
            .with_focus_order(10);

        assert!(!wrapper.can_focus());
        assert_eq!(wrapper.focus_order(), 10);
    }

    #[test]
    fn test_focus_wrapper_inner_access() {
        let wrapper = FocusWrapper::new(42);
        assert_eq!(*wrapper.inner(), 42);

        let mut wrapper = FocusWrapper::new(String::from("test"));
        wrapper.inner_mut().push('!');
        assert_eq!(wrapper.inner(), "test!");

        let inner = wrapper.into_inner();
        assert_eq!(inner, "test!");
    }
}
