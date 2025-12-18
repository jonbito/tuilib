//! Graceful shutdown signal handling.
//!
//! This module provides utilities for handling SIGINT and SIGTERM signals
//! to enable graceful application shutdown.

use std::io;

/// A signal handler for graceful shutdown.
///
/// Listens for SIGINT (Ctrl+C) and SIGTERM signals and provides an async
/// interface to wait for shutdown requests.
///
/// # Examples
///
/// ```rust,ignore
/// use tuilib::event::ShutdownSignal;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let mut shutdown = ShutdownSignal::new()?;
///
///     tokio::select! {
///         _ = shutdown.recv() => {
///             println!("Shutdown signal received!");
///         }
///         _ = async_work() => {
///             println!("Work completed!");
///         }
///     }
///
///     Ok(())
/// }
/// ```
#[cfg(unix)]
pub struct ShutdownSignal {
    sigint: tokio::signal::unix::Signal,
    sigterm: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl ShutdownSignal {
    /// Creates a new shutdown signal handler.
    ///
    /// Registers handlers for SIGINT and SIGTERM.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ShutdownSignal)` if signal handlers were registered successfully,
    /// or an `Err` if registration failed.
    pub fn new() -> io::Result<Self> {
        use tokio::signal::unix::{signal, SignalKind};

        let sigint = signal(SignalKind::interrupt())?;
        let sigterm = signal(SignalKind::terminate())?;

        Ok(Self { sigint, sigterm })
    }

    /// Waits for a shutdown signal.
    ///
    /// This method completes when either SIGINT or SIGTERM is received.
    /// It can be used in a `tokio::select!` to handle shutdown alongside
    /// other async operations.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut shutdown = ShutdownSignal::new()?;
    ///
    /// // Wait for shutdown
    /// shutdown.recv().await;
    /// println!("Shutting down...");
    /// ```
    pub async fn recv(&mut self) {
        tokio::select! {
            _ = self.sigint.recv() => {
                tracing::debug!("SIGINT received");
            }
            _ = self.sigterm.recv() => {
                tracing::debug!("SIGTERM received");
            }
        }
    }
}

#[cfg(unix)]
impl std::fmt::Debug for ShutdownSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShutdownSignal").finish()
    }
}

/// Windows implementation using Ctrl+C handler.
#[cfg(windows)]
pub struct ShutdownSignal {
    ctrl_c: tokio::sync::oneshot::Receiver<()>,
}

#[cfg(windows)]
impl ShutdownSignal {
    /// Creates a new shutdown signal handler.
    pub fn new() -> io::Result<Self> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Register Ctrl+C handler
        ctrlc::set_handler(move || {
            let _ = tx.send(());
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Self { ctrl_c: rx })
    }

    /// Waits for a shutdown signal.
    pub async fn recv(&mut self) {
        let _ = (&mut self.ctrl_c).await;
        tracing::debug!("Ctrl+C received");
    }
}

#[cfg(windows)]
impl std::fmt::Debug for ShutdownSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShutdownSignal").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn test_shutdown_signal_creation() {
        let result = ShutdownSignal::new();
        assert!(result.is_ok());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_shutdown_signal_debug() {
        let signal = ShutdownSignal::new().unwrap();
        let debug_str = format!("{:?}", signal);
        assert!(debug_str.contains("ShutdownSignal"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_shutdown_signal_recv_timeout() {
        let mut signal = ShutdownSignal::new().unwrap();

        // Use a timeout to ensure we don't hang
        let result =
            tokio::time::timeout(std::time::Duration::from_millis(10), signal.recv()).await;

        // Should timeout since no signal was sent
        assert!(result.is_err());
    }
}
