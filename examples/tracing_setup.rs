//! Example: Tracing Setup for TUI Applications
//!
//! This example demonstrates how to set up structured logging and tracing
//! in a TUI application. Since TUI applications use stdout for rendering,
//! logs are written to a file.
//!
//! # Running
//!
//! ```sh
//! cargo run --example tracing_setup --features tracing-setup
//! ```
//!
//! After running, check `debug.log` for the trace output.

use std::time::Duration;

use tuilib::focus::{FocusDirection, FocusId, FocusManager, FocusTrap};
use tuilib::tracing::{init_tracing, TracingConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with file-based logging
    // The guard must be kept alive for the duration of the application
    let config = TracingConfig::new()
        .with_log_file("debug.log")
        .with_level(tracing::Level::DEBUG)
        // Enable more verbose logging for specific modules
        .with_target_level("tuilib::focus", tracing::Level::TRACE)
        .with_timestamps(true)
        .with_target(true)
        .with_spans(true);

    let _guard = init_tracing(config)?;

    tracing::info!("Application started");

    // Demonstrate focus management with tracing
    demo_focus_navigation();

    // Demonstrate focus traps (modal dialogs) with tracing
    demo_focus_traps();

    // Demonstrate component lifecycle spans
    demo_component_spans();

    tracing::info!("Application finished - check debug.log for trace output");

    // Small delay to ensure logs are flushed
    std::thread::sleep(Duration::from_millis(100));

    println!("Example complete! Check debug.log for trace output.");
    Ok(())
}

/// Demonstrates focus navigation with tracing
fn demo_focus_navigation() {
    tracing::info!("Starting focus navigation demo");

    let mut focus_manager = FocusManager::new();

    // Register some focusable components
    focus_manager.register(FocusId::new("username-input"), 0);
    focus_manager.register(FocusId::new("password-input"), 0);
    focus_manager.register(FocusId::new("remember-checkbox"), 0);
    focus_manager.register(FocusId::new("submit-button"), 0);

    tracing::debug!(
        component_count = focus_manager.len(),
        "Registered focusable components"
    );

    // Navigate through components (Tab behavior)
    for _ in 0..5 {
        let result = focus_manager.navigate(FocusDirection::Next);
        tracing::debug!(?result, "Tab navigation");
    }

    // Navigate backwards (Shift+Tab behavior)
    for _ in 0..2 {
        let result = focus_manager.navigate(FocusDirection::Previous);
        tracing::debug!(?result, "Shift+Tab navigation");
    }

    tracing::info!("Focus navigation demo complete");
}

/// Demonstrates focus traps (modal dialogs) with tracing
fn demo_focus_traps() {
    tracing::info!("Starting focus trap demo");

    let mut focus_manager = FocusManager::new();

    // Main page components
    focus_manager.register(FocusId::new("main-nav"), 0);
    focus_manager.register(FocusId::new("main-content"), 0);
    focus_manager.focus_next();

    tracing::debug!("Main page ready, opening modal dialog");

    // Create a focus trap for a modal dialog
    let mut modal_trap = FocusTrap::new();
    modal_trap.register(FocusId::new("modal-title"), 0);
    modal_trap.register(FocusId::new("modal-input"), 0);
    modal_trap.register(FocusId::new("modal-ok"), 0);
    modal_trap.register(FocusId::new("modal-cancel"), 0);

    // Push the trap - this saves current focus and restricts navigation
    focus_manager.push_trap(modal_trap);

    tracing::debug!(
        current = ?focus_manager.current(),
        has_trap = focus_manager.has_trap(),
        "Modal opened"
    );

    // Navigate within the modal
    focus_manager.navigate(FocusDirection::Next);
    focus_manager.navigate(FocusDirection::Next);

    tracing::debug!("Closing modal dialog");

    // Pop the trap - restores previous focus
    focus_manager.pop_trap();

    tracing::debug!(
        current = ?focus_manager.current(),
        has_trap = focus_manager.has_trap(),
        "Modal closed, focus restored"
    );

    tracing::info!("Focus trap demo complete");
}

/// Demonstrates component lifecycle spans
fn demo_component_spans() {
    use tuilib::{component_render_span, component_update_span, focus_span};

    tracing::info!("Starting component span demo");

    // Simulate component update
    {
        let _span = component_update_span!("LoginForm");
        tracing::debug!(field = "username", "Processing form input");
        // Simulate some work
        std::thread::sleep(Duration::from_micros(100));
    }

    // Simulate component render
    {
        let _span = component_render_span!("LoginForm");
        tracing::trace!("Rendering form fields");
        // Simulate rendering
        std::thread::sleep(Duration::from_micros(50));
    }

    // Simulate focus operation
    {
        let _span = focus_span!("next");
        tracing::debug!(target = "submit-button", "Moving focus");
    }

    tracing::info!("Component span demo complete");
}
