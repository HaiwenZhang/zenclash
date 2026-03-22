# ZenClash

Mihomo GUI built with Rust and GPUI,GPUI Component.

## Building

### Prerequisites

- Rust 1.80+
- System dependencies (run `./scripts/install-deps.sh`)

### Build Commands

```bash
# Install dependencies
make install-deps

# Build release
make build

# Build debug
make build-dev

# Run tests
make test

# Format code
make fmt

# Run lints
make clippy

# Run application
make run
```

### Using Scripts

```bash
# Build for current platform
./scripts/build.sh

# Package for distribution
./scripts/package.sh 0.1.0
```

## Project Structure

```
zenclash/
├── crates/
│   ├── zenclash-core/    # Core logic
│   ├── zenclash-ui/      # UI components
│   └── zenclash-cli/     # CLI entry point
├── platforms/            # Platform-specific files
├── scripts/              # Build scripts
└── docs/                 # Documentation
```

## License

MIT
