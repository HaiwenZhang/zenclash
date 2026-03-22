#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building ZenClash..."

TARGET="${1:-release}"
PLATFORM="${2:-$(uname -s)}"

case "$PLATFORM" in
    Linux)
        echo "Building for Linux..."
        cargo build --release --target x86_64-unknown-linux-gnu
        ;;
    Darwin)
        echo "Building for macOS..."
        cargo build --release --target x86_64-apple-darwin
        cargo build --release --target aarch64-apple-darwin
        ;;
    *)
        echo "Building for current platform..."
        cargo build --"$TARGET"
        ;;
esac

echo "Build complete!"
