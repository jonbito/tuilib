//! Focus identifier type for unique component identification.
//!
//! The [`FocusId`] type provides a unique identifier for focusable components
//! in the TUI application. It supports creation from strings, static strings,
//! and other common types.

use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;

/// Unique identifier for focusable components.
///
/// A `FocusId` identifies a component in the focus system. Each focusable
/// component should have a unique `FocusId` to enable proper focus navigation
/// and programmatic focus control.
///
/// # Creating Focus IDs
///
/// ```rust
/// use tuilib::focus::FocusId;
///
/// // From a string literal (zero-allocation)
/// let id1 = FocusId::new("submit-button");
///
/// // From a String
/// let id2 = FocusId::from(String::from("cancel-button"));
///
/// // Using the From trait
/// let id3: FocusId = "input-field".into();
/// ```
///
/// # Comparison
///
/// Focus IDs can be compared for equality:
///
/// ```rust
/// use tuilib::focus::FocusId;
///
/// let id1 = FocusId::new("button");
/// let id2 = FocusId::new("button");
/// let id3 = FocusId::new("input");
///
/// assert_eq!(id1, id2);
/// assert_ne!(id1, id3);
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FocusId(Cow<'static, str>);

impl FocusId {
    /// Creates a new `FocusId` from a static string.
    ///
    /// This is the preferred way to create focus IDs from string literals
    /// as it avoids allocation.
    ///
    /// # Arguments
    ///
    /// * `id` - A static string identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::FocusId;
    ///
    /// let id = FocusId::new("my-component");
    /// assert_eq!(id.as_str(), "my-component");
    /// ```
    pub const fn new(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Returns the identifier as a string slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::FocusId;
    ///
    /// let id = FocusId::new("button-1");
    /// assert_eq!(id.as_str(), "button-1");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for FocusId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FocusId({:?})", self.0)
    }
}

impl fmt::Display for FocusId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&'static str> for FocusId {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl From<String> for FocusId {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl AsRef<str> for FocusId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for FocusId {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl PartialEq<&str> for FocusId {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_from_static() {
        let id = FocusId::new("test-id");
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_from_string() {
        let id = FocusId::from(String::from("dynamic-id"));
        assert_eq!(id.as_str(), "dynamic-id");
    }

    #[test]
    fn test_from_str() {
        let id: FocusId = "static-id".into();
        assert_eq!(id.as_str(), "static-id");
    }

    #[test]
    fn test_equality() {
        let id1 = FocusId::new("same");
        let id2 = FocusId::new("same");
        let id3 = FocusId::new("different");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_equality_with_str() {
        let id = FocusId::new("test");
        assert_eq!(id, "test");
        assert_ne!(id, "other");
    }

    #[test]
    fn test_debug_display() {
        let id = FocusId::new("debug-test");
        assert_eq!(format!("{:?}", id), "FocusId(\"debug-test\")");
        assert_eq!(format!("{}", id), "debug-test");
    }

    #[test]
    fn test_hash() {
        let id1 = FocusId::new("hash-test");
        let id2 = FocusId::new("hash-test");

        let mut set = HashSet::new();
        set.insert(id1.clone());
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }

    #[test]
    fn test_clone() {
        let id1 = FocusId::new("original");
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_as_ref() {
        let id = FocusId::new("ref-test");
        let s: &str = id.as_ref();
        assert_eq!(s, "ref-test");
    }
}
