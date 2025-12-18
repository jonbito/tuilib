//! Tracing configuration and setup for TUI applications.
//!
//! This module provides helpers for setting up structured logging and tracing
//! in TUI applications. Since TUI applications use stdout for rendering, logs
//! must be directed to files or other outputs.
//!
//! # Overview
//!
//! The tracing system provides:
//!
//! - Structured logging with configurable levels
//! - File-based log output (required since stdout is used for rendering)
//! - Per-module filtering
//! - Performance spans for component lifecycle events
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use tuilib::tracing::{TracingConfig, init_tracing};
//!
//! // Initialize with file logging
//! let config = TracingConfig::new()
//!     .with_log_file("app.log")
//!     .with_level(tracing::Level::DEBUG);
//!
//! let _guard = init_tracing(config)?;
//!
//! // Tracing is now active - spans and events will be written to the log file
//! ```
//!
//! # Lifecycle Spans
//!
//! The library automatically creates spans for key lifecycle events:
//!
//! - **Component Update**: Tracks message processing in components
//! - **Component Render**: Tracks render performance
//! - **Focus Navigation**: Tracks focus changes between components
//! - **Event Loop**: Tracks event processing
//!
//! # Module-Level Filtering
//!
//! You can configure different log levels per module:
//!
//! ```rust,ignore
//! use tuilib::tracing::TracingConfig;
//!
//! let config = TracingConfig::new()
//!     .with_log_file("app.log")
//!     .with_level(tracing::Level::INFO)
//!     .with_target_level("tuilib::event", tracing::Level::DEBUG)
//!     .with_target_level("tuilib::focus", tracing::Level::TRACE);
//! ```

use std::path::PathBuf;

/// Configuration for tracing initialization.
///
/// Use the builder methods to configure logging behavior before calling
/// [`init_tracing`].
///
/// # Examples
///
/// ```rust
/// use tuilib::tracing::TracingConfig;
///
/// // Basic configuration with file logging
/// let config = TracingConfig::new()
///     .with_log_file("debug.log")
///     .with_level(tracing::Level::DEBUG);
///
/// // Configuration with per-module filtering
/// let config = TracingConfig::new()
///     .with_log_file("app.log")
///     .with_level(tracing::Level::INFO)
///     .with_target_level("tuilib::event", tracing::Level::DEBUG);
/// ```
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// The default log level.
    pub level: tracing::Level,
    /// Path to the log file. Required for TUI applications.
    pub log_file: Option<PathBuf>,
    /// Per-target log level overrides.
    pub target_levels: Vec<(String, tracing::Level)>,
    /// Whether to include timestamps in log output.
    pub include_timestamps: bool,
    /// Whether to include target (module path) in log output.
    pub include_target: bool,
    /// Whether to include file and line information.
    pub include_file_line: bool,
    /// Whether to include span information.
    pub include_spans: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: tracing::Level::INFO,
            log_file: None,
            target_levels: Vec::new(),
            include_timestamps: true,
            include_target: true,
            include_file_line: false,
            include_spans: true,
        }
    }
}

impl TracingConfig {
    /// Creates a new tracing configuration with default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::tracing::TracingConfig;
    ///
    /// let config = TracingConfig::new();
    /// assert_eq!(config.level, tracing::Level::INFO);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the default log level.
    ///
    /// # Arguments
    ///
    /// * `level` - The minimum level for log messages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::tracing::TracingConfig;
    ///
    /// let config = TracingConfig::new()
    ///     .with_level(tracing::Level::DEBUG);
    /// ```
    pub fn with_level(mut self, level: tracing::Level) -> Self {
        self.level = level;
        self
    }

    /// Sets the log file path.
    ///
    /// This is required for TUI applications since stdout is used for rendering.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the log file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::tracing::TracingConfig;
    ///
    /// let config = TracingConfig::new()
    ///     .with_log_file("logs/app.log");
    /// ```
    pub fn with_log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.log_file = Some(path.into());
        self
    }

    /// Adds a per-target log level override.
    ///
    /// This allows different log levels for different modules.
    ///
    /// # Arguments
    ///
    /// * `target` - The module path to configure (e.g., "tuilib::event")
    /// * `level` - The log level for this target
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::tracing::TracingConfig;
    ///
    /// let config = TracingConfig::new()
    ///     .with_level(tracing::Level::INFO)
    ///     .with_target_level("tuilib::event", tracing::Level::DEBUG)
    ///     .with_target_level("tuilib::focus", tracing::Level::TRACE);
    /// ```
    pub fn with_target_level(mut self, target: impl Into<String>, level: tracing::Level) -> Self {
        self.target_levels.push((target.into(), level));
        self
    }

    /// Sets whether to include timestamps in log output.
    ///
    /// Default is `true`.
    pub fn with_timestamps(mut self, include: bool) -> Self {
        self.include_timestamps = include;
        self
    }

    /// Sets whether to include the target (module path) in log output.
    ///
    /// Default is `true`.
    pub fn with_target(mut self, include: bool) -> Self {
        self.include_target = include;
        self
    }

    /// Sets whether to include file and line information in log output.
    ///
    /// Default is `false`.
    pub fn with_file_line(mut self, include: bool) -> Self {
        self.include_file_line = include;
        self
    }

    /// Sets whether to include span information in log output.
    ///
    /// Default is `true`.
    pub fn with_spans(mut self, include: bool) -> Self {
        self.include_spans = include;
        self
    }

    /// Builds the filter directive string for tracing-subscriber.
    ///
    /// This creates a filter string like "info,tuilib::event=debug,tuilib::focus=trace".
    pub fn build_filter_directive(&self) -> String {
        let mut parts = vec![self.level.as_str().to_lowercase()];

        for (target, level) in &self.target_levels {
            parts.push(format!("{}={}", target, level.as_str().to_lowercase()));
        }

        parts.join(",")
    }
}

/// Guard that must be kept alive for the duration of tracing.
///
/// When this guard is dropped, the tracing subscriber is flushed and removed.
/// Keep this guard alive for the entire lifetime of your application.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::tracing::{TracingConfig, init_tracing};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = TracingConfig::new().with_log_file("app.log");
///
///     // Keep the guard alive for the entire main function
///     let _guard = init_tracing(config)?;
///
///     // Your application code here...
///
///     Ok(())
/// } // Guard is dropped here, flushing any buffered logs
/// ```
pub struct TracingGuard {
    _worker_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl std::fmt::Debug for TracingGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TracingGuard").finish()
    }
}

/// Error type for tracing initialization failures.
#[derive(Debug)]
pub enum TracingError {
    /// No log file was specified in the configuration.
    NoLogFile,
    /// Failed to create the log file or directory.
    IoError(std::io::Error),
    /// Failed to set the global subscriber.
    SetGlobalError(String),
}

impl std::fmt::Display for TracingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracingError::NoLogFile => {
                write!(
                    f,
                    "No log file specified. TUI applications require file-based logging."
                )
            }
            TracingError::IoError(e) => write!(f, "IO error: {}", e),
            TracingError::SetGlobalError(e) => write!(f, "Failed to set global subscriber: {}", e),
        }
    }
}

impl std::error::Error for TracingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TracingError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TracingError {
    fn from(err: std::io::Error) -> Self {
        TracingError::IoError(err)
    }
}

/// Initializes the tracing subscriber with the given configuration.
///
/// This function sets up file-based logging for TUI applications. Since TUI
/// applications use stdout for rendering, all tracing output is directed to
/// the specified log file.
///
/// # Arguments
///
/// * `config` - The tracing configuration
///
/// # Returns
///
/// A [`TracingGuard`] that must be kept alive for the duration of logging.
/// When the guard is dropped, any buffered logs are flushed.
///
/// # Errors
///
/// Returns an error if:
/// - No log file is specified in the configuration
/// - The log file or directory cannot be created
/// - The global subscriber cannot be set
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::tracing::{TracingConfig, init_tracing};
///
/// let config = TracingConfig::new()
///     .with_log_file("debug.log")
///     .with_level(tracing::Level::DEBUG);
///
/// let _guard = init_tracing(config)?;
///
/// tracing::info!("Application started");
/// tracing::debug!(component = "button", "Button rendered");
/// ```
pub fn init_tracing(config: TracingConfig) -> Result<TracingGuard, TracingError> {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::prelude::*;

    let log_file = config.log_file.clone().ok_or(TracingError::NoLogFile)?;

    // Create parent directories if needed
    if let Some(parent) = log_file.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create non-blocking file appender
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    // Build the filter
    let filter = tracing_subscriber::EnvFilter::try_new(config.build_filter_directive())
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Determine span events
    let span_events = if config.include_spans {
        FmtSpan::ENTER | FmtSpan::EXIT
    } else {
        FmtSpan::NONE
    };

    // Build the formatting layer with all options applied at once
    // We need to handle the different type configurations separately to avoid
    // incompatible types in if/else branches
    let subscriber = tracing_subscriber::registry().with(filter);

    if config.include_timestamps {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(config.include_target)
            .with_file(config.include_file_line)
            .with_line_number(config.include_file_line)
            .with_span_events(span_events);

        let subscriber = subscriber.with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| TracingError::SetGlobalError(e.to_string()))?;
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .without_time()
            .with_target(config.include_target)
            .with_file(config.include_file_line)
            .with_line_number(config.include_file_line)
            .with_span_events(span_events);

        let subscriber = subscriber.with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| TracingError::SetGlobalError(e.to_string()))?;
    }

    Ok(TracingGuard {
        _worker_guard: guard,
    })
}

/// Creates a span for component update operations.
///
/// This is a helper macro for instrumenting component update methods.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::tracing::component_update_span;
///
/// impl Component for MyComponent {
///     type Message = MyMessage;
///     type Action = MyAction;
///
///     fn update(&mut self, msg: Self::Message) -> Option<Self::Action> {
///         let _span = component_update_span!("MyComponent");
///         // ... update logic
///     }
/// }
/// ```
#[macro_export]
macro_rules! component_update_span {
    ($component:expr) => {
        tracing::info_span!("component_update", component = $component).entered()
    };
    ($component:expr, $($field:tt)*) => {
        tracing::info_span!("component_update", component = $component, $($field)*).entered()
    };
}

/// Creates a span for component render operations.
///
/// This is a helper macro for instrumenting component render methods.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::tracing::component_render_span;
///
/// impl Renderable for MyComponent {
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         let _span = component_render_span!("MyComponent");
///         // ... render logic
///     }
/// }
/// ```
#[macro_export]
macro_rules! component_render_span {
    ($component:expr) => {
        tracing::debug_span!("component_render", component = $component).entered()
    };
    ($component:expr, $($field:tt)*) => {
        tracing::debug_span!("component_render", component = $component, $($field)*).entered()
    };
}

/// Creates a span for focus navigation operations.
///
/// This is a helper macro for instrumenting focus navigation.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::tracing::focus_span;
///
/// fn handle_tab(&mut self) {
///     let _span = focus_span!("next");
///     self.focus_manager.focus_next();
/// }
/// ```
#[macro_export]
macro_rules! focus_span {
    ($direction:expr) => {
        tracing::debug_span!("focus_navigation", direction = $direction).entered()
    };
    ($direction:expr, $($field:tt)*) => {
        tracing::debug_span!("focus_navigation", direction = $direction, $($field)*).entered()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TracingConfig::default();
        assert_eq!(config.level, tracing::Level::INFO);
        assert!(config.log_file.is_none());
        assert!(config.target_levels.is_empty());
        assert!(config.include_timestamps);
        assert!(config.include_target);
        assert!(!config.include_file_line);
        assert!(config.include_spans);
    }

    #[test]
    fn test_config_builder() {
        let config = TracingConfig::new()
            .with_level(tracing::Level::DEBUG)
            .with_log_file("test.log")
            .with_target_level("tuilib::event", tracing::Level::TRACE)
            .with_timestamps(false)
            .with_target(false)
            .with_file_line(true)
            .with_spans(false);

        assert_eq!(config.level, tracing::Level::DEBUG);
        assert_eq!(config.log_file, Some(PathBuf::from("test.log")));
        assert_eq!(config.target_levels.len(), 1);
        assert_eq!(config.target_levels[0].0, "tuilib::event");
        assert_eq!(config.target_levels[0].1, tracing::Level::TRACE);
        assert!(!config.include_timestamps);
        assert!(!config.include_target);
        assert!(config.include_file_line);
        assert!(!config.include_spans);
    }

    #[test]
    fn test_build_filter_directive() {
        let config = TracingConfig::new()
            .with_level(tracing::Level::INFO)
            .with_target_level("tuilib::event", tracing::Level::DEBUG)
            .with_target_level("tuilib::focus", tracing::Level::TRACE);

        let directive = config.build_filter_directive();
        assert_eq!(directive, "info,tuilib::event=debug,tuilib::focus=trace");
    }

    #[test]
    fn test_build_filter_directive_default() {
        let config = TracingConfig::new();
        let directive = config.build_filter_directive();
        assert_eq!(directive, "info");
    }

    #[test]
    fn test_tracing_error_display() {
        let err = TracingError::NoLogFile;
        assert!(err.to_string().contains("No log file"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = TracingError::IoError(io_err);
        assert!(err.to_string().contains("IO error"));

        let err = TracingError::SetGlobalError("already set".to_string());
        assert!(err.to_string().contains("already set"));
    }

    #[test]
    fn test_tracing_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: TracingError = io_err.into();
        assert!(matches!(err, TracingError::IoError(_)));
    }
}
