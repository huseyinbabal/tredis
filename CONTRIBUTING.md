# Contributing to tredis

Thank you for your interest in contributing to tredis! This document provides guidelines and information for contributors.

## Before You Start

**Important:** Before adding a major feature, please start a discussion in our [GitHub Discussions](https://github.com/huseyinbabal/tredis/discussions) board. This helps us:

- Avoid duplicate work
- Discuss the best approach
- Ensure the feature aligns with project goals
- Get community feedback

## How to Contribute

1. **Fork the repository**
2. **Create your feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add some amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/tredis.git
cd tredis

# Build the project
cargo build

# Run in development mode
cargo run

# Run with debug logging
cargo run -- --log-level debug

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## Architecture

tredis follows a simple architecture with separation between UI, application state, and data models.

```
src/
├── main.rs          # Entry point, event loop, key handling
├── app.rs           # Application state and Redis operations
├── model.rs         # Data structures (KeyInfo, ServerConfig, etc.)
└── ui/
    ├── mod.rs       # UI module exports
    ├── header.rs    # Header with keybindings
    ├── keys_table.rs    # Keys browser
    ├── servers_table.rs # Server management
    ├── streams_table.rs # Redis Streams view
    ├── pubsub_table.rs  # PubSub channels
    ├── monitor_table.rs # Command monitor
    ├── clients_table.rs # Connected clients
    ├── info_view.rs     # Server info
    ├── configs_table.rs # Redis configuration
    ├── slowlog_table.rs # Slow query log
    ├── acls_table.rs    # ACL users
    ├── describe.rs      # Key/resource details
    ├── dialog.rs        # Confirmation dialogs
    ├── server_dialog.rs # Add server dialog
    ├── resources.rs     # Resource picker
    └── splash.rs        # Connection splash screen
```

## Adding a New Resource View

To add support for a new Redis resource type:

### 1. Start a Discussion

Before writing any code, [open a discussion](https://github.com/huseyinbabal/tredis/discussions/new?category=ideas) to propose the new feature.

### 2. Add the Data Model

Add the data structure to `src/model.rs`:

```rust
#[derive(Debug, Clone)]
pub struct MyResource {
    pub name: String,
    pub value: String,
    // ... other fields
}
```

### 3. Add State to App

Add the state fields to `src/app.rs`:

```rust
pub struct App {
    // ... existing fields
    pub my_resources: Vec<MyResource>,
    pub selected_my_resource_index: usize,
}
```

### 4. Add Fetch Function

Add the Redis fetch function to `src/app.rs`:

```rust
pub async fn fetch_my_resources(&mut self) -> Result<()> {
    if let Some(con) = &mut self.connection {
        // Redis commands here
    }
    Ok(())
}
```

### 5. Create UI Component

Create `src/ui/my_resource_table.rs`:

```rust
use crate::app::App;
use ratatui::{Frame, layout::Rect};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Render your table/view here
}
```

### 6. Register in Resources

Add to the resources list in `src/app.rs`:

```rust
ResourceItem { 
    name: "MyResource".to_string(), 
    command: "myresource".to_string(), 
    description: "Description here".to_string()
},
```

### 7. Add Key Handlers

Add navigation and refresh handlers in `src/main.rs`.

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Write descriptive commit messages
- Add comments for complex logic

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Update documentation if needed
- Ensure all tests pass
- Reference any related issues or discussions

## Questions?

If you have questions, feel free to:

- Open a [Discussion](https://github.com/huseyinbabal/tredis/discussions)
- Check existing issues and PRs

Thank you for contributing!
