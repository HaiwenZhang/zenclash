# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ZenClash is a Mihomo (Clash Meta) proxy GUI built with Rust and GPUI (Zed's UI framework). It manages a Mihomo subprocess and provides a desktop interface for proxy configuration, traffic monitoring, and rule management.

## Build Commands

```bash
make build          # Release build
make build-dev      # Debug build
make test           # Run all tests
make test-release   # Run tests in release mode
make check          # Quick check (cargo check)
make fmt            # Format code
make clippy         # Run lints (strict: -D warnings)
make run            # Run GUI application (zenclash-ui)
make all            # Run check + fmt + clippy + test
```

For running specific tests:
```bash
cargo test -p zenclash-core          # Test only core crate
cargo test test_core_state           # Run specific test by name
cargo test --test integration_test   # Run integration tests
```

## Architecture

### Workspace Structure

- **zenclash-core**: Core logic - process management, API client, configuration, proxies, rules
- **zenclash-ui**: GPUI-based GUI with pages and components
- **zenclash-cli**: CLI entry point (minimal, mostly for testing)

### Key Components

**CoreManager** (`core/manager.rs`): Central orchestrator that:
- Manages Mihomo subprocess lifecycle (start/stop/restart)
- Holds ApiClient for RESTful API communication
- Provides proxy selection, delay testing, traffic monitoring
- Handles system proxy and TUN mode toggling

**ApiClient** (`core/api.rs`): HTTP/WebSocket client for Mihomo's API:
- Traffic streaming via WebSocket
- Connection management
- Proxy selection and delay testing
- Config hot-reload

**ConfigFactory** (`core/factory.rs`): Generates runtime Mihomo config from profiles.

### State Management Pattern

- `Arc<RwLock<T>>` for shared mutable state across threads
- `parking_lot::RwLock` preferred over std::RwLock for performance
- GPUI uses its own Entity<T> system for UI state

### Tokio + GPUI Integration

GPUI has its own async system. When calling Tokio async code from GPUI contexts:

```rust
cx.spawn(async move |this, cx| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // Tokio async operations here
        })
    })
})
.detach();
```

The main.rs creates a static Tokio runtime and enters it before GPUI initialization.

### UI Architecture

**Pages** (`ui/pages/`): Top-level views (Dashboard, Proxies, Profiles, Connections, Rules, Logs, Settings, etc.)

**Components** (`ui/components/`): Reusable UI widgets (Sidebar, ProxyItem, ProfileItem, ConnectionTable, etc.)

**App** (`ui/app.rs`): Main application struct holding all page entities and navigation state.

## Configuration Paths

- `data_dir()`: Application data directory
- `config_dir()`: Configuration files
- `profiles_dir()`: Profile YAML files
- `cache_dir()`: Cache data

All defined in `utils/dirs.rs`.

## Platform Support

Primary: macOS. Secondary: Linux. Platform-specific code in `sys/` and `system/` modules.

## Important Patterns

- **Prelude exports**: Each crate exports commonly used types via `prelude` module
- **Error handling**: Uses `thiserror` for custom error types, `anyhow` for general errors
- **Logging**: `tracing` crate with subscriber setup in `utils/logger.rs`
- **WebSocket streaming**: Traffic and logs use WebSocket streams with futures-util