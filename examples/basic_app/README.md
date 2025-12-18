# Basic App Example

This example demonstrates the core features of tuilib's Phase 1 infrastructure:

- **Event Loop**: Tokio-powered async event handling with configurable tick rate
- **Keybindings**: Declarative action mapping with the builder API
- **Focus Management**: Tab/Shift+Tab navigation between focusable widgets
- **Theming**: Dark/light themes with semantic colors
- **Tracing**: Structured logging for debugging TUI applications

## Running the Example

```sh
cargo run --example basic_app
```

After running, check `basic_app.log` for trace output.

## Controls

| Key | Action |
|-----|--------|
| `Tab` | Move focus to next widget |
| `Shift+Tab` | Move focus to previous widget |
| `Enter` / `Space` | Activate the focused widget |
| `t` | Toggle between dark and light themes |
| `q` / `Ctrl+q` | Quit the application |

## What This Example Demonstrates

### 1. Tracing Setup

TUI applications can't use stdout for logging since it's used for rendering. This example shows how to set up file-based tracing:

```rust
let config = TracingConfig::new()
    .with_log_file("basic_app.log")
    .with_level(tracing::Level::DEBUG)
    .with_timestamps(true)
    .with_spans(true);

let _guard = init_tracing(config)?;
```

### 2. Keybindings Configuration

Use the fluent builder API to define semantic actions:

```rust
let bindings = KeyBindings::builder()
    .bind("focus_next", "Tab")
    .bind("focus_prev", "Shift+Tab")
    .bind_multi("activate", &["Enter", "Space"])
    .bind_multi("quit", &["q", "Ctrl+q"])
    .build();
```

### 3. Input Matching

The `InputMatcher` matches terminal key events against registered bindings:

```rust
let mut input_matcher = InputMatcher::new(Duration::from_secs(1));

for (sequence, action) in bindings.global_bindings() {
    input_matcher.register(sequence.clone(), action.clone());
}

// Later, in the event loop:
let result = input_matcher.process(&key_event);
if let Some(action) = result.into_action() {
    app.handle_action(&action);
}
```

### 4. Focus Management

Register focusable widgets and navigate between them:

```rust
let mut focus_manager = FocusManager::new();

focus_manager.register(FocusId::new("button-1"), 0);
focus_manager.register(FocusId::new("button-2"), 1);
focus_manager.register(FocusId::new("button-3"), 2);

// Navigate with Tab/Shift+Tab
focus_manager.navigate(FocusDirection::Next);
focus_manager.navigate(FocusDirection::Previous);

// Check current focus
if let Some(current) = focus_manager.current() {
    println!("Focused: {}", current);
}
```

### 5. Theming

Apply consistent styling with semantic theme colors:

```rust
let theme = Theme::dark();

// Get styles for different states
let normal_style = theme.button_normal_style();
let focused_style = theme.button_focused_style();
let border_style = theme.border_focused_style();

// Toggle themes
let theme = if is_dark { Theme::dark() } else { Theme::light() };
```

### 6. Event Loop

Run the async event loop to handle terminal events:

```rust
let config = EventLoopConfig::new()
    .tick_rate(Duration::from_millis(16)) // ~60 FPS
    .handle_signals(true);

let mut event_loop: EventLoop<String> = EventLoop::new(config);

event_loop.run(|event| async move {
    match event {
        AppEvent::Terminal(term_event) => {
            // Handle keyboard/mouse input
            ControlFlow::Continue
        }
        AppEvent::Tick => {
            // Render or update animations
            ControlFlow::Continue
        }
        AppEvent::Shutdown => ControlFlow::Exit,
        _ => ControlFlow::Continue,
    }
}).await?;
```

## Application Architecture

```
+------------------------------------------+
|           Basic App Demo                  |
+------------------------------------------+
|                                          |
|              Counter: 0                  |
|                                          |
+------------------------------------------+
|   [ - ]    |   [Reset]   |    [ + ]     |
+------------------------------------------+
|  Status: Press Tab to navigate...        |
+------------------------------------------+
| Tab/Shift+Tab: Navigate  Enter: Activate |
+------------------------------------------+
```

The app demonstrates:
- A counter display that changes color based on value
- Three focusable buttons for decrement, reset, and increment
- A status bar showing current state
- A help bar with keyboard shortcuts

## Tracing Output

After running, check `basic_app.log` for structured trace output:

```
2024-01-15T10:00:00.123456Z  INFO basic_app: === Basic App Example Started ===
2024-01-15T10:00:00.123789Z  INFO basic_app: Keybindings configured binding_count=6
2024-01-15T10:00:00.123890Z  INFO basic_app: Focus manager initialized component_count=3
2024-01-15T10:00:00.124000Z DEBUG basic_app: Handling action action="focus_next"
2024-01-15T10:00:00.124100Z DEBUG focus{direction=next}: Focus moved to reset-button
```

## Next Steps

This example provides a foundation for building more complex TUI applications. Consider:

1. Adding more widget types (text inputs, lists)
2. Implementing focus traps for modal dialogs
3. Using context-scoped keybindings
4. Adding async background tasks via the event loop sender
