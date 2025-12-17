//! Action type for named semantic actions.
//!
//! Actions represent semantic operations like "quit", "save", or "navigate_up"
//! that can be triggered by input events. This abstraction allows components
//! to handle actions without knowing the specific key bindings.
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::Action;
//!
//! let quit = Action::new("quit");
//! assert_eq!(quit.name(), "quit");
//!
//! // Actions can be compared
//! let quit2 = Action::new("quit");
//! assert_eq!(quit, quit2);
//!
//! // Common actions can be created using the constants
//! let save = Action::new("save");
//! let undo = Action::new("undo");
//! ```

use std::borrow::Cow;
use std::fmt;

/// A named action that components can handle.
///
/// Actions provide a semantic layer between raw input events and component
/// behavior. Instead of components checking for specific keys, they handle
/// named actions, making the code more readable and key bindings configurable.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::Action;
///
/// // Create actions
/// let quit = Action::new("quit");
/// let save = Action::new("save");
///
/// // Actions are hashable and can be used as map keys
/// use std::collections::HashMap;
/// let mut handlers: HashMap<Action, fn()> = HashMap::new();
/// handlers.insert(quit, || println!("Quitting..."));
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Action(Cow<'static, str>);

impl Action {
    /// Creates a new action with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the action (e.g., "quit", "save", "navigate_up")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::Action;
    ///
    /// let action = Action::new("my_action");
    /// assert_eq!(action.name(), "my_action");
    /// ```
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    /// Returns the name of this action.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::Action;
    ///
    /// let action = Action::new("quit");
    /// assert_eq!(action.name(), "quit");
    /// ```
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action(\"{}\")", self.0)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&'static str> for Action {
    fn from(name: &'static str) -> Self {
        Self::new(name)
    }
}

impl From<String> for Action {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_action_creation() {
        let action = Action::new("quit");
        assert_eq!(action.name(), "quit");
    }

    #[test]
    fn test_action_from_string() {
        let action = Action::new(String::from("save"));
        assert_eq!(action.name(), "save");
    }

    #[test]
    fn test_action_equality() {
        let a1 = Action::new("test");
        let a2 = Action::new("test");
        let a3 = Action::new("other");

        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }

    #[test]
    fn test_action_hash() {
        let mut map: HashMap<Action, i32> = HashMap::new();
        map.insert(Action::new("quit"), 1);
        map.insert(Action::new("save"), 2);

        assert_eq!(map.get(&Action::new("quit")), Some(&1));
        assert_eq!(map.get(&Action::new("save")), Some(&2));
        assert_eq!(map.get(&Action::new("unknown")), None);
    }

    #[test]
    fn test_action_from_static_str() {
        let action: Action = "quit".into();
        assert_eq!(action.name(), "quit");
    }

    #[test]
    fn test_action_from_owned_string() {
        let action: Action = String::from("save").into();
        assert_eq!(action.name(), "save");
    }

    #[test]
    fn test_action_debug() {
        let action = Action::new("test");
        let debug_str = format!("{:?}", action);
        assert_eq!(debug_str, "Action(\"test\")");
    }

    #[test]
    fn test_action_display() {
        let action = Action::new("quit");
        let display_str = format!("{}", action);
        assert_eq!(display_str, "quit");
    }

    #[test]
    fn test_action_clone() {
        let original = Action::new("test");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_action_as_ref() {
        let action = Action::new("test");
        let s: &str = action.as_ref();
        assert_eq!(s, "test");
    }
}
