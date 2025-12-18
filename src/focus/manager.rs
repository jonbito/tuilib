//! Focus manager for coordinating focus navigation.
//!
//! The [`FocusManager`] provides the main interface for focus management in a
//! TUI application. It handles Tab/Shift+Tab navigation, focus traps for modals,
//! and focus restoration.

use super::{FocusId, FocusRing, FocusTrap};

/// Focus navigation direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    /// Navigate to the next focusable component (Tab).
    Next,
    /// Navigate to the previous focusable component (Shift+Tab).
    Previous,
}

/// Result of a focus navigation operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusResult {
    /// Focus moved to a new component.
    Moved {
        /// The ID of the previously focused component, if any.
        from: Option<FocusId>,
        /// The ID of the newly focused component.
        to: FocusId,
    },
    /// Focus stayed on the same component (only one item in ring).
    Unchanged(FocusId),
    /// No focusable components available.
    NoFocusables,
}

/// Main focus management interface.
///
/// The `FocusManager` coordinates focus navigation across all focusable components
/// in the application. It supports:
///
/// - Tab/Shift+Tab navigation between components
/// - Programmatic focus control
/// - Focus traps for modal dialogs
/// - Focus restoration when traps are popped
///
/// # Basic Usage
///
/// ```rust
/// use tuilib::focus::{FocusId, FocusManager, FocusDirection};
///
/// let mut manager = FocusManager::new();
///
/// // Register focusable components
/// manager.register(FocusId::new("input-1"), 0);
/// manager.register(FocusId::new("input-2"), 0);
/// manager.register(FocusId::new("submit"), 0);
///
/// // Navigate with Tab
/// manager.navigate(FocusDirection::Next);
/// assert_eq!(manager.current(), Some(&FocusId::new("input-1")));
///
/// manager.navigate(FocusDirection::Next);
/// assert_eq!(manager.current(), Some(&FocusId::new("input-2")));
///
/// // Navigate with Shift+Tab
/// manager.navigate(FocusDirection::Previous);
/// assert_eq!(manager.current(), Some(&FocusId::new("input-1")));
/// ```
///
/// # Focus Traps (Modal Dialogs)
///
/// ```rust
/// use tuilib::focus::{FocusId, FocusManager, FocusTrap, FocusDirection};
///
/// let mut manager = FocusManager::new();
/// manager.register(FocusId::new("main-input"), 0);
/// manager.navigate(FocusDirection::Next);
///
/// // Open a modal dialog with focus trap
/// let mut trap = FocusTrap::new();
/// trap.register(FocusId::new("modal-ok"), 0);
/// trap.register(FocusId::new("modal-cancel"), 0);
///
/// manager.push_trap(trap);
///
/// // Trap auto-focuses first item, navigation moves to next
/// assert_eq!(manager.current(), Some(&FocusId::new("modal-ok")));
/// manager.navigate(FocusDirection::Next);
/// assert_eq!(manager.current(), Some(&FocusId::new("modal-cancel")));
///
/// // Close modal and restore focus
/// manager.pop_trap();
/// assert_eq!(manager.current(), Some(&FocusId::new("main-input")));
/// ```
#[derive(Debug, Clone)]
pub struct FocusManager {
    ring: FocusRing,
    traps: Vec<FocusTrap>,
    restoration_stack: Vec<FocusId>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Creates a new focus manager.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::FocusManager;
    ///
    /// let manager = FocusManager::new();
    /// assert!(manager.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            ring: FocusRing::new(),
            traps: Vec::new(),
            restoration_stack: Vec::new(),
        }
    }

    /// Returns `true` if no focusable components are registered.
    ///
    /// This only checks the main focus ring, not any active traps.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Returns the total number of registered focusable components.
    ///
    /// This only counts the main focus ring, not trap contents.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Returns `true` if a focus trap is currently active.
    pub fn has_trap(&self) -> bool {
        !self.traps.is_empty()
    }

    /// Returns the number of active focus traps.
    pub fn trap_count(&self) -> usize {
        self.traps.len()
    }

    /// Registers a focusable component.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the component
    /// * `order` - The focus order priority (lower = earlier in tab order)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusManager};
    ///
    /// let mut manager = FocusManager::new();
    /// manager.register(FocusId::new("button"), 0);
    /// assert_eq!(manager.len(), 1);
    /// ```
    pub fn register(&mut self, id: FocusId, order: i32) {
        self.ring.register(id, order);
    }

    /// Unregisters a component from the focus manager.
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

    /// Navigates focus in the given direction.
    ///
    /// If a focus trap is active, navigation is restricted to the trap.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to navigate (Next or Previous)
    ///
    /// # Returns
    ///
    /// A `FocusResult` indicating what happened.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusManager, FocusDirection, FocusResult};
    ///
    /// let mut manager = FocusManager::new();
    /// manager.register(FocusId::new("a"), 0);
    /// manager.register(FocusId::new("b"), 0);
    ///
    /// let result = manager.navigate(FocusDirection::Next);
    /// assert!(matches!(result, FocusResult::Moved { to, .. } if to == FocusId::new("a")));
    /// ```
    pub fn navigate(&mut self, direction: FocusDirection) -> FocusResult {
        // Get current focus before navigation (must be done separately to avoid borrow issues)
        let from = if let Some(trap) = self.traps.last() {
            trap.current().cloned()
        } else {
            self.ring.current().cloned()
        };

        // Navigate in the appropriate ring
        let to = if let Some(trap) = self.traps.last_mut() {
            match direction {
                FocusDirection::Next => trap.next(),
                FocusDirection::Previous => trap.prev(),
            }
        } else {
            match direction {
                FocusDirection::Next => self.ring.next(),
                FocusDirection::Previous => self.ring.prev(),
            }
        };

        match to {
            Some(to_id) => {
                if from.as_ref() == Some(&to_id) {
                    FocusResult::Unchanged(to_id)
                } else {
                    FocusResult::Moved { from, to: to_id }
                }
            }
            None => FocusResult::NoFocusables,
        }
    }

    /// Moves focus to the next component (Tab behavior).
    ///
    /// This is a convenience method for `navigate(FocusDirection::Next)`.
    ///
    /// # Returns
    ///
    /// The ID of the newly focused component, or `None` if no focusables.
    pub fn focus_next(&mut self) -> Option<FocusId> {
        match self.navigate(FocusDirection::Next) {
            FocusResult::Moved { to, .. } => Some(to),
            FocusResult::Unchanged(id) => Some(id),
            FocusResult::NoFocusables => None,
        }
    }

    /// Moves focus to the previous component (Shift+Tab behavior).
    ///
    /// This is a convenience method for `navigate(FocusDirection::Previous)`.
    ///
    /// # Returns
    ///
    /// The ID of the newly focused component, or `None` if no focusables.
    pub fn focus_prev(&mut self) -> Option<FocusId> {
        match self.navigate(FocusDirection::Previous) {
            FocusResult::Moved { to, .. } => Some(to),
            FocusResult::Unchanged(id) => Some(id),
            FocusResult::NoFocusables => None,
        }
    }

    /// Focuses a specific component by ID.
    ///
    /// If a focus trap is active, the ID must be within the trap.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the component to focus
    ///
    /// # Returns
    ///
    /// `true` if the component was found and focused.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusManager};
    ///
    /// let mut manager = FocusManager::new();
    /// manager.register(FocusId::new("a"), 0);
    /// manager.register(FocusId::new("b"), 0);
    ///
    /// assert!(manager.focus(&FocusId::new("b")));
    /// assert_eq!(manager.current(), Some(&FocusId::new("b")));
    ///
    /// assert!(!manager.focus(&FocusId::new("nonexistent")));
    /// ```
    pub fn focus(&mut self, id: &FocusId) -> bool {
        if let Some(trap) = self.traps.last_mut() {
            trap.focus(id)
        } else {
            self.ring.focus(id)
        }
    }

    /// Returns the currently focused component's ID.
    ///
    /// If a focus trap is active, returns the focused item within the trap.
    pub fn current(&self) -> Option<&FocusId> {
        if let Some(trap) = self.traps.last() {
            trap.current()
        } else {
            self.ring.current()
        }
    }

    /// Clears the current focus.
    ///
    /// If a focus trap is active, clears focus within the trap.
    pub fn clear_focus(&mut self) {
        if let Some(trap) = self.traps.last_mut() {
            trap.ring_mut().clear_focus();
        } else {
            self.ring.clear_focus();
        }
    }

    /// Pushes a new focus trap onto the stack.
    ///
    /// The current focus is automatically saved and will be restored when
    /// the trap is popped.
    ///
    /// # Arguments
    ///
    /// * `trap` - The focus trap to activate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusManager, FocusTrap, FocusDirection};
    ///
    /// let mut manager = FocusManager::new();
    /// manager.register(FocusId::new("main"), 0);
    /// manager.navigate(FocusDirection::Next);
    ///
    /// let mut trap = FocusTrap::new();
    /// trap.register(FocusId::new("modal"), 0);
    /// manager.push_trap(trap);
    ///
    /// assert!(manager.has_trap());
    /// ```
    pub fn push_trap(&mut self, mut trap: FocusTrap) {
        // Save current focus for restoration
        let current = self.current().cloned();
        if let Some(id) = current {
            self.restoration_stack.push(id);
        }

        // Focus first item in trap if nothing is focused
        if trap.current().is_none() && !trap.is_empty() {
            trap.next();
        }

        self.traps.push(trap);
    }

    /// Pops the topmost focus trap and restores previous focus.
    ///
    /// # Returns
    ///
    /// The popped trap, or `None` if no trap was active.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusManager, FocusTrap, FocusDirection};
    ///
    /// let mut manager = FocusManager::new();
    /// manager.register(FocusId::new("main"), 0);
    /// manager.navigate(FocusDirection::Next);
    ///
    /// let mut trap = FocusTrap::new();
    /// trap.register(FocusId::new("modal"), 0);
    /// manager.push_trap(trap);
    ///
    /// let popped = manager.pop_trap();
    /// assert!(popped.is_some());
    /// assert!(!manager.has_trap());
    /// assert_eq!(manager.current(), Some(&FocusId::new("main")));
    /// ```
    pub fn pop_trap(&mut self) -> Option<FocusTrap> {
        let trap = self.traps.pop()?;

        // Restore previous focus
        if let Some(saved_id) = self.restoration_stack.pop() {
            if self.traps.is_empty() {
                // Restoring to main ring
                self.ring.focus(&saved_id);
            } else {
                // Restoring to outer trap
                if let Some(outer) = self.traps.last_mut() {
                    outer.focus(&saved_id);
                }
            }
        }

        Some(trap)
    }

    /// Saves the current focus to the restoration stack.
    ///
    /// This is useful for manual focus management when not using traps.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::focus::{FocusId, FocusManager, FocusDirection};
    ///
    /// let mut manager = FocusManager::new();
    /// manager.register(FocusId::new("a"), 0);
    /// manager.register(FocusId::new("b"), 0);
    /// manager.navigate(FocusDirection::Next); // Focus "a"
    ///
    /// manager.save_focus();
    /// manager.focus(&FocusId::new("b"));
    ///
    /// manager.restore_focus();
    /// assert_eq!(manager.current(), Some(&FocusId::new("a")));
    /// ```
    pub fn save_focus(&mut self) {
        if let Some(id) = self.current().cloned() {
            self.restoration_stack.push(id);
        }
    }

    /// Restores focus from the restoration stack.
    ///
    /// # Returns
    ///
    /// The restored focus ID, or `None` if the stack was empty.
    pub fn restore_focus(&mut self) -> Option<FocusId> {
        let id = self.restoration_stack.pop()?;
        self.focus(&id);
        Some(id)
    }

    /// Returns `true` if the given ID is focusable in the current context.
    ///
    /// If a trap is active, checks within the trap.
    pub fn contains(&self, id: &FocusId) -> bool {
        if let Some(trap) = self.traps.last() {
            trap.contains(id)
        } else {
            self.ring.contains(id)
        }
    }

    /// Clears all registrations and traps.
    pub fn clear(&mut self) {
        self.ring.clear();
        self.traps.clear();
        self.restoration_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let manager = FocusManager::new();
        assert!(manager.is_empty());
        assert!(!manager.has_trap());
        assert!(manager.current().is_none());
    }

    #[test]
    fn test_register_and_navigate() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.register(FocusId::new("b"), 0);
        manager.register(FocusId::new("c"), 0);

        assert_eq!(manager.len(), 3);

        // Navigate forward
        let result = manager.navigate(FocusDirection::Next);
        assert!(matches!(result, FocusResult::Moved { from: None, to } if to == FocusId::new("a")));

        let result = manager.navigate(FocusDirection::Next);
        assert!(
            matches!(result, FocusResult::Moved { from: Some(f), to } if f == FocusId::new("a") && to == FocusId::new("b"))
        );
    }

    #[test]
    fn test_navigate_previous() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.register(FocusId::new("b"), 0);

        // Start from end
        manager.navigate(FocusDirection::Previous);
        assert_eq!(manager.current(), Some(&FocusId::new("b")));

        manager.navigate(FocusDirection::Previous);
        assert_eq!(manager.current(), Some(&FocusId::new("a")));
    }

    #[test]
    fn test_focus_next_prev_convenience() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.register(FocusId::new("b"), 0);

        assert_eq!(manager.focus_next(), Some(FocusId::new("a")));
        assert_eq!(manager.focus_next(), Some(FocusId::new("b")));
        assert_eq!(manager.focus_prev(), Some(FocusId::new("a")));
    }

    #[test]
    fn test_focus_direct() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.register(FocusId::new("b"), 0);

        assert!(manager.focus(&FocusId::new("b")));
        assert_eq!(manager.current(), Some(&FocusId::new("b")));

        assert!(!manager.focus(&FocusId::new("nonexistent")));
    }

    #[test]
    fn test_unregister() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.register(FocusId::new("b"), 0);

        assert!(manager.unregister(&FocusId::new("a")));
        assert_eq!(manager.len(), 1);
        assert!(!manager.contains(&FocusId::new("a")));
    }

    #[test]
    fn test_focus_trap_basic() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("main-1"), 0);
        manager.register(FocusId::new("main-2"), 0);
        manager.focus_next(); // Focus main-1

        // Push trap
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("modal-1"), 0);
        trap.register(FocusId::new("modal-2"), 0);
        manager.push_trap(trap);

        assert!(manager.has_trap());
        assert_eq!(manager.trap_count(), 1);

        // Navigation should be within trap
        assert_eq!(manager.current(), Some(&FocusId::new("modal-1")));
        manager.focus_next();
        assert_eq!(manager.current(), Some(&FocusId::new("modal-2")));
        manager.focus_next();
        assert_eq!(manager.current(), Some(&FocusId::new("modal-1"))); // Wraps in trap
    }

    #[test]
    fn test_focus_trap_restoration() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("main"), 0);
        manager.focus_next();
        assert_eq!(manager.current(), Some(&FocusId::new("main")));

        // Push trap
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("modal"), 0);
        manager.push_trap(trap);

        // Pop trap and restore focus
        manager.pop_trap();

        assert!(!manager.has_trap());
        assert_eq!(manager.current(), Some(&FocusId::new("main")));
    }

    #[test]
    fn test_nested_traps() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("main"), 0);
        manager.focus_next();

        // First trap
        let mut trap1 = FocusTrap::new();
        trap1.register(FocusId::new("modal1"), 0);
        manager.push_trap(trap1);
        manager.focus_next();

        // Nested trap
        let mut trap2 = FocusTrap::new();
        trap2.register(FocusId::new("modal2"), 0);
        manager.push_trap(trap2);

        assert_eq!(manager.trap_count(), 2);
        assert_eq!(manager.current(), Some(&FocusId::new("modal2")));

        // Pop inner trap
        manager.pop_trap();
        assert_eq!(manager.trap_count(), 1);
        assert_eq!(manager.current(), Some(&FocusId::new("modal1")));

        // Pop outer trap
        manager.pop_trap();
        assert!(!manager.has_trap());
        assert_eq!(manager.current(), Some(&FocusId::new("main")));
    }

    #[test]
    fn test_save_restore_focus() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.register(FocusId::new("b"), 0);
        manager.focus_next(); // Focus "a"

        manager.save_focus();
        manager.focus(&FocusId::new("b"));
        assert_eq!(manager.current(), Some(&FocusId::new("b")));

        let restored = manager.restore_focus();
        assert_eq!(restored, Some(FocusId::new("a")));
        assert_eq!(manager.current(), Some(&FocusId::new("a")));
    }

    #[test]
    fn test_clear_focus() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.focus_next();

        assert!(manager.current().is_some());
        manager.clear_focus();
        assert!(manager.current().is_none());
    }

    #[test]
    fn test_clear() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("a"), 0);
        manager.focus_next();

        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("modal"), 0);
        manager.push_trap(trap);

        manager.clear();

        assert!(manager.is_empty());
        assert!(!manager.has_trap());
    }

    #[test]
    fn test_contains() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("main"), 0);

        assert!(manager.contains(&FocusId::new("main")));
        assert!(!manager.contains(&FocusId::new("other")));

        // With trap, contains checks trap
        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("modal"), 0);
        manager.push_trap(trap);

        assert!(manager.contains(&FocusId::new("modal")));
        assert!(!manager.contains(&FocusId::new("main"))); // Not in trap
    }

    #[test]
    fn test_navigate_empty() {
        let mut manager = FocusManager::new();
        let result = manager.navigate(FocusDirection::Next);
        assert_eq!(result, FocusResult::NoFocusables);
    }

    #[test]
    fn test_navigate_single_item() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("only"), 0);

        manager.focus_next();
        let result = manager.navigate(FocusDirection::Next);
        assert_eq!(result, FocusResult::Unchanged(FocusId::new("only")));
    }

    #[test]
    fn test_default() {
        let manager = FocusManager::default();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_focus_in_trap_restricts_to_trap() {
        let mut manager = FocusManager::new();
        manager.register(FocusId::new("main"), 0);

        let mut trap = FocusTrap::new();
        trap.register(FocusId::new("modal"), 0);
        manager.push_trap(trap);

        // Can't focus main ring items when trap is active
        assert!(!manager.focus(&FocusId::new("main")));
        assert!(manager.focus(&FocusId::new("modal")));
    }
}
