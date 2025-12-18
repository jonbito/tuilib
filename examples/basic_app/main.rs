//! Basic App Example: Demonstrating tuilib Phase 1 Infrastructure
//!
//! This example demonstrates the core features of tuilib's Phase 1 infrastructure:
//!
//! - **Event Loop**: Tokio-powered async event handling with configurable tick rate
//! - **Keybindings**: Declarative action mapping with the builder API
//! - **Focus Management**: Tab/Shift+Tab navigation between focusable "widgets"
//! - **Theming**: Dark/light themes with semantic colors
//! - **Tracing**: Structured logging for debugging TUI applications
//!
//! # Running the Example
//!
//! ```sh
//! cargo run --example basic_app
//! ```
//!
//! After running, check `basic_app.log` for trace output.
//!
//! # Controls
//!
//! - `Tab` / `Shift+Tab`: Navigate between widgets
//! - `Enter` / `Space`: Activate the focused widget
//! - `t`: Toggle between dark and light themes
//! - `Ctrl+q` / `q`: Quit the application
//!
//! # Architecture
//!
//! The application demonstrates a simple form with three focusable "buttons".
//! It showcases how to:
//!
//! 1. Set up tracing to write logs to a file
//! 2. Configure keybindings using the fluent builder API
//! 3. Create an InputMatcher to match terminal events to actions
//! 4. Use FocusManager to handle Tab navigation
//! 5. Run an async event loop that handles terminal input, ticks, and shutdown
//! 6. Render a themed UI using ratatui

use std::time::Duration;

use crossterm::event::KeyEventKind;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use terminput_crossterm::to_terminput_key;
use tuilib::{
    component_render_span, component_update_span,
    event::{restore_terminal, setup_terminal, AppEvent, ControlFlow, EventLoop, EventLoopConfig},
    focus::{FocusDirection, FocusId, FocusManager},
    focus_span,
    input::{Action, InputMatcher, KeyBindings},
    theme::Theme,
    tracing::{init_tracing, TracingConfig},
};

// =============================================================================
// Application State
// =============================================================================

/// The main application state.
struct App {
    /// Focus manager for handling Tab navigation between widgets
    focus_manager: FocusManager,
    /// Input matcher for mapping key events to semantic actions
    input_matcher: InputMatcher,
    /// Current theme (dark or light)
    theme: Theme,
    /// Whether the theme is dark (for toggling)
    is_dark_theme: bool,
    /// Counter to demonstrate widget activation
    counter: i32,
    /// Status message to display
    status: String,
    /// Whether the app should exit
    should_exit: bool,
}

impl App {
    /// Creates a new application with default state.
    fn new() -> Self {
        // =====================================================================
        // Step 1: Configure Keybindings
        // =====================================================================
        //
        // Use the fluent builder API to define semantic action bindings.
        // These bindings are stored in KeyBindings for documentation/config,
        // and registered with InputMatcher for runtime matching.

        let bindings = KeyBindings::builder()
            // Navigation bindings
            .bind("focus_next", "Tab")
            .bind("focus_prev", "Shift+Tab")
            // Activation bindings (multiple keys for the same action)
            .bind_multi("activate", &["Enter", "Space"])
            // Theme toggle
            .bind("toggle_theme", "t")
            // Quit bindings
            .bind_multi("quit", &["q", "Ctrl+q"])
            .build();

        tracing::info!(
            binding_count = bindings.total_count(),
            "Keybindings configured"
        );

        // =====================================================================
        // Step 2: Create InputMatcher from Keybindings
        // =====================================================================
        //
        // The InputMatcher maintains state for key sequence matching (e.g., for
        // multi-key sequences like Ctrl+X Ctrl+S). We register all our bindings.

        let mut input_matcher = InputMatcher::new(Duration::from_secs(1));

        // Register all global bindings with the matcher
        for (sequence, action) in bindings.global_bindings() {
            input_matcher.register(sequence.clone(), action.clone());
        }

        tracing::debug!(
            matcher_bindings = input_matcher.binding_count(),
            "Input matcher initialized"
        );

        // =====================================================================
        // Step 3: Set Up Focus Manager
        // =====================================================================
        //
        // Register focusable widgets with their IDs and order.
        // Lower order values focus first when using Tab navigation.

        let mut focus_manager = FocusManager::new();

        // Register three demo "buttons" with focus order
        focus_manager.register(FocusId::new("decrement-button"), 0);
        focus_manager.register(FocusId::new("reset-button"), 1);
        focus_manager.register(FocusId::new("increment-button"), 2);

        // Focus the first widget
        focus_manager.focus_next();

        tracing::info!(
            component_count = focus_manager.len(),
            current = ?focus_manager.current(),
            "Focus manager initialized"
        );

        Self {
            focus_manager,
            input_matcher,
            theme: Theme::dark(),
            is_dark_theme: true,
            counter: 0,
            status: "Press Tab to navigate, Enter/Space to activate".to_string(),
            should_exit: false,
        }
    }

    /// Handles an action from the input matcher.
    fn handle_action(&mut self, action: &Action) {
        let _span = component_update_span!("App");

        tracing::debug!(action = action.name(), "Handling action");

        match action.name() {
            "quit" => {
                tracing::info!("Quit action received");
                self.should_exit = true;
                self.status = "Goodbye!".to_string();
            }

            "focus_next" => {
                let _span = focus_span!("next");
                let result = self.focus_manager.navigate(FocusDirection::Next);
                tracing::debug!(?result, current = ?self.focus_manager.current(), "Focus next");
                self.update_status_from_focus();
            }

            "focus_prev" => {
                let _span = focus_span!("prev");
                let result = self.focus_manager.navigate(FocusDirection::Previous);
                tracing::debug!(?result, current = ?self.focus_manager.current(), "Focus prev");
                self.update_status_from_focus();
            }

            "activate" => {
                if let Some(current) = self.focus_manager.current() {
                    tracing::info!(widget = %current, "Widget activated");
                    self.activate_current_widget();
                }
            }

            "toggle_theme" => {
                self.is_dark_theme = !self.is_dark_theme;
                self.theme = if self.is_dark_theme {
                    Theme::dark()
                } else {
                    Theme::light()
                };
                tracing::info!(theme = self.theme.name(), "Theme toggled");
                self.status = format!("Theme: {}", self.theme.name());
            }

            _ => {
                tracing::warn!(action = action.name(), "Unknown action");
            }
        }
    }

    /// Activates the currently focused widget.
    fn activate_current_widget(&mut self) {
        if let Some(current) = self.focus_manager.current() {
            match current.as_str() {
                "decrement-button" => {
                    self.counter -= 1;
                    self.status = format!("Decremented! Counter: {}", self.counter);
                }
                "reset-button" => {
                    self.counter = 0;
                    self.status = "Counter reset to 0".to_string();
                }
                "increment-button" => {
                    self.counter += 1;
                    self.status = format!("Incremented! Counter: {}", self.counter);
                }
                _ => {}
            }
        }
    }

    /// Updates the status message based on current focus.
    fn update_status_from_focus(&mut self) {
        if let Some(current) = self.focus_manager.current() {
            self.status = format!("Focused: {}", current);
        }
    }

    /// Renders the UI.
    fn render(&self, frame: &mut Frame) {
        let _span = component_render_span!("App");

        let size = frame.area();

        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Counter display
                Constraint::Length(3), // Buttons row
                Constraint::Length(3), // Status bar
                Constraint::Min(0),    // Help text
            ])
            .split(size);

        // Render title
        self.render_title(frame, chunks[0]);

        // Render counter display
        self.render_counter(frame, chunks[1]);

        // Render button row
        self.render_buttons(frame, chunks[2]);

        // Render status bar
        self.render_status(frame, chunks[3]);

        // Render help text
        self.render_help(frame, chunks[4]);
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("tuilib Basic App Demo")
            .style(self.theme.heading_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(self.theme.border_style()),
            );
        frame.render_widget(title, area);
    }

    fn render_counter(&self, frame: &mut Frame, area: Rect) {
        let counter_text = format!("Counter: {}", self.counter);
        let style = if self.counter > 0 {
            self.theme.success_text_style()
        } else if self.counter < 0 {
            self.theme.error_text_style()
        } else {
            self.theme.primary_text_style()
        };

        let counter = Paragraph::new(counter_text)
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Value")
                    .title_style(self.theme.secondary_text_style())
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style()),
            );
        frame.render_widget(counter, area);
    }

    fn render_buttons(&self, frame: &mut Frame, area: Rect) {
        // Split into three equal columns for buttons
        let button_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(area);

        let buttons = [
            ("decrement-button", "[ - ]"),
            ("reset-button", "[Reset]"),
            ("increment-button", "[ + ]"),
        ];

        for (i, (id, label)) in buttons.iter().enumerate() {
            let is_focused = self.focus_manager.current().map(|c| c.as_str()) == Some(*id);
            self.render_button(frame, button_areas[i], label, is_focused);
        }
    }

    fn render_button(&self, frame: &mut Frame, area: Rect, label: &str, focused: bool) {
        let (style, border_style) = if focused {
            (
                self.theme.button_focused_style(),
                self.theme.border_focused_style(),
            )
        } else {
            (self.theme.button_normal_style(), self.theme.border_style())
        };

        let button = Paragraph::new(label)
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );
        frame.render_widget(button, area);
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let status = Paragraph::new(self.status.as_str())
            .style(self.theme.info_text_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Status")
                    .title_style(self.theme.secondary_text_style())
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style()),
            );
        frame.render_widget(status, area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![Line::from(vec![
            Span::styled("Tab", self.theme.emphasis_style()),
            Span::styled("/", self.theme.secondary_text_style()),
            Span::styled("Shift+Tab", self.theme.emphasis_style()),
            Span::styled(": Navigate  ", self.theme.secondary_text_style()),
            Span::styled("Enter", self.theme.emphasis_style()),
            Span::styled("/", self.theme.secondary_text_style()),
            Span::styled("Space", self.theme.emphasis_style()),
            Span::styled(": Activate  ", self.theme.secondary_text_style()),
            Span::styled("t", self.theme.emphasis_style()),
            Span::styled(": Toggle Theme  ", self.theme.secondary_text_style()),
            Span::styled("q", self.theme.emphasis_style()),
            Span::styled(": Quit", self.theme.secondary_text_style()),
        ])];

        let help = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(format!("Theme: {}", self.theme.name()))
                    .title_style(self.theme.muted_style())
                    .borders(Borders::TOP)
                    .border_style(self.theme.border_style()),
            );
        frame.render_widget(help, area);
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // =========================================================================
    // Step 1: Initialize Tracing
    // =========================================================================
    //
    // Set up file-based logging since TUI apps use stdout for rendering.
    // The guard must be kept alive for the application's duration.

    let config = TracingConfig::new()
        .with_log_file("basic_app.log")
        .with_level(tracing::Level::DEBUG)
        .with_target_level("tuilib", tracing::Level::TRACE)
        .with_timestamps(true)
        .with_target(true)
        .with_spans(true);

    let _guard = init_tracing(config)?;

    tracing::info!("=== Basic App Example Started ===");

    // =========================================================================
    // Step 2: Set Up Terminal
    // =========================================================================

    let mut terminal = setup_terminal()?;
    tracing::debug!("Terminal initialized");

    // =========================================================================
    // Step 3: Create Application State
    // =========================================================================

    let mut app = App::new();

    // =========================================================================
    // Step 4: Create and Run Event Loop
    // =========================================================================
    //
    // The event loop handles:
    // - Terminal events (keyboard, mouse)
    // - Tick events for rendering
    // - Shutdown signals (Ctrl+C)
    // - Custom messages (not used in this example)

    let config = EventLoopConfig::new()
        .tick_rate(Duration::from_millis(16)) // ~60 FPS
        .handle_signals(true);

    let mut event_loop: EventLoop<String> = EventLoop::new(config);

    tracing::info!("Starting event loop");

    // Initial render
    terminal.draw(|f| app.render(f))?;

    // Run the event loop
    // Note: We use a closure that processes events synchronously
    // The async block just wraps the synchronous result
    let result = event_loop
        .run(|event| {
            // Process the event and determine control flow
            let control = match &event {
                // Handle terminal events (keyboard, mouse, resize)
                AppEvent::Terminal(term_event) => {
                    if let crossterm::event::Event::Key(key_event) = term_event {
                        // Only process key press events (not release or repeat)
                        if key_event.kind == KeyEventKind::Press {
                            // Convert crossterm KeyEvent to terminput KeyEvent
                            if let Ok(terminput_event) = to_terminput_key(*key_event) {
                                // Match the key event against registered bindings
                                let result = app.input_matcher.process(&terminput_event);

                                if let Some(action) = result.into_action() {
                                    app.handle_action(&action);
                                }
                            }
                        }
                    }

                    // Check if we should exit
                    if app.should_exit {
                        ControlFlow::Exit
                    } else {
                        // Re-render after handling input
                        if let Err(e) = terminal.draw(|f| app.render(f)) {
                            tracing::error!(error = %e, "Render error");
                        }
                        ControlFlow::Continue
                    }
                }

                // Handle tick events for periodic updates
                AppEvent::Tick => {
                    // In a real app, you might update animations or fetch data here
                    ControlFlow::Continue
                }

                // Handle shutdown signal
                AppEvent::Shutdown => {
                    tracing::info!("Shutdown signal received");
                    ControlFlow::Exit
                }

                // Handle custom messages (not used in this example)
                AppEvent::Message(_) => ControlFlow::Continue,

                // Handle actions sent through the channel
                AppEvent::Action(action) => {
                    app.handle_action(action);
                    if app.should_exit {
                        ControlFlow::Exit
                    } else {
                        ControlFlow::Continue
                    }
                }
            };

            // Return an immediately-ready future with the control flow
            async move { control }
        })
        .await;

    // =========================================================================
    // Step 5: Cleanup
    // =========================================================================

    tracing::info!("=== Basic App Example Finished ===");

    restore_terminal(&mut terminal)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        return Err(e.into());
    }

    println!("Thank you for trying tuilib! Check basic_app.log for trace output.");
    Ok(())
}
