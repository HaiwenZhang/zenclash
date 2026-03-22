#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

VERSION="${1:-0.1.0}"
PLATFORM="${2:-$(uname -s)}"

echo "Packaging ZenClash v${VERSION} for ${PLATFORM}..."

case "$PLATFORM" in
    Linux)
        echo "Creating Linux package..."
        cargo build --release
        mkdir -p "release/zenclash-${VERSION}-linux-x86_64"
        cp "target/release/zenclash" "release/zenclash-${VERSION}-linux-x86_64/"
        tar -czf "release/zenclash-${VERSION}-linux-x86_64.tar.gz" \
            -C "release" "zenclash-${VERSION}-linux-x86_64"
        ;;
    Darwin)
        echo "Creating macOS package..."
        cargo build --release
        mkdir -p "release/zenclash-${VERSION}-macos"
        cp "target/release/zenclash" "release/zenclash-${VERSION}-macos/"
        zip -r "release/zenclash-${VERSION}-macos.zip" "release/zenclash-${VERSION}-macos"
        ;;
    *)
        echo "Unsupported platform: ${PLATFORM}"
        exit 1
        ;;
esac

echo "Packaging complete!"
