//! Main event loop implementation.
//!
//! This module provides the [`EventLoop`] struct that manages the async event loop
//! for TUI applications, integrating terminal events, tick timing, and shutdown signals.

use std::future::Future;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, trace};

use super::shutdown::ShutdownSignal;
use super::terminal::TerminalEventStream;
use crate::input::Action;

/// Application event types that flow through the event loop.
///
/// These events represent all the different things that can happen in a TUI application:
/// terminal input, matched actions, custom messages, render ticks, and shutdown signals.
///
/// # Examples
///
/// ```rust
/// use tuilib::event::AppEvent;
/// use tuilib::input::Action;
///
/// // Create different event types
/// let action_event = AppEvent::<String>::Action(Action::new("quit"));
/// let message_event = AppEvent::Message("Data loaded".to_string());
/// let tick_event = AppEvent::<String>::Tick;
/// let shutdown_event = AppEvent::<String>::Shutdown;
/// ```
#[derive(Debug, Clone)]
pub enum AppEvent<M = String> {
    /// A terminal event from crossterm.
    #[cfg(feature = "crossterm-backend")]
    Terminal(crossterm::event::Event),

    /// A matched keybinding action.
    Action(Action),

    /// A custom application message.
    Message(M),

    /// A render tick event (fires at the configured frame rate).
    Tick,

    /// A shutdown signal was received.
    Shutdown,
}

impl<M> AppEvent<M> {
    /// Returns true if this is a terminal event.
    #[cfg(feature = "crossterm-backend")]
    pub fn is_terminal(&self) -> bool {
        matches!(self, AppEvent::Terminal(_))
    }

    /// Returns true if this is an action event.
    pub fn is_action(&self) -> bool {
        matches!(self, AppEvent::Action(_))
    }

    /// Returns true if this is a message event.
    pub fn is_message(&self) -> bool {
        matches!(self, AppEvent::Message(_))
    }

    /// Returns true if this is a tick event.
    pub fn is_tick(&self) -> bool {
        matches!(self, AppEvent::Tick)
    }

    /// Returns true if this is a shutdown event.
    pub fn is_shutdown(&self) -> bool {
        matches!(self, AppEvent::Shutdown)
    }

    /// Returns the action if this is an action event.
    pub fn action(&self) -> Option<&Action> {
        match self {
            AppEvent::Action(action) => Some(action),
            _ => None,
        }
    }

    /// Returns the message if this is a message event.
    pub fn message(&self) -> Option<&M> {
        match self {
            AppEvent::Message(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Control flow signal returned by event handlers.
///
/// Handlers return this to indicate whether the event loop should continue
/// running or exit.
///
/// # Examples
///
/// ```rust
/// use tuilib::event::ControlFlow;
///
/// // Continue processing events
/// let continue_running = ControlFlow::Continue;
///
/// // Exit the event loop
/// let should_exit = ControlFlow::Exit;
///
/// assert!(!continue_running.should_exit());
/// assert!(should_exit.should_exit());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlFlow {
    /// Continue running the event loop.
    #[default]
    Continue,

    /// Exit the event loop.
    Exit,
}

impl ControlFlow {
    /// Returns true if the event loop should exit.
    pub fn should_exit(&self) -> bool {
        matches!(self, ControlFlow::Exit)
    }

    /// Returns true if the event loop should continue.
    pub fn should_continue(&self) -> bool {
        matches!(self, ControlFlow::Continue)
    }
}

/// Configuration for the event loop.
///
/// Controls timing behavior like tick rate and debounce delays.
///
/// # Examples
///
/// ```rust
/// use tuilib::event::EventLoopConfig;
/// use std::time::Duration;
///
/// // Default configuration (60 FPS, 50ms debounce)
/// let default_config = EventLoopConfig::default();
///
/// // Custom configuration
/// let custom_config = EventLoopConfig::new()
///     .tick_rate(Duration::from_millis(33))  // ~30 FPS
///     .debounce_delay(Duration::from_millis(100));
/// ```
#[derive(Debug, Clone)]
pub struct EventLoopConfig {
    /// How often to fire tick events (controls frame rate).
    pub tick_rate: Duration,

    /// Delay for debouncing rapid events.
    pub debounce_delay: Duration,

    /// Size of the internal message channel buffer.
    pub channel_buffer_size: usize,

    /// Whether to handle SIGINT/SIGTERM for graceful shutdown.
    pub handle_signals: bool,
}

impl EventLoopConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tick rate (frame rate).
    ///
    /// # Arguments
    ///
    /// * `rate` - Duration between tick events
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::EventLoopConfig;
    /// use std::time::Duration;
    ///
    /// // 30 FPS
    /// let config = EventLoopConfig::new()
    ///     .tick_rate(Duration::from_millis(33));
    /// ```
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }

    /// Sets the debounce delay.
    ///
    /// # Arguments
    ///
    /// * `delay` - Minimum time between processing duplicate events
    pub fn debounce_delay(mut self, delay: Duration) -> Self {
        self.debounce_delay = delay;
        self
    }

    /// Sets the channel buffer size.
    ///
    /// # Arguments
    ///
    /// * `size` - Buffer size for the internal message channel
    pub fn channel_buffer_size(mut self, size: usize) -> Self {
        self.channel_buffer_size = size;
        self
    }

    /// Sets whether to handle shutdown signals.
    ///
    /// # Arguments
    ///
    /// * `handle` - Whether to listen for SIGINT/SIGTERM
    pub fn handle_signals(mut self, handle: bool) -> Self {
        self.handle_signals = handle;
        self
    }
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(16), // ~60 FPS
            debounce_delay: Duration::from_millis(50),
            channel_buffer_size: 256,
            handle_signals: true,
        }
    }
}

/// The main event loop for TUI applications.
///
/// Manages terminal event polling, tick timing, message channels, and shutdown handling.
///
/// # Type Parameters
///
/// * `M` - The type of custom messages that can be sent through the event loop
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::event::{EventLoop, EventLoopConfig, AppEvent, ControlFlow};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = EventLoopConfig::default();
///     let mut event_loop: EventLoop<String> = EventLoop::new(config);
///
///     // Get sender for async tasks
///     let sender = event_loop.sender();
///
///     event_loop.run(|event| async move {
///         match event {
///             AppEvent::Shutdown => ControlFlow::Exit,
///             _ => ControlFlow::Continue,
///         }
///     }).await?;
///
///     Ok(())
/// }
/// ```
pub struct EventLoop<M = String> {
    config: EventLoopConfig,
    tx: mpsc::Sender<AppEvent<M>>,
    rx: mpsc::Receiver<AppEvent<M>>,
}

impl<M> EventLoop<M>
where
    M: Send + 'static,
{
    /// Creates a new event loop with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the event loop
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::{EventLoop, EventLoopConfig};
    ///
    /// let config = EventLoopConfig::default();
    /// let event_loop: EventLoop<String> = EventLoop::new(config);
    /// ```
    pub fn new(config: EventLoopConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.channel_buffer_size);
        Self { config, tx, rx }
    }

    /// Returns a sender that can be used to send events to the loop.
    ///
    /// This sender can be cloned and given to async tasks that need to
    /// communicate with the UI.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::{EventLoop, EventLoopConfig, AppEvent};
    ///
    /// let event_loop: EventLoop<String> = EventLoop::new(EventLoopConfig::default());
    /// let sender = event_loop.sender();
    ///
    /// // Sender can be cloned for multiple tasks
    /// let sender2 = sender.clone();
    /// ```
    pub fn sender(&self) -> mpsc::Sender<AppEvent<M>> {
        self.tx.clone()
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &EventLoopConfig {
        &self.config
    }

    /// Runs the event loop until exit is signaled.
    ///
    /// This method will block until the handler returns `ControlFlow::Exit`
    /// or a shutdown signal is received.
    ///
    /// # Arguments
    ///
    /// * `handler` - Async function that processes events and returns control flow
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on clean shutdown, or an error if something went wrong.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use tuilib::event::{EventLoop, EventLoopConfig, AppEvent, ControlFlow};
    ///
    /// let mut event_loop: EventLoop<String> = EventLoop::new(EventLoopConfig::default());
    ///
    /// event_loop.run(|event| async move {
    ///     match event {
    ///         AppEvent::Shutdown => ControlFlow::Exit,
    ///         AppEvent::Tick => {
    ///             // Render the UI
    ///             ControlFlow::Continue
    ///         }
    ///         _ => ControlFlow::Continue,
    ///     }
    /// }).await?;
    /// ```
    #[cfg(feature = "crossterm-backend")]
    pub async fn run<F, Fut>(&mut self, mut handler: F) -> std::io::Result<()>
    where
        F: FnMut(AppEvent<M>) -> Fut,
        Fut: Future<Output = ControlFlow>,
    {
        debug!(
            tick_rate_ms = ?self.config.tick_rate.as_millis(),
            "Starting event loop"
        );

        // Create terminal event stream
        let mut terminal_events = TerminalEventStream::new();

        // Create tick interval
        let mut tick_interval = tokio::time::interval(self.config.tick_rate);
        tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Create shutdown signal handler
        let mut shutdown = if self.config.handle_signals {
            Some(ShutdownSignal::new()?)
        } else {
            None
        };

        loop {
            let event = tokio::select! {
                // Terminal events
                Some(term_event) = terminal_events.next() => {
                    match term_event {
                        Ok(event) => {
                            trace!(?event, "Terminal event received");
                            AppEvent::Terminal(event)
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Terminal event error");
                            continue;
                        }
                    }
                }

                // Tick events
                _ = tick_interval.tick() => {
                    trace!("Tick event");
                    AppEvent::Tick
                }

                // Channel messages
                Some(msg) = self.rx.recv() => {
                    trace!("Channel message received");
                    msg
                }

                // Shutdown signal
                _ = async {
                    if let Some(ref mut s) = shutdown {
                        s.recv().await
                    } else {
                        std::future::pending::<()>().await
                    }
                } => {
                    debug!("Shutdown signal received");
                    AppEvent::Shutdown
                }
            };

            let control = handler(event).await;

            if control.should_exit() {
                debug!("Event loop exiting");
                break;
            }
        }

        Ok(())
    }

    /// Runs the event loop without terminal event handling.
    ///
    /// Useful for testing or headless operation where terminal input
    /// is not needed.
    pub async fn run_headless<F, Fut>(&mut self, mut handler: F) -> std::io::Result<()>
    where
        F: FnMut(AppEvent<M>) -> Fut,
        Fut: Future<Output = ControlFlow>,
    {
        debug!(
            tick_rate_ms = ?self.config.tick_rate.as_millis(),
            "Starting headless event loop"
        );

        // Create tick interval
        let mut tick_interval = tokio::time::interval(self.config.tick_rate);
        tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Create shutdown signal handler
        let mut shutdown = if self.config.handle_signals {
            Some(ShutdownSignal::new()?)
        } else {
            None
        };

        loop {
            let event = tokio::select! {
                // Tick events
                _ = tick_interval.tick() => {
                    trace!("Tick event");
                    AppEvent::Tick
                }

                // Channel messages
                Some(msg) = self.rx.recv() => {
                    trace!("Channel message received");
                    msg
                }

                // Shutdown signal
                _ = async {
                    if let Some(ref mut s) = shutdown {
                        s.recv().await
                    } else {
                        std::future::pending::<()>().await
                    }
                } => {
                    debug!("Shutdown signal received");
                    AppEvent::Shutdown
                }
            };

            let control = handler(event).await;

            if control.should_exit() {
                debug!("Headless event loop exiting");
                break;
            }
        }

        Ok(())
    }
}

impl<M> std::fmt::Debug for EventLoop<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLoop")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_flow() {
        let continue_flow = ControlFlow::Continue;
        let exit_flow = ControlFlow::Exit;

        assert!(!continue_flow.should_exit());
        assert!(continue_flow.should_continue());
        assert!(exit_flow.should_exit());
        assert!(!exit_flow.should_continue());
    }

    #[test]
    fn test_control_flow_default() {
        let default = ControlFlow::default();
        assert_eq!(default, ControlFlow::Continue);
    }

    #[test]
    fn test_event_loop_config_default() {
        let config = EventLoopConfig::default();
        assert_eq!(config.tick_rate, Duration::from_millis(16));
        assert_eq!(config.debounce_delay, Duration::from_millis(50));
        assert_eq!(config.channel_buffer_size, 256);
        assert!(config.handle_signals);
    }

    #[test]
    fn test_event_loop_config_builder() {
        let config = EventLoopConfig::new()
            .tick_rate(Duration::from_millis(33))
            .debounce_delay(Duration::from_millis(100))
            .channel_buffer_size(512)
            .handle_signals(false);

        assert_eq!(config.tick_rate, Duration::from_millis(33));
        assert_eq!(config.debounce_delay, Duration::from_millis(100));
        assert_eq!(config.channel_buffer_size, 512);
        assert!(!config.handle_signals);
    }

    #[test]
    fn test_event_loop_creation() {
        let config = EventLoopConfig::default();
        let event_loop: EventLoop<String> = EventLoop::new(config.clone());

        assert_eq!(event_loop.config().tick_rate, config.tick_rate);
    }

    #[test]
    fn test_event_loop_sender() {
        let event_loop: EventLoop<String> = EventLoop::new(EventLoopConfig::default());
        let _sender = event_loop.sender();
        let _sender2 = event_loop.sender();
        // Both senders are valid clones
    }

    #[test]
    fn test_app_event_helpers() {
        let action_event = AppEvent::<String>::Action(Action::new("test"));
        let message_event = AppEvent::Message("hello".to_string());
        let tick_event = AppEvent::<String>::Tick;
        let shutdown_event = AppEvent::<String>::Shutdown;

        assert!(action_event.is_action());
        assert!(!action_event.is_message());
        assert_eq!(action_event.action().unwrap().name(), "test");

        assert!(message_event.is_message());
        assert!(!message_event.is_action());
        assert_eq!(message_event.message().unwrap(), "hello");

        assert!(tick_event.is_tick());
        assert!(!tick_event.is_action());

        assert!(shutdown_event.is_shutdown());
        assert!(!shutdown_event.is_tick());
    }

    #[tokio::test]
    async fn test_event_loop_send_receive() {
        let mut event_loop: EventLoop<String> = EventLoop::new(
            EventLoopConfig::new()
                .handle_signals(false)
                .tick_rate(Duration::from_millis(100)),
        );

        let sender = event_loop.sender();

        // Send a message before running
        sender
            .send(AppEvent::Message("test".to_string()))
            .await
            .unwrap();
        sender.send(AppEvent::Shutdown).await.unwrap();

        let mut received = Vec::new();

        event_loop
            .run_headless(|event| {
                let is_shutdown = event.is_shutdown();
                received.push(event);
                async move {
                    if is_shutdown {
                        ControlFlow::Exit
                    } else {
                        ControlFlow::Continue
                    }
                }
            })
            .await
            .unwrap();

        // Should have received tick(s), message, and shutdown
        assert!(received.iter().any(|e| e.is_message()));
        assert!(received.iter().any(|e| e.is_shutdown()));
    }

    #[tokio::test]
    async fn test_event_loop_immediate_exit() {
        let mut event_loop: EventLoop<String> = EventLoop::new(
            EventLoopConfig::new()
                .handle_signals(false)
                .tick_rate(Duration::from_millis(10)),
        );

        let sender = event_loop.sender();
        sender.send(AppEvent::Shutdown).await.unwrap();

        let result = event_loop
            .run_headless(|event| async move {
                if event.is_shutdown() {
                    ControlFlow::Exit
                } else {
                    ControlFlow::Continue
                }
            })
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_event_loop_debug() {
        let event_loop: EventLoop<String> = EventLoop::new(EventLoopConfig::default());
        let debug_str = format!("{:?}", event_loop);
        assert!(debug_str.contains("EventLoop"));
        assert!(debug_str.contains("config"));
    }
}
