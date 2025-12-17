//! KeyBindings container and builder API with context support.
//!
//! This module provides the [`KeyBindings`] struct for storing keybindings organized
//! by context, and a fluent [`KeyBindingsBuilder`] API for declarative configuration.
//!
//! # Overview
//!
//! The keybindings system supports:
//! - Global bindings that apply everywhere
//! - Context-scoped bindings for different UI states (modal, component-specific)
//! - Multiple keys mapping to the same action
//! - Both programmatic and configuration-file-based setup
//!
//! # Examples
//!
//! ```rust
//! use tuilib::input::{KeyBindings, Action};
//!
//! let bindings = KeyBindings::builder()
//!     .bind("quit", "Ctrl+q")
//!     .bind("save", "Ctrl+s")
//!     .bind_multi("navigate_up", &["k", "Up"])
//!     .context("modal", |ctx| {
//!         ctx.bind("close", "Escape")
//!            .bind("confirm", "Enter")
//!     })
//!     .build();
//!
//! // Look up actions
//! use tuilib::input::parser::parse_key_binding;
//! let ctrl_q = parse_key_binding("Ctrl+q").unwrap();
//! let action = bindings.lookup(None, &ctrl_q.into());
//! assert_eq!(action.map(|a| a.name()), Some("quit"));
//! ```

use std::collections::HashMap;

use serde::Deserialize;

use super::parser::{parse_key_sequence, ParseKeyError};
use super::{Action, KeyBinding, KeySequence};

/// Container for all keybindings organized by context.
///
/// Keybindings are organized into:
/// - Global bindings: Apply in all contexts
/// - Context bindings: Apply only within specific named contexts
///
/// When looking up an action, the context bindings are checked first,
/// then global bindings as a fallback.
#[derive(Debug, Clone, Default)]
pub struct KeyBindings {
    /// Global bindings that apply everywhere
    global: HashMap<KeySequence, Action>,
    /// Context-specific bindings
    contexts: HashMap<String, HashMap<KeySequence, Action>>,
}

impl KeyBindings {
    /// Creates a new empty KeyBindings instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new KeyBindingsBuilder for fluent construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBindings;
    ///
    /// let bindings = KeyBindings::builder()
    ///     .bind("quit", "q")
    ///     .build();
    /// ```
    pub fn builder() -> KeyBindingsBuilder {
        KeyBindingsBuilder::new()
    }

    /// Looks up the action for a key sequence in the given context.
    ///
    /// If a context is provided, it first searches that context's bindings,
    /// then falls back to global bindings. If no context is provided,
    /// only global bindings are searched.
    ///
    /// # Arguments
    ///
    /// * `context` - Optional context name to search in
    /// * `sequence` - The key sequence to look up
    ///
    /// # Returns
    ///
    /// The action bound to the sequence, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::{KeyBindings, KeySequence, KeyBinding};
    /// use terminput::KeyCode;
    ///
    /// let bindings = KeyBindings::builder()
    ///     .bind("quit", "q")
    ///     .context("modal", |ctx| ctx.bind("close", "Escape"))
    ///     .build();
    ///
    /// let q_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
    ///
    /// // Global lookup
    /// assert!(bindings.lookup(None, &q_seq).is_some());
    ///
    /// // Context lookup (falls back to global)
    /// assert!(bindings.lookup(Some("modal"), &q_seq).is_some());
    /// ```
    pub fn lookup(&self, context: Option<&str>, sequence: &KeySequence) -> Option<&Action> {
        // First check context-specific bindings
        if let Some(ctx_name) = context {
            if let Some(ctx_bindings) = self.contexts.get(ctx_name) {
                if let Some(action) = ctx_bindings.get(sequence) {
                    return Some(action);
                }
            }
        }

        // Fall back to global bindings
        self.global.get(sequence)
    }

    /// Looks up the action for a single key binding.
    ///
    /// This is a convenience method that wraps the binding in a sequence.
    pub fn lookup_key(&self, context: Option<&str>, binding: &KeyBinding) -> Option<&Action> {
        let sequence = KeySequence::single(binding.clone());
        self.lookup(context, &sequence)
    }

    /// Returns all global bindings.
    pub fn global_bindings(&self) -> &HashMap<KeySequence, Action> {
        &self.global
    }

    /// Returns the bindings for a specific context.
    pub fn context_bindings(&self, context: &str) -> Option<&HashMap<KeySequence, Action>> {
        self.contexts.get(context)
    }

    /// Returns all context names.
    pub fn context_names(&self) -> impl Iterator<Item = &str> {
        self.contexts.keys().map(|s| s.as_str())
    }

    /// Returns the number of global bindings.
    pub fn global_count(&self) -> usize {
        self.global.len()
    }

    /// Returns the total number of bindings across all contexts.
    pub fn total_count(&self) -> usize {
        let context_count: usize = self.contexts.values().map(|c| c.len()).sum();
        self.global.len() + context_count
    }

    /// Merges another KeyBindings into this one.
    ///
    /// Bindings from `other` will override bindings in `self` for
    /// the same key sequence.
    pub fn merge(&mut self, other: KeyBindings) {
        self.global.extend(other.global);
        for (ctx, bindings) in other.contexts {
            self.contexts.entry(ctx).or_default().extend(bindings);
        }
    }
}

/// Builder for creating [`KeyBindings`] with a fluent API.
///
/// # Examples
///
/// ```rust
/// use tuilib::input::KeyBindingsBuilder;
///
/// let bindings = KeyBindingsBuilder::new()
///     .bind("quit", "Ctrl+q")
///     .bind("save", "Ctrl+s")
///     .bind_multi("up", &["k", "Up"])
///     .context("modal", |ctx| {
///         ctx.bind("close", "Escape")
///            .bind("confirm", "Enter")
///     })
///     .build();
/// ```
#[derive(Default)]
pub struct KeyBindingsBuilder {
    global: HashMap<KeySequence, Action>,
    contexts: HashMap<String, HashMap<KeySequence, Action>>,
    errors: Vec<ParseKeyError>,
}

impl KeyBindingsBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds an action to a key combination string.
    ///
    /// The key string is parsed to support formats like:
    /// - `"q"` - single key
    /// - `"Ctrl+s"` - key with modifier
    /// - `"Ctrl+x Ctrl+s"` - key sequence
    ///
    /// # Arguments
    ///
    /// * `action` - The action name
    /// * `keys` - The key combination string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBindingsBuilder;
    ///
    /// let bindings = KeyBindingsBuilder::new()
    ///     .bind("quit", "Ctrl+q")
    ///     .bind("vim_quit", "Z Q")
    ///     .build();
    /// ```
    pub fn bind(mut self, action: impl Into<Action>, keys: &str) -> Self {
        match parse_key_sequence(keys) {
            Ok(sequence) => {
                self.global.insert(sequence, action.into());
            }
            Err(e) => {
                self.errors.push(e);
            }
        }
        self
    }

    /// Binds an action to multiple key combinations.
    ///
    /// This is useful for having multiple ways to trigger the same action,
    /// like vim's 'k' and arrow Up for moving up.
    ///
    /// # Arguments
    ///
    /// * `action` - The action name
    /// * `keys` - Array of key combination strings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBindingsBuilder;
    ///
    /// let bindings = KeyBindingsBuilder::new()
    ///     .bind_multi("up", &["k", "Up"])
    ///     .bind_multi("down", &["j", "Down"])
    ///     .build();
    /// ```
    pub fn bind_multi(mut self, action: impl Into<Action>, keys: &[&str]) -> Self {
        let action = action.into();
        for key_str in keys {
            match parse_key_sequence(key_str) {
                Ok(sequence) => {
                    self.global.insert(sequence, action.clone());
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }
        self
    }

    /// Binds an action to a pre-parsed KeySequence.
    ///
    /// Use this when you have already constructed the key sequence
    /// programmatically.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The key sequence
    /// * `action` - The action
    pub fn bind_sequence(mut self, sequence: KeySequence, action: impl Into<Action>) -> Self {
        self.global.insert(sequence, action.into());
        self
    }

    /// Binds an action to a pre-parsed KeyBinding (single key).
    ///
    /// # Arguments
    ///
    /// * `binding` - The key binding
    /// * `action` - The action
    pub fn bind_key(self, binding: KeyBinding, action: impl Into<Action>) -> Self {
        self.bind_sequence(KeySequence::single(binding), action)
    }

    /// Creates a scoped context for bindings.
    ///
    /// Context bindings are checked before global bindings when looking
    /// up actions within that context.
    ///
    /// # Arguments
    ///
    /// * `name` - The context name (e.g., "modal", "edit", "list")
    /// * `f` - A function that configures the context bindings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::input::KeyBindingsBuilder;
    ///
    /// let bindings = KeyBindingsBuilder::new()
    ///     .bind("quit", "Ctrl+q")
    ///     .context("modal", |ctx| {
    ///         ctx.bind("close", "Escape")
    ///            .bind("confirm", "Enter")
    ///     })
    ///     .context("edit", |ctx| {
    ///         ctx.bind("save", "Ctrl+s")
    ///            .bind("cancel", "Escape")
    ///     })
    ///     .build();
    /// ```
    pub fn context<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ContextBuilder) -> ContextBuilder,
    {
        let ctx_builder = ContextBuilder::new();
        let ctx_builder = f(ctx_builder);

        self.contexts.insert(name.to_string(), ctx_builder.bindings);
        self.errors.extend(ctx_builder.errors);
        self
    }

    /// Returns any parse errors that occurred during building.
    ///
    /// This allows you to check for and report invalid key strings
    /// without stopping the build process.
    pub fn errors(&self) -> &[ParseKeyError] {
        &self.errors
    }

    /// Returns true if any parse errors occurred.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Builds the KeyBindings, consuming the builder.
    ///
    /// Note: Parse errors are silently ignored. Use `errors()` before
    /// calling `build()` if you want to handle them.
    pub fn build(self) -> KeyBindings {
        KeyBindings {
            global: self.global,
            contexts: self.contexts,
        }
    }

    /// Builds the KeyBindings, returning an error if any parse errors occurred.
    ///
    /// # Returns
    ///
    /// `Ok(KeyBindings)` if all key strings parsed successfully, or
    /// `Err(Vec<ParseKeyError>)` if any parsing failed.
    pub fn try_build(self) -> Result<KeyBindings, Vec<ParseKeyError>> {
        if self.errors.is_empty() {
            Ok(KeyBindings {
                global: self.global,
                contexts: self.contexts,
            })
        } else {
            Err(self.errors)
        }
    }
}

/// Builder for context-scoped bindings.
///
/// This builder is used within the `context()` method of [`KeyBindingsBuilder`]
/// to define bindings that only apply within a specific context.
#[derive(Default)]
pub struct ContextBuilder {
    bindings: HashMap<KeySequence, Action>,
    errors: Vec<ParseKeyError>,
}

impl ContextBuilder {
    /// Creates a new empty context builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds an action to a key combination string within this context.
    ///
    /// # Arguments
    ///
    /// * `action` - The action name
    /// * `keys` - The key combination string
    pub fn bind(mut self, action: impl Into<Action>, keys: &str) -> Self {
        match parse_key_sequence(keys) {
            Ok(sequence) => {
                self.bindings.insert(sequence, action.into());
            }
            Err(e) => {
                self.errors.push(e);
            }
        }
        self
    }

    /// Binds an action to multiple key combinations within this context.
    ///
    /// # Arguments
    ///
    /// * `action` - The action name
    /// * `keys` - Array of key combination strings
    pub fn bind_multi(mut self, action: impl Into<Action>, keys: &[&str]) -> Self {
        let action = action.into();
        for key_str in keys {
            match parse_key_sequence(key_str) {
                Ok(sequence) => {
                    self.bindings.insert(sequence, action.clone());
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }
        self
    }

    /// Binds an action to a pre-parsed KeySequence within this context.
    pub fn bind_sequence(mut self, sequence: KeySequence, action: impl Into<Action>) -> Self {
        self.bindings.insert(sequence, action.into());
        self
    }

    /// Binds an action to a pre-parsed KeyBinding within this context.
    pub fn bind_key(self, binding: KeyBinding, action: impl Into<Action>) -> Self {
        self.bind_sequence(KeySequence::single(binding), action)
    }
}

/// Configuration structure for deserializing keybindings from files.
///
/// This can be used with serde to load keybindings from TOML, JSON, or YAML
/// configuration files.
///
/// # Example TOML Configuration
///
/// ```toml
/// [global]
/// quit = "Ctrl+q"
/// save = "Ctrl+s"
/// navigate_up = ["k", "Up"]
///
/// [contexts.modal]
/// close = "Escape"
/// confirm = "Enter"
///
/// [contexts.edit]
/// save = "Ctrl+s"
/// cancel = "Escape"
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct KeyBindingsConfig {
    /// Global keybindings
    #[serde(default)]
    pub global: HashMap<String, KeyOrKeys>,
    /// Context-specific keybindings
    #[serde(default)]
    pub contexts: HashMap<String, HashMap<String, KeyOrKeys>>,
}

/// Represents either a single key string or multiple key strings.
///
/// This allows configuration files to use either:
/// ```toml
/// quit = "Ctrl+q"           # Single key
/// navigate_up = ["k", "Up"] # Multiple keys
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum KeyOrKeys {
    Single(String),
    Multiple(Vec<String>),
}

impl KeyBindingsConfig {
    /// Converts the configuration into a [`KeyBindings`] instance.
    ///
    /// # Returns
    ///
    /// `Ok(KeyBindings)` on success, or `Err(Vec<ParseKeyError>)` if
    /// any key strings failed to parse.
    pub fn into_key_bindings(self) -> Result<KeyBindings, Vec<ParseKeyError>> {
        let mut builder = KeyBindingsBuilder::new();

        // Add global bindings
        for (action, keys) in self.global {
            match keys {
                KeyOrKeys::Single(key) => {
                    builder = builder.bind(action.clone(), &key);
                }
                KeyOrKeys::Multiple(key_list) => {
                    let key_refs: Vec<&str> = key_list.iter().map(|s| s.as_str()).collect();
                    builder = builder.bind_multi(action.clone(), &key_refs);
                }
            }
        }

        // Add context bindings
        for (ctx_name, bindings) in self.contexts {
            builder = builder.context(&ctx_name, |mut ctx| {
                for (action, keys) in bindings {
                    match keys {
                        KeyOrKeys::Single(key) => {
                            ctx = ctx.bind(action.clone(), &key);
                        }
                        KeyOrKeys::Multiple(key_list) => {
                            let key_refs: Vec<&str> = key_list.iter().map(|s| s.as_str()).collect();
                            ctx = ctx.bind_multi(action.clone(), &key_refs);
                        }
                    }
                }
                ctx
            });
        }

        builder.try_build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminput::KeyCode;

    #[test]
    fn test_builder_basic() {
        let bindings = KeyBindings::builder().bind("quit", "q").build();

        let q_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
        let action = bindings.lookup(None, &q_seq);
        assert_eq!(action.map(|a| a.name()), Some("quit"));
    }

    #[test]
    fn test_builder_with_modifier() {
        let bindings = KeyBindings::builder().bind("save", "Ctrl+s").build();

        let ctrl_s = KeySequence::single(KeyBinding::with_mods(
            KeyCode::Char('s'),
            terminput::KeyModifiers::CTRL,
        ));
        let action = bindings.lookup(None, &ctrl_s);
        assert_eq!(action.map(|a| a.name()), Some("save"));
    }

    #[test]
    fn test_builder_multi() {
        let bindings = KeyBindings::builder()
            .bind_multi("up", &["k", "Up"])
            .build();

        let k_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('k')));
        let up_seq = KeySequence::single(KeyBinding::new(KeyCode::Up));

        assert_eq!(bindings.lookup(None, &k_seq).map(|a| a.name()), Some("up"));
        assert_eq!(bindings.lookup(None, &up_seq).map(|a| a.name()), Some("up"));
    }

    #[test]
    fn test_builder_context() {
        let bindings = KeyBindings::builder()
            .bind("quit", "q")
            .context("modal", |ctx| ctx.bind("close", "Escape"))
            .build();

        let esc_seq = KeySequence::single(KeyBinding::new(KeyCode::Esc));

        // In modal context, Escape closes
        let action = bindings.lookup(Some("modal"), &esc_seq);
        assert_eq!(action.map(|a| a.name()), Some("close"));

        // Outside modal, no action for Escape
        assert!(bindings.lookup(None, &esc_seq).is_none());
    }

    #[test]
    fn test_context_fallback_to_global() {
        let bindings = KeyBindings::builder()
            .bind("quit", "q")
            .context("modal", |ctx| ctx.bind("close", "Escape"))
            .build();

        let q_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));

        // In modal context, 'q' still triggers quit from global
        let action = bindings.lookup(Some("modal"), &q_seq);
        assert_eq!(action.map(|a| a.name()), Some("quit"));
    }

    #[test]
    fn test_context_override_global() {
        let bindings = KeyBindings::builder()
            .bind("global_action", "Escape")
            .context("modal", |ctx| ctx.bind("close", "Escape"))
            .build();

        let esc_seq = KeySequence::single(KeyBinding::new(KeyCode::Esc));

        // In modal context, Escape triggers context action (not global)
        let action = bindings.lookup(Some("modal"), &esc_seq);
        assert_eq!(action.map(|a| a.name()), Some("close"));

        // Globally, Escape triggers global action
        let action = bindings.lookup(None, &esc_seq);
        assert_eq!(action.map(|a| a.name()), Some("global_action"));
    }

    #[test]
    fn test_builder_sequence() {
        let bindings = KeyBindings::builder().bind("save", "Ctrl+x Ctrl+s").build();

        let save_seq = KeySequence::new(vec![
            KeyBinding::with_mods(KeyCode::Char('x'), terminput::KeyModifiers::CTRL),
            KeyBinding::with_mods(KeyCode::Char('s'), terminput::KeyModifiers::CTRL),
        ]);

        let action = bindings.lookup(None, &save_seq);
        assert_eq!(action.map(|a| a.name()), Some("save"));
    }

    #[test]
    fn test_builder_bind_key() {
        let bindings = KeyBindings::builder()
            .bind_key(
                KeyBinding::with_mods(KeyCode::Char('s'), terminput::KeyModifiers::CTRL),
                "save",
            )
            .build();

        let ctrl_s = KeySequence::single(KeyBinding::with_mods(
            KeyCode::Char('s'),
            terminput::KeyModifiers::CTRL,
        ));
        assert!(bindings.lookup(None, &ctrl_s).is_some());
    }

    #[test]
    fn test_builder_errors() {
        let builder = KeyBindingsBuilder::new()
            .bind("valid", "q")
            .bind("invalid", "NotAValidKey");

        assert!(builder.has_errors());
        assert_eq!(builder.errors().len(), 1);
    }

    #[test]
    fn test_try_build_with_errors() {
        let result = KeyBindingsBuilder::new()
            .bind("invalid", "NotAValidKey")
            .try_build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn test_try_build_success() {
        let result = KeyBindingsBuilder::new().bind("quit", "q").try_build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_key_bindings_counts() {
        let bindings = KeyBindings::builder()
            .bind("quit", "q")
            .bind("save", "Ctrl+s")
            .context("modal", |ctx| ctx.bind("close", "Escape"))
            .build();

        assert_eq!(bindings.global_count(), 2);
        assert_eq!(bindings.total_count(), 3);
    }

    #[test]
    fn test_key_bindings_context_names() {
        let bindings = KeyBindings::builder()
            .context("modal", |ctx| ctx.bind("close", "Escape"))
            .context("edit", |ctx| ctx.bind("save", "Ctrl+s"))
            .build();

        let names: Vec<&str> = bindings.context_names().collect();
        assert!(names.contains(&"modal"));
        assert!(names.contains(&"edit"));
    }

    #[test]
    fn test_key_bindings_merge() {
        let mut base = KeyBindings::builder().bind("quit", "q").build();

        let overlay = KeyBindings::builder()
            .bind("save", "Ctrl+s")
            .bind("quit_alt", "Q") // Add alternative quit binding
            .context("modal", |ctx| ctx.bind("close", "Escape"))
            .build();

        base.merge(overlay);

        // Now has both the original quit and the merged bindings
        assert!(bindings_has_action(&base, None, "q", "quit").is_some()); // Original still present
        assert!(bindings_has_action(&base, None, "Q", "quit_alt").is_some()); // New binding added
        assert!(bindings_has_action(&base, None, "Ctrl+s", "save").is_some());
        assert!(bindings_has_action(&base, Some("modal"), "Escape", "close").is_some());
    }

    fn bindings_has_action(
        bindings: &KeyBindings,
        context: Option<&str>,
        keys: &str,
        expected_action: &str,
    ) -> Option<()> {
        let seq = parse_key_sequence(keys).ok()?;
        let action = bindings.lookup(context, &seq)?;
        if action.name() == expected_action {
            Some(())
        } else {
            None
        }
    }

    #[test]
    fn test_config_single_key() {
        let config = KeyBindingsConfig {
            global: [("quit".to_string(), KeyOrKeys::Single("q".to_string()))]
                .into_iter()
                .collect(),
            contexts: HashMap::new(),
        };

        let bindings = config.into_key_bindings().unwrap();
        let q_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
        assert!(bindings.lookup(None, &q_seq).is_some());
    }

    #[test]
    fn test_config_multiple_keys() {
        let config = KeyBindingsConfig {
            global: [(
                "up".to_string(),
                KeyOrKeys::Multiple(vec!["k".to_string(), "Up".to_string()]),
            )]
            .into_iter()
            .collect(),
            contexts: HashMap::new(),
        };

        let bindings = config.into_key_bindings().unwrap();
        let k_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('k')));
        let up_seq = KeySequence::single(KeyBinding::new(KeyCode::Up));

        assert!(bindings.lookup(None, &k_seq).is_some());
        assert!(bindings.lookup(None, &up_seq).is_some());
    }

    #[test]
    fn test_config_with_contexts() {
        let config = KeyBindingsConfig {
            global: [("quit".to_string(), KeyOrKeys::Single("q".to_string()))]
                .into_iter()
                .collect(),
            contexts: [(
                "modal".to_string(),
                [("close".to_string(), KeyOrKeys::Single("Escape".to_string()))]
                    .into_iter()
                    .collect(),
            )]
            .into_iter()
            .collect(),
        };

        let bindings = config.into_key_bindings().unwrap();
        let esc_seq = KeySequence::single(KeyBinding::new(KeyCode::Esc));

        assert!(bindings.lookup(Some("modal"), &esc_seq).is_some());
        assert!(bindings.lookup(None, &esc_seq).is_none());
    }

    #[test]
    fn test_lookup_key() {
        let bindings = KeyBindings::builder().bind("quit", "q").build();

        let q_binding = KeyBinding::new(KeyCode::Char('q'));
        let action = bindings.lookup_key(None, &q_binding);
        assert_eq!(action.map(|a| a.name()), Some("quit"));
    }

    #[test]
    fn test_context_multi_bindings() {
        let bindings = KeyBindings::builder()
            .context("modal", |ctx| ctx.bind_multi("close", &["Escape", "q"]))
            .build();

        let esc_seq = KeySequence::single(KeyBinding::new(KeyCode::Esc));
        let q_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));

        assert!(bindings.lookup(Some("modal"), &esc_seq).is_some());
        assert!(bindings.lookup(Some("modal"), &q_seq).is_some());
    }

    #[test]
    fn test_empty_bindings() {
        let bindings = KeyBindings::new();

        let q_seq = KeySequence::single(KeyBinding::new(KeyCode::Char('q')));
        assert!(bindings.lookup(None, &q_seq).is_none());
        assert_eq!(bindings.global_count(), 0);
        assert_eq!(bindings.total_count(), 0);
    }
}
