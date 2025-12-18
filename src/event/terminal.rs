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
#[cfg(feature = "crossterm-backend")]
pub struct TerminalEventStream {
    /// Polling timeout for non-blocking event checks.
    poll_timeout: Duration,
}

#[cfg(feature = "crossterm-backend")]
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

#[cfg(feature = "crossterm-backend")]
impl Default for TerminalEventStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "crossterm-backend")]
impl std::fmt::Debug for TerminalEventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalEventStream")
            .field("poll_timeout", &self.poll_timeout)
            .finish()
    }
}

/// Stub for when crossterm-backend is not enabled.
#[cfg(not(feature = "crossterm-backend"))]
pub struct TerminalEventStream;

#[cfg(not(feature = "crossterm-backend"))]
impl TerminalEventStream {
    /// Creates a new terminal event stream (stub).
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "crossterm-backend"))]
impl Default for TerminalEventStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_event_stream_creation() {
        let stream = TerminalEventStream::new();
        #[cfg(feature = "crossterm-backend")]
        assert_eq!(stream.poll_timeout(), Duration::from_millis(10));
        let _ = stream;
    }

    #[cfg(feature = "crossterm-backend")]
    #[test]
    fn test_terminal_event_stream_with_timeout() {
        let stream = TerminalEventStream::with_timeout(Duration::from_millis(50));
        assert_eq!(stream.poll_timeout(), Duration::from_millis(50));
    }

    #[cfg(feature = "crossterm-backend")]
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

    #[cfg(feature = "crossterm-backend")]
    #[test]
    fn test_terminal_event_stream_debug() {
        let stream = TerminalEventStream::new();
        let debug_str = format!("{:?}", stream);
        assert!(debug_str.contains("TerminalEventStream"));
        assert!(debug_str.contains("poll_timeout"));
    }
}
