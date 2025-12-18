//! Event loop module for async TUI applications.
//!
//! This module provides the async event loop infrastructure for TUI applications,
//! integrating tokio with the rendering cycle. It handles terminal events,
//! channel-based communication between async tasks and UI, and graceful shutdown.
//!
//! # Overview
//!
//! The event loop is the core of a TUI application, responsible for:
//!
//! - Polling terminal events from crossterm
//! - Dispatching actions through the component hierarchy
//! - Managing a configurable render loop with frame rate limiting
//! - Handling signals for graceful shutdown
//! - Providing channels for async task communication
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                       EventLoop                              │
//! │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
//! │  │   Terminal    │  │    Tick       │  │   Shutdown    │   │
//! │  │   Events      │  │    Timer      │  │   Signals     │   │
//! │  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘   │
//! │          │                  │                  │            │
//! │          └──────────────────┼──────────────────┘            │
//! │                             ▼                               │
//! │                    ┌───────────────┐                        │
//! │                    │   AppEvent    │                        │
//! │                    │    Stream     │                        │
//! │                    └───────┬───────┘                        │
//! │                            │                                │
//! │                            ▼                                │
//! │                    ┌───────────────┐                        │
//! │                    │   Handler     │                        │
//! │                    │   Callback    │                        │
//! │                    └───────────────┘                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ## Basic Event Loop
//!
//! ```rust,ignore
//! use tuilib::event::{EventLoop, EventLoopConfig, AppEvent, ControlFlow};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = EventLoopConfig::default();
//!     let mut event_loop = EventLoop::new(config);
//!
//!     event_loop.run(|event| async move {
//!         match event {
//!             AppEvent::Terminal(term_event) => {
//!                 // Handle terminal input
//!                 ControlFlow::Continue
//!             }
//!             AppEvent::Tick => {
//!                 // Render frame
//!                 ControlFlow::Continue
//!             }
//!             AppEvent::Shutdown => ControlFlow::Exit,
//!             _ => ControlFlow::Continue,
//!         }
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Sending Messages from Async Tasks
//!
//! ```rust,ignore
//! use tuilib::event::{EventLoop, EventLoopConfig, AppEvent};
//!
//! let config = EventLoopConfig::default();
//! let event_loop = EventLoop::new(config);
//!
//! // Get a sender for external tasks
//! let sender = event_loop.sender();
//!
//! // Spawn an async task that sends messages
//! tokio::spawn(async move {
//!     sender.send(AppEvent::Message("Data loaded".to_string())).await.ok();
//! });
//! ```

mod event_loop;
mod shutdown;
mod terminal;
mod timing;

pub use event_loop::{AppEvent, ControlFlow, EventLoop, EventLoopConfig};
pub use shutdown::ShutdownSignal;
pub use terminal::TerminalEventStream;
pub use timing::{Debouncer, Throttle};
