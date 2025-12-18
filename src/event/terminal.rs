//! Terminal event stream for reading terminal input.
//!
//! This module provides an async stream of terminal events using crossterm.

use std::time::Duration;

/// An async stream of terminal events.
///
/// Wraps crossterm's event polling in an async-friendly interface
/// that can be used with tokio's select! macro.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::event::TerminalEventStream;
///
/// let mut events = TerminalEventStream::new();
///
/// while let Some(result) = events.next().await {
///     match result {
///         Ok(event) => println!("Got event: {:?}", event),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct TerminalEventStream {
    /// Polling timeout for non-blocking event checks.
    poll_timeout: Duration,
}

impl TerminalEventStream {
    /// Creates a new terminal event stream.
    pub fn new() -> Self {
        Self {
            poll_timeout: Duration::from_millis(10),
        }
    }

    /// Creates a terminal event stream with a custom poll timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - How long to wait when polling for events
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            poll_timeout: timeout,
        }
    }

    /// Attempts to get the next terminal event.
    ///
    /// This method is cancel-safe and can be used in tokio::select!
    ///
    /// # Returns
    ///
    /// - `Some(Ok(event))` if an event was received
    /// - `Some(Err(e))` if there was an error reading events
    /// - `None` is never returned (the stream is infinite)
    pub async fn next(&mut self) -> Option<std::io::Result<crossterm::event::Event>> {
        loop {
            // Use spawn_blocking to poll in a thread pool
            let timeout = self.poll_timeout;
            let result = tokio::task::spawn_blocking(move || {
                if crossterm::event::poll(timeout)? {
                    crossterm::event::read().map(Some)
                } else {
                    Ok(None)
                }
            })
            .await;

            match result {
                Ok(Ok(Some(event))) => return Some(Ok(event)),
                Ok(Ok(None)) => {
                    // No event available, yield and try again
                    tokio::task::yield_now().await;
                    continue;
                }
                Ok(Err(e)) => return Some(Err(e)),
                Err(e) => {
                    return Some(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Task join error: {}", e),
                    )))
                }
            }
        }
    }

    /// Returns the poll timeout duration.
    pub fn poll_timeout(&self) -> Duration {
        self.poll_timeout
    }

    /// Sets a new poll timeout.
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }
}

impl Default for TerminalEventStream {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TerminalEventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalEventStream")
            .field("poll_timeout", &self.poll_timeout)
            .finish()
    }
}

/// Sets up the terminal for a TUI application.
///
/// This function performs the standard terminal setup sequence:
/// - Enables raw mode (disabling line buffering and echoing)
/// - Switches to the alternate screen buffer
/// - Enables mouse capture
///
/// # Returns
///
/// A configured `Terminal` with a crossterm backend, ready for rendering.
///
/// # Errors
///
/// Returns an IO error if terminal setup fails.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::event::setup_terminal;
///
/// let mut terminal = setup_terminal()?;
/// // Use terminal for rendering...
/// ```
pub fn setup_terminal(
) -> std::io::Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    ratatui::Terminal::new(backend)
}

/// Restores the terminal to its original state.
///
/// This function reverses the setup performed by [`setup_terminal`]:
/// - Disables raw mode
/// - Leaves the alternate screen buffer
/// - Disables mouse capture
/// - Shows the cursor
///
/// # Arguments
///
/// * `terminal` - A mutable reference to the terminal to restore
///
/// # Errors
///
/// Returns an IO error if terminal restoration fails.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::event::{setup_terminal, restore_terminal};
///
/// let mut terminal = setup_terminal()?;
/// // Use terminal for rendering...
/// restore_terminal(&mut terminal)?;
/// ```
pub fn restore_terminal(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_event_stream_creation() {
        let stream = TerminalEventStream::new();
        assert_eq!(stream.poll_timeout(), Duration::from_millis(10));
    }

    #[test]
    fn test_terminal_event_stream_with_timeout() {
        let stream = TerminalEventStream::with_timeout(Duration::from_millis(50));
        assert_eq!(stream.poll_timeout(), Duration::from_millis(50));
    }

    #[test]
    fn test_terminal_event_stream_set_timeout() {
        let mut stream = TerminalEventStream::new();
        stream.set_poll_timeout(Duration::from_millis(100));
        assert_eq!(stream.poll_timeout(), Duration::from_millis(100));
    }

    #[test]
    fn test_terminal_event_stream_default() {
        let stream = TerminalEventStream::default();
        let _ = stream;
    }

    #[test]
    fn test_terminal_event_stream_debug() {
        let stream = TerminalEventStream::new();
        let debug_str = format!("{:?}", stream);
        assert!(debug_str.contains("TerminalEventStream"));
        assert!(debug_str.contains("poll_timeout"));
    }
}
