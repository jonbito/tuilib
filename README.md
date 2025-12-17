# tuilib

An opinionated TUI component library with input action mapping built on [ratatui](https://github.com/ratatui/ratatui).

## Features

- **Component System**: Reusable, composable UI components following consistent patterns
- **Input Action Mapping**: Semantic input handling using [terminput](https://github.com/aschey/terminput)
- **Focus Management**: Built-in focus navigation between components
- **Theming**: Consistent styling with design tokens and color schemes
- **Async Event Loop**: Tokio-powered event handling for responsive applications

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tuilib = "0.1"
```

## Quick Start

```rust
use tuilib::prelude::*;

// Example usage will be documented as components are implemented
```

## Modules

| Module | Description |
|--------|-------------|
| `components` | UI components (buttons, inputs, modals, etc.) |
| `input` | Input action mapping and keyboard handling |
| `focus` | Focus management and navigation |
| `theme` | Theming and design tokens |
| `event` | Async event loop infrastructure |

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## License

MIT
