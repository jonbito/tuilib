//! Focus trap for modal dialogs.
//!
//! A [`FocusTrap`] restricts focus navigation to a subset of components,
//! typically used for modal dialogs to prevent users from tabbing out of
//! the modal while it's open.

use super::{FocusId, FocusRing};

/// A focus trap that restricts navigation to a specific set of components.
///
/// Focus traps are used to contain keyboard navigation within a specific
/// region of the UI, such as a modal dialog. When a trap is active, Tab
/// and Shift+Tab navigation only cycles through the components in the trap.
///
/// # Modal Dialog Example
///
/// ```rust
/// use tuilib::focus::{FocusId, FocusTrap};
///
/// // Create a trap for a confirmation dialog
/// let mut trap = FocusTrap::new();
/// trap.register(FocusId::new("confirm-btn"), 0);
/// trap.register(FocusId::new("cancel-btn"), 0);
///
/// // Navigation is restricted to these two buttons
/// assert_eq!(trap.next(), Some(FocusId::new("confirm-btn")));
/// assert_eq!(trap.next(), Some(FocusId::new("cancel-btn")));
/// assert_eq!(trap.next(), Some(FocusId::new("confirm-btn"))); // Cycles
/// ```
///
/// # With Saved Focus
///
/// ```rust
/// use tuilib::focus::{FocusId, FocusTrap};
///
/// // Save the previously focused element
/// let trap = FocusTrap::with_saved_focus(FocusId::new("main-input"));
///
/// // Later, restore focus when the trap is popped
/// assert_eq!(trap.saved_focus(), Some(&FocusId::new("main-input")));
/// ```
#[derive(Debug, Clone)]
pub struct FocusTrap {
    ring: FocusRing,
    saved_focus: Option<FocusId>,
}

impl Default for FocusTrap {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusTrap {
    /// Creates a new empty focus trap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::FocusTrap;
    ///
    /// let trap = FocusTrap::new();
    /// assert!(trap.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            ring: FocusRing::new(),
            saved_focus: None,
        }
    }

    /// Creates a new focus trap with a saved focus ID.
    ///
    /// The saved focus ID is used to restore focus when the trap is removed.
    ///
    /// # Arguments
    ///
    /// * `saved` - The ID of the component that had focus before the trap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusTrap};
    ///
    /// let trap = FocusTrap::with_saved_focus(FocusId::new("original"));
    /// assert_eq!(trap.saved_focus(), Some(&FocusId::new("original")));
    /// ```
    pub fn with_saved_focus(saved: FocusId) -> Self {
        Self {
            ring: FocusRing::new(),
            saved_focus: Some(saved),
        }
    }

    /// Returns `true` if the trap has no registered components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusTrap};
    ///
    /// let mut trap = FocusTrap::new();
    /// assert!(trap.is_empty());
    ///
    /// trap.register(FocusId::new("button"), 0);
    /// assert!(!trap.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Returns the number of components in the trap.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Registers a focusable component in the trap.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the component
    /// * `order` - The focus order priority (lower = earlier in tab order)
    pub fn register(&mut self, id: FocusId, order: i32) {
        self.ring.register(id, order);
    }

    /// Unregisters a component from the trap.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the component to unregister
    ///
    /// # Returns
    ///
    /// `true` if the component was found and removed.
    pub fn unregister(&mut self, id: &FocusId) -> bool {
        self.ring.unregister(id)
    }

    /// Moves focus to the next component in the trap.
    ///
    /// # Returns
    ///
    /// The ID of the newly focused component, or `None` if the trap is empty.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<FocusId> {
        self.ring.next()
    }

    /// Moves focus to the previous component in the trap.
    ///
    /// # Returns
    ///
    /// The ID of the newly focused component, or `None` if the trap is empty.
    pub fn prev(&mut self) -> Option<FocusId> {
        self.ring.prev()
    }

    /// Focuses a specific component by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the component to focus
    ///
    /// # Returns
    ///
    /// `true` if the component was found and focused.
    pub fn focus(&mut self, id: &FocusId) -> bool {
        self.ring.focus(id)
    }

    /// Returns a reference to the currently focused component's ID.
    pub fn current(&self) -> Option<&FocusId> {
        self.ring.current()
    }

    /// Returns the saved focus ID, if any.
    ///
    /// This is the ID of the component that had focus before the trap was created.
    pub fn saved_focus(&self) -> Option<&FocusId> {
        self.saved_focus.as_ref()
    }

    /// Takes the saved focus ID, leaving `None` in its place.
    ///
    /// This is useful when restoring focus after the trap is popped.
    pub fn take_saved_focus(&mut self) -> Option<FocusId> {
        self.saved_focus.take()
    }

    /// Returns `true` if the given ID is in this trap.
    pub fn contains(&self, id: &FocusId) -> bool {
        self.ring.contains(id)
    }

    /// Returns a reference to the underlying focus ring.
    pub fn ring(&self) -> &FocusRing {
        &self.ring
    }

    /// Returns a mutable reference to the underlying focus ring.
    pub fn ring_mut(&mut self) -> &mut FocusRing {
        &mut self.ring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let trap = FocusTrap::new();
        assert!(trap.is_empty());
        assert!(trap.saved_focus().is_none());
    }

    #[test]
    fn test_with_saved_focus() {
        let trap = FocusTrap::with_saved_focus(FocusId::new("original"));
        assert!(trap.is_empty());
        assert_eq!(trap.saved_focus(), Some(&FocusId::new("original")));
    }

    #[test]
    fn test_register_and_navigate() {
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("a"), 0);
        trap.register(FocusId::new("b"), 0);

        assert_eq!(trap.next(), Some(FocusId::new("a")));
        assert_eq!(trap.next(), Some(FocusId::new("b")));
        assert_eq!(trap.next(), Some(FocusId::new("a"))); // Cycles
    }

    #[test]
    fn test_prev_navigation() {
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("a"), 0);
        trap.register(FocusId::new("b"), 0);

        assert_eq!(trap.prev(), Some(FocusId::new("b"))); // Starts at end
        assert_eq!(trap.prev(), Some(FocusId::new("a")));
    }

    #[test]
    fn test_focus_direct() {
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("a"), 0);
        trap.register(FocusId::new("b"), 0);

        assert!(trap.focus(&FocusId::new("b")));
        assert_eq!(trap.current(), Some(&FocusId::new("b")));
    }

    #[test]
    fn test_unregister() {
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("a"), 0);
        trap.register(FocusId::new("b"), 0);

        assert!(trap.unregister(&FocusId::new("a")));
        assert_eq!(trap.len(), 1);
        assert!(!trap.contains(&FocusId::new("a")));
    }

    #[test]
    fn test_take_saved_focus() {
        let mut trap = FocusTrap::with_saved_focus(FocusId::new("saved"));

        let taken = trap.take_saved_focus();
        assert_eq!(taken, Some(FocusId::new("saved")));
        assert!(trap.saved_focus().is_none());
    }

    #[test]
    fn test_contains() {
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("inside"), 0);

        assert!(trap.contains(&FocusId::new("inside")));
        assert!(!trap.contains(&FocusId::new("outside")));
    }

    #[test]
    fn test_ring_access() {
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("a"), 0);

        // Read access
        assert_eq!(trap.ring().len(), 1);

        // Write access
        trap.ring_mut().register(FocusId::new("b"), 0);
        assert_eq!(trap.len(), 2);
    }

    #[test]
    fn test_default() {
        let trap = FocusTrap::default();
        assert!(trap.is_empty());
    }
}
