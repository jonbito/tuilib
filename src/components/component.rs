//! Component trait following the Elm architecture.
//!
//! The [`Component`] trait defines the core interface for interactive TUI components.
//! It follows the Elm architecture pattern with explicit state, messages, and actions,
//! providing a predictable and testable component model.
//!
//! # The Elm Architecture
//!
//! The Elm architecture consists of three main parts:
//!
//! 1. **State**: The data that describes the component's current condition
//! 2. **Message**: Events that describe what happened (user input, timer, etc.)
//! 3. **Update**: A function that takes a message and produces a new state
//!
//! In tuilib, we extend this with:
//!
//! 4. **Action**: Commands that the component wants the parent to execute
//!
//! # Examples
//!
//! ```rust
//! use tuilib::components::{Component, Renderable};
//! use ratatui::prelude::*;
//!
//! // A simple counter component
//! struct Counter {
//!     count: i32,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum CounterMsg {
//!     Increment,
//!     Decrement,
//! }
//!
//! #[derive(Debug)]
//! enum CounterAction {
//!     ValueChanged(i32),
//! }
//!
//! impl Component for Counter {
//!     type Message = CounterMsg;
//!     type Action = CounterAction;
//!
//!     fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
//!         match msg {
//!             CounterMsg::Increment => self.count += 1,
//!             CounterMsg::Decrement => self.count -= 1,
//!         }
//!         Some(CounterAction::ValueChanged(self.count))
//!     }
//! }
//!
//! impl Renderable for Counter {
//!     fn render(&self, frame: &mut Frame, area: Rect) {
//!         let text = format!("Count: {}", self.count);
//!         let paragraph = ratatui::widgets::Paragraph::new(text);
//!         frame.render_widget(paragraph, area);
//!     }
//! }
//! ```

use super::{Focusable, Renderable};

/// Main component trait following the Elm architecture.
///
/// Components encapsulate state, handle messages, and render themselves.
/// This trait provides the foundation for building interactive TUI components.
///
/// # Type Parameters
///
/// - `Message`: The type of messages this component can handle. Messages represent
///   events like user input, timer ticks, or data updates.
/// - `Action`: The type of actions this component can emit. Actions are commands
///   that the parent component or application should handle.
///
/// # Lifecycle
///
/// 1. Component is created with initial state
/// 2. Messages are dispatched to `update()`, which may return actions
/// 3. `render()` is called each frame to display the component
/// 4. Component may be destroyed when no longer needed
///
/// # Examples
///
/// ## Basic Component
///
/// ```rust
/// use tuilib::components::{Component, Renderable};
/// use ratatui::prelude::*;
///
/// struct Toggle {
///     enabled: bool,
/// }
///
/// #[derive(Debug, Clone)]
/// enum ToggleMsg {
///     Toggle,
///     SetEnabled(bool),
/// }
///
/// impl Component for Toggle {
///     type Message = ToggleMsg;
///     type Action = (); // No actions emitted
///
///     fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
///         match msg {
///             ToggleMsg::Toggle => self.enabled = !self.enabled,
///             ToggleMsg::SetEnabled(v) => self.enabled = v,
///         }
///         None
///     }
/// }
///
/// impl Renderable for Toggle {
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         let text = if self.enabled { "[x]" } else { "[ ]" };
///         let paragraph = ratatui::widgets::Paragraph::new(text);
///         frame.render_widget(paragraph, area);
///     }
/// }
/// ```
///
/// ## Component with Actions
///
/// ```rust
/// use tuilib::components::{Component, Renderable};
/// use ratatui::prelude::*;
///
/// struct SubmitButton {
///     label: String,
/// }
///
/// #[derive(Debug, Clone)]
/// enum ButtonMsg {
///     Press,
/// }
///
/// #[derive(Debug)]
/// enum ButtonAction {
///     Submitted,
/// }
///
/// impl Component for SubmitButton {
///     type Message = ButtonMsg;
///     type Action = ButtonAction;
///
///     fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
///         match msg {
///             ButtonMsg::Press => Some(ButtonAction::Submitted),
///         }
///     }
/// }
///
/// impl Renderable for SubmitButton {
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         let paragraph = ratatui::widgets::Paragraph::new(self.label.as_str());
///         frame.render_widget(paragraph, area);
///     }
/// }
/// ```
pub trait Component: Renderable {
    /// The type of messages this component handles.
    ///
    /// Messages represent events that can change the component's state.
    /// Common message types include user input events, timer events,
    /// and data updates.
    type Message;

    /// The type of actions this component can emit.
    ///
    /// Actions are commands that the component wants the parent to handle.
    /// Use `()` if the component doesn't emit any actions.
    type Action;

    /// Updates the component's state in response to a message.
    ///
    /// This method is the heart of the Elm architecture. It receives a message,
    /// updates the internal state, and optionally returns an action for the
    /// parent to handle.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process
    ///
    /// # Returns
    ///
    /// An optional action to be handled by the parent component or application.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tuilib::components::{Component, Renderable};
    /// use ratatui::prelude::*;
    ///
    /// struct TextInput {
    ///     value: String,
    /// }
    ///
    /// #[derive(Debug, Clone)]
    /// enum InputMsg {
    ///     SetValue(String),
    ///     Clear,
    /// }
    ///
    /// #[derive(Debug)]
    /// enum InputAction {
    ///     Changed(String),
    /// }
    ///
    /// impl Component for TextInput {
    ///     type Message = InputMsg;
    ///     type Action = InputAction;
    ///
    ///     fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
    ///         match msg {
    ///             InputMsg::SetValue(v) => {
    ///                 self.value = v;
    ///                 Some(InputAction::Changed(self.value.clone()))
    ///             }
    ///             InputMsg::Clear => {
    ///                 self.value.clear();
    ///                 Some(InputAction::Changed(String::new()))
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// impl Renderable for TextInput {
    ///     fn render(&self, frame: &mut Frame, area: Rect) {
    ///         let paragraph = ratatui::widgets::Paragraph::new(self.value.as_str());
    ///         frame.render_widget(paragraph, area);
    ///     }
    /// }
    /// ```
    fn update(&mut self, msg: Self::Message) -> Option<Self::Action>;
}

/// A component that also supports focus management.
///
/// This is a convenience trait for components that are both interactive
/// (implementing [`Component`]) and focusable (implementing [`Focusable`]).
///
/// # Examples
///
/// ```rust
/// use tuilib::components::{Component, Focusable, FocusableComponent, Renderable};
/// use ratatui::prelude::*;
///
/// struct FocusableInput {
///     value: String,
///     focused: bool,
/// }
///
/// impl Focusable for FocusableInput {
///     fn is_focused(&self) -> bool { self.focused }
///     fn set_focused(&mut self, focused: bool) { self.focused = focused; }
/// }
///
/// impl Component for FocusableInput {
///     type Message = ();
///     type Action = ();
///     fn update(&mut self, _msg: Self::Message) -> Option<Self::Action> { None }
/// }
///
/// impl Renderable for FocusableInput {
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         let style = if self.focused {
///             Style::default().fg(Color::Yellow)
///         } else {
///             Style::default()
///         };
///         let paragraph = ratatui::widgets::Paragraph::new(self.value.as_str())
///             .style(style);
///         frame.render_widget(paragraph, area);
///     }
/// }
///
/// // FocusableInput automatically implements FocusableComponent
/// fn process_focusable<C: FocusableComponent>(component: &mut C) {
///     component.set_focused(true);
/// }
/// ```
pub trait FocusableComponent: Component + Focusable {}

// Blanket implementation for any type implementing both Component and Focusable
impl<T> FocusableComponent for T where T: Component + Focusable {}

/// Marker trait for stateless components.
///
/// Stateless components don't maintain internal state and only render
/// based on their properties. They typically don't handle messages.
///
/// This is useful for simple presentational components like labels,
/// icons, or decorative elements.
pub trait StatelessComponent: Renderable {}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::*;

    struct TestComponent {
        value: i32,
    }

    #[derive(Debug, Clone)]
    enum TestMsg {
        Set(i32),
        Add(i32),
    }

    #[derive(Debug, PartialEq)]
    enum TestAction {
        Changed(i32),
    }

    impl Component for TestComponent {
        type Message = TestMsg;
        type Action = TestAction;

        fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
            match msg {
                TestMsg::Set(v) => {
                    self.value = v;
                    Some(TestAction::Changed(self.value))
                }
                TestMsg::Add(v) => {
                    self.value += v;
                    Some(TestAction::Changed(self.value))
                }
            }
        }
    }

    impl Renderable for TestComponent {
        fn render(&self, _frame: &mut Frame, _area: Rect) {
            // Test implementation - no actual rendering
        }
    }

    #[test]
    fn test_component_update() {
        let mut component = TestComponent { value: 0 };

        let action = component.update(TestMsg::Set(10));
        assert_eq!(component.value, 10);
        assert_eq!(action, Some(TestAction::Changed(10)));

        let action = component.update(TestMsg::Add(5));
        assert_eq!(component.value, 15);
        assert_eq!(action, Some(TestAction::Changed(15)));
    }

    struct NoActionComponent;

    impl Component for NoActionComponent {
        type Message = ();
        type Action = ();

        fn update(&mut self, _msg: Self::Message) -> Option<Self::Action> {
            None
        }
    }

    impl Renderable for NoActionComponent {
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
    }

    #[test]
    fn test_no_action_component() {
        let mut component = NoActionComponent;
        let action = component.update(());
        assert!(action.is_none());
    }

    struct FocusTestComponent {
        focused: bool,
    }

    impl Focusable for FocusTestComponent {
        fn is_focused(&self) -> bool {
            self.focused
        }

        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
    }

    impl Component for FocusTestComponent {
        type Message = ();
        type Action = ();

        fn update(&mut self, _msg: Self::Message) -> Option<Self::Action> {
            None
        }
    }

    impl Renderable for FocusTestComponent {
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
    }

    #[test]
    fn test_focusable_component_trait() {
        let mut component = FocusTestComponent { focused: false };

        // Test that FocusableComponent is automatically implemented
        fn takes_focusable<T: FocusableComponent>(_: &T) {}
        takes_focusable(&component);

        // Test focus functionality through the trait
        assert!(!component.is_focused());
        component.set_focused(true);
        assert!(component.is_focused());
    }
}
