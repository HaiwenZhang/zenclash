# Contributing to ZenClash

## Development Setup

1. Clone the repository
2. Install Rust 1.80+
3. Run `make install-deps`
4. Run `make build-dev` to verify the build

## Code Style

- Format: `make fmt`
- Lint: `make clippy`
- Test: `make test`

All CI checks must pass before merging.

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make all` to check everything
5. Submit a pull request

## Commit Messages

Use clear, descriptive commit messages explaining the "why" not just the "what".
