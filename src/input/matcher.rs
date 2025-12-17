//! Input matching logic for mapping events to actions.
//!
//! This module provides the [`InputMatcher`] that maintains state for
//! multi-key sequences and matches input events against registered bindings.
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::{Action, InputMatcher, KeyBinding, KeySequence};
//! use terminput::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind, KeyEventState};
//! use std::time::Duration;
//!
//! let mut matcher = InputMatcher::new(Duration::from_millis(1000));
//!
//! // Register bindings
//! matcher.register(
//!     KeySequence::single(KeyBinding::new(KeyCode::Char('q'))),
//!     Action::new("quit")
//! );
//! matcher.register(
//!     KeySequence::new(vec![
//!         KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
//!         KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
//!     ]),
//!     Action::new("save")
//! );
//!
//! // Process key events
//! let event = KeyEvent {
//!     code: KeyCode::Char('q'),
//!     modifiers: KeyModifiers::NONE,
//!     kind: KeyEventKind::Press,
//!     state: KeyEventState::NONE,
//! };
//!
//! match matcher.process(&event) {
//!     tuilib::input::MatchResult::Matched(action) => {
//!         assert_eq!(action.name(), "quit");
//!     }
//!     _ => panic!("Expected match"),
//! }
//! ```

use std::time::{Duration, Instant};

use terminput::KeyEvent;

use super::{Action, KeyBinding, KeySequence};

/// Result of processing an input event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchResult {
    /// No binding matched the input.
    NoMatch,
    /// A partial sequence match - more keys are needed.
    Pending,
    /// A complete binding matched, returning the associated action.
    Matched(Action),
}

impl MatchResult {
    /// Returns true if this result is a match.
    pub fn is_matched(&self) -> bool {
        matches!(self, MatchResult::Matched(_))
    }

    /// Returns true if this result is pending (partial sequence).
    pub fn is_pending(&self) -> bool {
        matches!(self, MatchResult::Pending)
    }

    /// Returns true if no binding matched.
    pub fn is_no_match(&self) -> bool {
        matches!(self, MatchResult::NoMatch)
    }

    /// Returns the matched action if this is a match.
    pub fn action(&self) -> Option<&Action> {
        match self {
            MatchResult::Matched(action) => Some(action),
            _ => None,
        }
    }

    /// Consumes the result and returns the action if matched.
    pub fn into_action(self) -> Option<Action> {
        match self {
            MatchResult::Matched(action) => Some(action),
            _ => None,
        }
    }
}

/// A registered binding with its associated action.
#[derive(Debug, Clone)]
struct RegisteredBinding {
    sequence: KeySequence,
    action: Action,
}

/// Matches input events against registered key bindings.
///
/// The matcher maintains state for multi-key sequences and handles
/// timeouts when a sequence is partially matched.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::{Action, InputMatcher, KeyBinding, KeySequence};
/// use terminput::{KeyCode, KeyModifiers};
/// use std::time::Duration;
///
/// let mut matcher = InputMatcher::new(Duration::from_millis(1000));
///
/// // Register a simple quit binding
/// matcher.register(
///     KeySequence::single(KeyBinding::new(KeyCode::Char('q'))),
///     Action::new("quit")
/// );
///
/// // Register multiple bindings for the same action
/// matcher.register_multiple(
///     vec![
///         KeySequence::single(KeyBinding::new(KeyCode::Char('k'))),
///         KeySequence::single(KeyBinding::new(KeyCode::Up)),
///     ],
///     Action::new("move_up")
/// );
/// ```
pub struct InputMatcher {
    bindings: Vec<RegisteredBinding>,
    pending_keys: Vec<KeyBinding>,
    last_key_time: Option<Instant>,
    sequence_timeout: Duration,
}

impl InputMatcher {
    /// Creates a new input matcher with the specified sequence timeout.
    ///
    /// # Arguments
    ///
    /// * `sequence_timeout` - How long to wait for the next key in a sequence
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::InputMatcher;
    /// use std::time::Duration;
    ///
    /// let matcher = InputMatcher::new(Duration::from_millis(500));
    /// ```
    pub fn new(sequence_timeout: Duration) -> Self {
        Self {
            bindings: Vec::new(),
            pending_keys: Vec::new(),
            last_key_time: None,
            sequence_timeout,
        }
    }

    /// Creates a new matcher with a default 1 second timeout.
    pub fn with_default_timeout() -> Self {
        Self::new(Duration::from_secs(1))
    }

    /// Registers a key sequence that triggers an action.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The key sequence to register
    /// * `action` - The action to trigger when the sequence matches
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::{Action, InputMatcher, KeyBinding, KeySequence};
    /// use terminput::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut matcher = InputMatcher::new(Duration::from_millis(1000));
    /// matcher.register(
    ///     KeySequence::new(vec![
    ///         KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
    ///         KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
    ///     ]),
    ///     Action::new("save")
    /// );
    /// ```
    pub fn register(&mut self, sequence: KeySequence, action: Action) {
        self.bindings.push(RegisteredBinding { sequence, action });
    }

    /// Registers multiple key sequences that trigger the same action.
    ///
    /// This is useful for having multiple keys map to the same action,
    /// like both 'k' and Up arrow for "move up".
    ///
    /// # Arguments
    ///
    /// * `sequences` - The key sequences to register
    /// * `action` - The action to trigger when any sequence matches
    pub fn register_multiple(&mut self, sequences: Vec<KeySequence>, action: Action) {
        for sequence in sequences {
            self.bindings.push(RegisteredBinding {
                sequence,
                action: action.clone(),
            });
        }
    }

    /// Registers a single key binding that triggers an action.
    ///
    /// Convenience method for registering single-key bindings.
    ///
    /// # Arguments
    ///
    /// * `binding` - The key binding
    /// * `action` - The action to trigger
    pub fn register_key(&mut self, binding: KeyBinding, action: Action) {
        self.register(KeySequence::single(binding), action);
    }

    /// Processes an input event and returns the match result.
    ///
    /// This method maintains internal state for multi-key sequences.
    /// If a sequence times out, it will be reset.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to process
    ///
    /// # Returns
    ///
    /// - `MatchResult::Matched(action)` if a binding completed
    /// - `MatchResult::Pending` if a partial sequence matched
    /// - `MatchResult::NoMatch` if no binding matched
    pub fn process(&mut self, event: &KeyEvent) -> MatchResult {
        let now = Instant::now();

        // Check for sequence timeout
        if let Some(last_time) = self.last_key_time {
            if now.duration_since(last_time) > self.sequence_timeout {
                self.reset_sequence();
            }
        }

        // Create binding from event
        let key_binding = KeyBinding::with_mods(event.code, event.modifiers);

        // Add to pending keys for sequence matching
        self.pending_keys.push(key_binding.clone());
        self.last_key_time = Some(now);

        // Check for partial sequence matches first - if a longer sequence is possible,
        // we should wait for more keys even if we have a complete shorter match
        if self.has_partial_match() {
            return MatchResult::Pending;
        }

        // Check for complete matches - only if no longer sequence is possible
        if let Some(action) = self.find_complete_match() {
            self.reset_sequence();
            return MatchResult::Matched(action);
        }

        // No match - try just this key alone (reset sequence and retry)
        if self.pending_keys.len() > 1 {
            self.pending_keys.clear();
            self.pending_keys.push(key_binding);

            // Check for partial matches first
            if self.has_partial_match() {
                return MatchResult::Pending;
            }

            // Check single key match
            if let Some(action) = self.find_complete_match() {
                self.reset_sequence();
                return MatchResult::Matched(action);
            }
        }

        self.reset_sequence();
        MatchResult::NoMatch
    }

    /// Resets the sequence matching state.
    ///
    /// Call this when you want to cancel any pending sequence.
    pub fn reset_sequence(&mut self) {
        self.pending_keys.clear();
        self.last_key_time = None;
    }

    /// Returns true if there's a partial sequence in progress.
    pub fn is_sequence_pending(&self) -> bool {
        !self.pending_keys.is_empty()
    }

    /// Returns the pending keys in the current sequence.
    pub fn pending_keys(&self) -> &[KeyBinding] {
        &self.pending_keys
    }

    /// Returns the sequence timeout duration.
    pub fn sequence_timeout(&self) -> Duration {
        self.sequence_timeout
    }

    /// Sets a new sequence timeout.
    pub fn set_sequence_timeout(&mut self, timeout: Duration) {
        self.sequence_timeout = timeout;
    }

    /// Returns the number of registered bindings.
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Clears all registered bindings.
    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
        self.reset_sequence();
    }

    /// Finds a binding that completely matches the pending keys.
    fn find_complete_match(&self) -> Option<Action> {
        for binding in &self.bindings {
            if binding.sequence.len() == self.pending_keys.len() {
                let matches = binding
                    .sequence
                    .keys()
                    .iter()
                    .zip(&self.pending_keys)
                    .all(|(seq_key, pending_key)| seq_key == pending_key);

                if matches {
                    return Some(binding.action.clone());
                }
            }
        }
        None
    }

    /// Checks if any binding could potentially match with more keys.
    fn has_partial_match(&self) -> bool {
        for binding in &self.bindings {
            if binding.sequence.len() > self.pending_keys.len() {
                let prefix_matches = binding
                    .sequence
                    .keys()
                    .iter()
                    .take(self.pending_keys.len())
                    .zip(&self.pending_keys)
                    .all(|(seq_key, pending_key)| seq_key == pending_key);

                if prefix_matches {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for InputMatcher {
    fn default() -> Self {
        Self::with_default_timeout()
    }
}

impl std::fmt::Debug for InputMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputMatcher")
            .field("binding_count", &self.bindings.len())
            .field("pending_keys", &self.pending_keys.len())
            .field("sequence_timeout", &self.sequence_timeout)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminput::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_matcher_creation() {
        let matcher = InputMatcher::new(Duration::from_millis(500));
        assert_eq!(matcher.binding_count(), 0);
        assert!(!matcher.is_sequence_pending());
    }

    #[test]
    fn test_matcher_default() {
        let matcher = InputMatcher::default();
        assert_eq!(matcher.sequence_timeout(), Duration::from_secs(1));
    }

    #[test]
    fn test_single_key_match() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register_key(KeyBinding::new(KeyCode::Char('q')), Action::new("quit"));

        let event = make_key_event(KeyCode::Char('q'), KeyModifiers::NONE);
        let result = matcher.process(&event);

        assert!(result.is_matched());
        assert_eq!(result.action().unwrap().name(), "quit");
    }

    #[test]
    fn test_single_key_with_modifier() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register_key(
            KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
            Action::new("save"),
        );

        let event = make_key_event(KeyCode::Char('s'), KeyModifiers::CTRL);
        let result = matcher.process(&event);

        assert!(result.is_matched());
        assert_eq!(result.action().unwrap().name(), "save");
    }

    #[test]
    fn test_no_match() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register_key(KeyBinding::new(KeyCode::Char('q')), Action::new("quit"));

        let event = make_key_event(KeyCode::Char('w'), KeyModifiers::NONE);
        let result = matcher.process(&event);

        assert!(result.is_no_match());
    }

    #[test]
    fn test_modifier_mismatch() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register_key(
            KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
            Action::new("save"),
        );

        // Without modifier - should not match
        let event = make_key_event(KeyCode::Char('s'), KeyModifiers::NONE);
        let result = matcher.process(&event);

        assert!(result.is_no_match());
    }

    #[test]
    fn test_sequence_match() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register(
            KeySequence::new(vec![
                KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
                KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
            ]),
            Action::new("save"),
        );

        // First key - should be pending
        let event1 = make_key_event(KeyCode::Char('x'), KeyModifiers::CTRL);
        let result1 = matcher.process(&event1);
        assert!(result1.is_pending());

        // Second key - should complete
        let event2 = make_key_event(KeyCode::Char('s'), KeyModifiers::CTRL);
        let result2 = matcher.process(&event2);
        assert!(result2.is_matched());
        assert_eq!(result2.action().unwrap().name(), "save");
    }

    #[test]
    fn test_sequence_wrong_second_key() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register(
            KeySequence::new(vec![
                KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
                KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
            ]),
            Action::new("save"),
        );
        matcher.register_key(KeyBinding::new(KeyCode::Char('q')), Action::new("quit"));

        // First key - pending
        let event1 = make_key_event(KeyCode::Char('x'), KeyModifiers::CTRL);
        let result1 = matcher.process(&event1);
        assert!(result1.is_pending());

        // Wrong second key - should try to match as single key
        let event2 = make_key_event(KeyCode::Char('q'), KeyModifiers::NONE);
        let result2 = matcher.process(&event2);
        assert!(result2.is_matched());
        assert_eq!(result2.action().unwrap().name(), "quit");
    }

    #[test]
    fn test_multiple_bindings_same_action() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register_multiple(
            vec![
                KeySequence::single(KeyBinding::new(KeyCode::Char('k'))),
                KeySequence::single(KeyBinding::new(KeyCode::Up)),
            ],
            Action::new("move_up"),
        );

        let event1 = make_key_event(KeyCode::Char('k'), KeyModifiers::NONE);
        let result1 = matcher.process(&event1);
        assert!(result1.is_matched());
        assert_eq!(result1.action().unwrap().name(), "move_up");

        let event2 = make_key_event(KeyCode::Up, KeyModifiers::NONE);
        let result2 = matcher.process(&event2);
        assert!(result2.is_matched());
        assert_eq!(result2.action().unwrap().name(), "move_up");
    }

    #[test]
    fn test_reset_sequence() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register(
            KeySequence::new(vec![
                KeyBinding::new(KeyCode::Char('x')).with_modifiers(KeyModifiers::CTRL),
                KeyBinding::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::CTRL),
            ]),
            Action::new("save"),
        );

        let event = make_key_event(KeyCode::Char('x'), KeyModifiers::CTRL);
        matcher.process(&event);
        assert!(matcher.is_sequence_pending());

        matcher.reset_sequence();
        assert!(!matcher.is_sequence_pending());
    }

    #[test]
    fn test_clear_bindings() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register_key(KeyBinding::new(KeyCode::Char('q')), Action::new("quit"));
        assert_eq!(matcher.binding_count(), 1);

        matcher.clear_bindings();
        assert_eq!(matcher.binding_count(), 0);
    }

    #[test]
    fn test_match_result_helpers() {
        let matched = MatchResult::Matched(Action::new("test"));
        assert!(matched.is_matched());
        assert!(!matched.is_pending());
        assert!(!matched.is_no_match());
        assert_eq!(matched.action().unwrap().name(), "test");

        let pending = MatchResult::Pending;
        assert!(!pending.is_matched());
        assert!(pending.is_pending());
        assert!(!pending.is_no_match());
        assert!(pending.action().is_none());

        let no_match = MatchResult::NoMatch;
        assert!(!no_match.is_matched());
        assert!(!no_match.is_pending());
        assert!(no_match.is_no_match());
        assert!(no_match.action().is_none());
    }

    #[test]
    fn test_into_action() {
        let matched = MatchResult::Matched(Action::new("test"));
        let action = matched.into_action();
        assert!(action.is_some());
        assert_eq!(action.unwrap().name(), "test");

        let no_match = MatchResult::NoMatch;
        assert!(no_match.into_action().is_none());
    }

    #[test]
    fn test_vim_gg_sequence() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register(
            KeySequence::new(vec![
                KeyBinding::new(KeyCode::Char('g')),
                KeyBinding::new(KeyCode::Char('g')),
            ]),
            Action::new("go_to_top"),
        );

        // First 'g' - pending
        let event1 = make_key_event(KeyCode::Char('g'), KeyModifiers::NONE);
        let result1 = matcher.process(&event1);
        assert!(result1.is_pending());

        // Second 'g' - complete
        let event2 = make_key_event(KeyCode::Char('g'), KeyModifiers::NONE);
        let result2 = matcher.process(&event2);
        assert!(result2.is_matched());
        assert_eq!(result2.action().unwrap().name(), "go_to_top");
    }

    #[test]
    fn test_pending_keys() {
        let mut matcher = InputMatcher::with_default_timeout();
        matcher.register(
            KeySequence::new(vec![
                KeyBinding::new(KeyCode::Char('a')),
                KeyBinding::new(KeyCode::Char('b')),
            ]),
            Action::new("ab"),
        );

        assert!(matcher.pending_keys().is_empty());

        let event = make_key_event(KeyCode::Char('a'), KeyModifiers::NONE);
        matcher.process(&event);

        assert_eq!(matcher.pending_keys().len(), 1);
        assert_eq!(matcher.pending_keys()[0].key(), KeyCode::Char('a'));
    }

    #[test]
    fn test_set_sequence_timeout() {
        let mut matcher = InputMatcher::with_default_timeout();
        assert_eq!(matcher.sequence_timeout(), Duration::from_secs(1));

        matcher.set_sequence_timeout(Duration::from_millis(500));
        assert_eq!(matcher.sequence_timeout(), Duration::from_millis(500));
    }

    #[test]
    fn test_overlapping_sequences() {
        let mut matcher = InputMatcher::with_default_timeout();

        // Single 'g' does something
        matcher.register_key(KeyBinding::new(KeyCode::Char('g')), Action::new("single_g"));

        // 'g' 'g' does something else
        matcher.register(
            KeySequence::new(vec![
                KeyBinding::new(KeyCode::Char('g')),
                KeyBinding::new(KeyCode::Char('g')),
            ]),
            Action::new("double_g"),
        );

        // First 'g' - should be pending because "gg" is possible
        let event1 = make_key_event(KeyCode::Char('g'), KeyModifiers::NONE);
        let result1 = matcher.process(&event1);
        assert!(result1.is_pending());

        // Second 'g' - should match "gg"
        let event2 = make_key_event(KeyCode::Char('g'), KeyModifiers::NONE);
        let result2 = matcher.process(&event2);
        assert!(result2.is_matched());
        assert_eq!(result2.action().unwrap().name(), "double_g");
    }
}
