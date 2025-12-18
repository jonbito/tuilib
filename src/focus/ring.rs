//! Focus ring for tracking focusable components.
//!
//! The [`FocusRing`] maintains an ordered list of focusable component IDs
//! and handles navigation between them. It supports forward (Tab) and
//! backward (Shift+Tab) navigation with wrapping.

use super::FocusId;

/// An entry in the focus ring with its order priority.
#[derive(Debug, Clone)]
struct FocusEntry {
    id: FocusId,
    order: i32,
}

/// Tracks the order and state of focusable components.
///
/// A `FocusRing` maintains a list of focusable components in a defined order
/// and tracks which component currently has focus. It supports circular
/// navigation (wrapping from last to first and vice versa).
///
/// # Navigation
///
/// - `next()` moves focus forward through the ring (Tab behavior)
/// - `prev()` moves focus backward through the ring (Shift+Tab behavior)
/// - `focus()` directly focuses a specific component by ID
///
/// # Focus Order
///
/// Components are ordered first by their `order` value (lower values first),
/// then by their registration order for components with equal order values.
///
/// # Examples
///
/// ```rust
/// use tuilib::focus::{FocusId, FocusRing};
///
/// let mut ring = FocusRing::new();
///
/// // Register components with default order
/// ring.register(FocusId::new("input-1"), 0);
/// ring.register(FocusId::new("input-2"), 0);
/// ring.register(FocusId::new("submit"), 0);
///
/// // Navigate forward
/// assert_eq!(ring.next(), Some(FocusId::new("input-1")));
/// assert_eq!(ring.next(), Some(FocusId::new("input-2")));
/// assert_eq!(ring.next(), Some(FocusId::new("submit")));
/// assert_eq!(ring.next(), Some(FocusId::new("input-1"))); // Wraps around
/// ```
#[derive(Debug, Clone)]
pub struct FocusRing {
    entries: Vec<FocusEntry>,
    current_index: Option<usize>,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusRing {
    /// Creates a new empty focus ring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::FocusRing;
    ///
    /// let ring = FocusRing::new();
    /// assert!(ring.is_empty());
    /// assert!(ring.current().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
        }
    }

    /// Returns `true` if the focus ring has no entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// assert!(ring.is_empty());
    ///
    /// ring.register(FocusId::new("button"), 0);
    /// assert!(!ring.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of entries in the focus ring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// assert_eq!(ring.len(), 0);
    ///
    /// ring.register(FocusId::new("a"), 0);
    /// ring.register(FocusId::new("b"), 0);
    /// assert_eq!(ring.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Registers a focusable component with the given order priority.
    ///
    /// Components with lower order values are focused first. Components
    /// with equal order values are focused in registration order.
    ///
    /// If a component with the same ID is already registered, this
    /// updates its order value.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the component
    /// * `order` - The focus order priority (lower = earlier in tab order)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    ///
    /// // Register with custom order
    /// ring.register(FocusId::new("last"), 10);
    /// ring.register(FocusId::new("first"), -10);
    /// ring.register(FocusId::new("middle"), 0);
    ///
    /// // First focuses "first" due to lowest order
    /// assert_eq!(ring.next(), Some(FocusId::new("first")));
    /// ```
    pub fn register(&mut self, id: FocusId, order: i32) {
        // Check if already registered
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.order = order;
            self.sort_entries();
            return;
        }

        self.entries.push(FocusEntry { id, order });
        self.sort_entries();
    }

    /// Unregisters a component from the focus ring.
    ///
    /// If the unregistered component had focus, focus moves to the next
    /// component in the ring. If the ring becomes empty, focus is cleared.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the component to unregister
    ///
    /// # Returns
    ///
    /// `true` if the component was found and removed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("button"), 0);
    ///
    /// assert!(ring.unregister(&FocusId::new("button")));
    /// assert!(!ring.unregister(&FocusId::new("button"))); // Already removed
    /// ```
    pub fn unregister(&mut self, id: &FocusId) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| &e.id == id) {
            self.entries.remove(pos);

            // Adjust current index if needed
            if let Some(current) = self.current_index {
                if self.entries.is_empty() {
                    self.current_index = None;
                } else if current == pos {
                    // Focused item was removed, adjust index
                    self.current_index = Some(current.min(self.entries.len() - 1));
                } else if current > pos {
                    self.current_index = Some(current - 1);
                }
            }

            return true;
        }
        false
    }

    /// Moves focus to the next component in the ring.
    ///
    /// If no component has focus, focuses the first component.
    /// Wraps around from the last component to the first.
    ///
    /// # Returns
    ///
    /// The ID of the newly focused component, or `None` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("a"), 0);
    /// ring.register(FocusId::new("b"), 0);
    ///
    /// assert_eq!(ring.next(), Some(FocusId::new("a")));
    /// assert_eq!(ring.next(), Some(FocusId::new("b")));
    /// assert_eq!(ring.next(), Some(FocusId::new("a"))); // Wraps
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<FocusId> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(current) => (current + 1) % self.entries.len(),
            None => 0,
        };

        self.current_index = Some(new_index);
        Some(self.entries[new_index].id.clone())
    }

    /// Moves focus to the previous component in the ring.
    ///
    /// If no component has focus, focuses the last component.
    /// Wraps around from the first component to the last.
    ///
    /// # Returns
    ///
    /// The ID of the newly focused component, or `None` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("a"), 0);
    /// ring.register(FocusId::new("b"), 0);
    ///
    /// assert_eq!(ring.prev(), Some(FocusId::new("b"))); // Starts at end
    /// assert_eq!(ring.prev(), Some(FocusId::new("a")));
    /// assert_eq!(ring.prev(), Some(FocusId::new("b"))); // Wraps
    /// ```
    pub fn prev(&mut self) -> Option<FocusId> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(current) => {
                if current == 0 {
                    self.entries.len() - 1
                } else {
                    current - 1
                }
            }
            None => self.entries.len() - 1,
        };

        self.current_index = Some(new_index);
        Some(self.entries[new_index].id.clone())
    }

    /// Focuses a specific component by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the component to focus
    ///
    /// # Returns
    ///
    /// `true` if the component was found and focused, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("a"), 0);
    /// ring.register(FocusId::new("b"), 0);
    ///
    /// assert!(ring.focus(&FocusId::new("b")));
    /// assert_eq!(ring.current(), Some(&FocusId::new("b")));
    ///
    /// assert!(!ring.focus(&FocusId::new("nonexistent")));
    /// ```
    pub fn focus(&mut self, id: &FocusId) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| &e.id == id) {
            self.current_index = Some(pos);
            return true;
        }
        false
    }

    /// Returns a reference to the currently focused component's ID.
    ///
    /// # Returns
    ///
    /// The ID of the currently focused component, or `None` if no component
    /// has focus.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// assert!(ring.current().is_none());
    ///
    /// ring.register(FocusId::new("button"), 0);
    /// ring.next();
    /// assert_eq!(ring.current(), Some(&FocusId::new("button")));
    /// ```
    pub fn current(&self) -> Option<&FocusId> {
        self.current_index.map(|i| &self.entries[i].id)
    }

    /// Clears the current focus without removing any entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("button"), 0);
    /// ring.next();
    /// assert!(ring.current().is_some());
    ///
    /// ring.clear_focus();
    /// assert!(ring.current().is_none());
    /// ```
    pub fn clear_focus(&mut self) {
        self.current_index = None;
    }

    /// Removes all entries from the focus ring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("a"), 0);
    /// ring.register(FocusId::new("b"), 0);
    ///
    /// ring.clear();
    /// assert!(ring.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
    }

    /// Returns `true` if the given ID is registered in the focus ring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("button"), 0);
    ///
    /// assert!(ring.contains(&FocusId::new("button")));
    /// assert!(!ring.contains(&FocusId::new("other")));
    /// ```
    pub fn contains(&self, id: &FocusId) -> bool {
        self.entries.iter().any(|e| &e.id == id)
    }

    /// Returns an iterator over the IDs in focus order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusRing};
    ///
    /// let mut ring = FocusRing::new();
    /// ring.register(FocusId::new("a"), 0);
    /// ring.register(FocusId::new("b"), 0);
    ///
    /// let ids: Vec<_> = ring.iter().collect();
    /// assert_eq!(ids.len(), 2);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &FocusId> {
        self.entries.iter().map(|e| &e.id)
    }

    /// Sorts entries by order value, maintaining registration order for equal values.
    fn sort_entries(&mut self) {
        // Get the current focused ID before sorting
        let current_id = self.current_index.map(|i| self.entries[i].id.clone());

        // Stable sort preserves registration order for equal order values
        self.entries.sort_by_key(|e| e.order);

        // Restore the current index after sorting
        if let Some(id) = current_id {
            self.current_index = self.entries.iter().position(|e| e.id == id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let ring = FocusRing::new();
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
        assert!(ring.current().is_none());
    }

    #[test]
    fn test_register() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);

        assert_eq!(ring.len(), 2);
        assert!(ring.contains(&FocusId::new("a")));
        assert!(ring.contains(&FocusId::new("b")));
    }

    #[test]
    fn test_register_duplicate_updates_order() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 10);
        ring.register(FocusId::new("b"), 0);
        ring.next(); // Focus first (b)

        assert_eq!(ring.current(), Some(&FocusId::new("b")));

        // Re-register "a" with lower order
        ring.register(FocusId::new("a"), -10);

        // Should still only have 2 entries
        assert_eq!(ring.len(), 2);

        // "a" should now be first due to lower order
        ring.clear_focus();
        ring.next();
        assert_eq!(ring.current(), Some(&FocusId::new("a")));
    }

    #[test]
    fn test_unregister() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);

        assert!(ring.unregister(&FocusId::new("a")));
        assert_eq!(ring.len(), 1);
        assert!(!ring.contains(&FocusId::new("a")));
        assert!(ring.contains(&FocusId::new("b")));
    }

    #[test]
    fn test_unregister_nonexistent() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);

        assert!(!ring.unregister(&FocusId::new("nonexistent")));
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn test_unregister_focused_item() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);
        ring.register(FocusId::new("c"), 0);

        ring.focus(&FocusId::new("b"));
        assert_eq!(ring.current(), Some(&FocusId::new("b")));

        ring.unregister(&FocusId::new("b"));

        // Focus should move to the next valid item
        assert!(ring.current().is_some());
        assert_eq!(ring.len(), 2);
    }

    #[test]
    fn test_unregister_all() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.next();

        ring.unregister(&FocusId::new("a"));

        assert!(ring.is_empty());
        assert!(ring.current().is_none());
    }

    #[test]
    fn test_next_empty() {
        let mut ring = FocusRing::new();
        assert!(ring.next().is_none());
    }

    #[test]
    fn test_next_single() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("only"), 0);

        assert_eq!(ring.next(), Some(FocusId::new("only")));
        assert_eq!(ring.next(), Some(FocusId::new("only")));
    }

    #[test]
    fn test_next_wrapping() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);
        ring.register(FocusId::new("c"), 0);

        assert_eq!(ring.next(), Some(FocusId::new("a")));
        assert_eq!(ring.next(), Some(FocusId::new("b")));
        assert_eq!(ring.next(), Some(FocusId::new("c")));
        assert_eq!(ring.next(), Some(FocusId::new("a"))); // Wraps
    }

    #[test]
    fn test_prev_empty() {
        let mut ring = FocusRing::new();
        assert!(ring.prev().is_none());
    }

    #[test]
    fn test_prev_single() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("only"), 0);

        assert_eq!(ring.prev(), Some(FocusId::new("only")));
        assert_eq!(ring.prev(), Some(FocusId::new("only")));
    }

    #[test]
    fn test_prev_wrapping() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);
        ring.register(FocusId::new("c"), 0);

        assert_eq!(ring.prev(), Some(FocusId::new("c"))); // Starts at end
        assert_eq!(ring.prev(), Some(FocusId::new("b")));
        assert_eq!(ring.prev(), Some(FocusId::new("a")));
        assert_eq!(ring.prev(), Some(FocusId::new("c"))); // Wraps
    }

    #[test]
    fn test_focus_direct() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);
        ring.register(FocusId::new("c"), 0);

        assert!(ring.focus(&FocusId::new("b")));
        assert_eq!(ring.current(), Some(&FocusId::new("b")));

        // Next should go to c
        assert_eq!(ring.next(), Some(FocusId::new("c")));
    }

    #[test]
    fn test_focus_nonexistent() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);

        assert!(!ring.focus(&FocusId::new("nonexistent")));
        assert!(ring.current().is_none());
    }

    #[test]
    fn test_clear_focus() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.next();

        assert!(ring.current().is_some());
        ring.clear_focus();
        assert!(ring.current().is_none());

        // Ring still has entries
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);
        ring.next();

        ring.clear();

        assert!(ring.is_empty());
        assert!(ring.current().is_none());
    }

    #[test]
    fn test_order_priority() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("low-priority"), 100);
        ring.register(FocusId::new("high-priority"), -100);
        ring.register(FocusId::new("medium-priority"), 0);

        // Should focus in order: high, medium, low
        assert_eq!(ring.next(), Some(FocusId::new("high-priority")));
        assert_eq!(ring.next(), Some(FocusId::new("medium-priority")));
        assert_eq!(ring.next(), Some(FocusId::new("low-priority")));
    }

    #[test]
    fn test_order_preserves_registration_for_equal() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("first"), 0);
        ring.register(FocusId::new("second"), 0);
        ring.register(FocusId::new("third"), 0);

        // Should focus in registration order since all have same priority
        assert_eq!(ring.next(), Some(FocusId::new("first")));
        assert_eq!(ring.next(), Some(FocusId::new("second")));
        assert_eq!(ring.next(), Some(FocusId::new("third")));
    }

    #[test]
    fn test_iter() {
        let mut ring = FocusRing::new();
        ring.register(FocusId::new("a"), 0);
        ring.register(FocusId::new("b"), 0);

        let ids: Vec<_> = ring.iter().cloned().collect();
        assert_eq!(ids, vec![FocusId::new("a"), FocusId::new("b")]);
    }

    #[test]
    fn test_default() {
        let ring = FocusRing::default();
        assert!(ring.is_empty());
    }
}
