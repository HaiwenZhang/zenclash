#!/bin/bash
set -e

PLATFORM="$(uname -s)"

echo "Installing system dependencies for ${PLATFORM}..."

case "$PLATFORM" in
    Linux)
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                pkg-config \
                libssl-dev \
                libudev-dev \
                libasound2-dev
        elif command -v yum &> /dev/null; then
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y \
                pkgconfig \
                openssl-devel \
                udev-devel \
                alsa-lib-devel
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --needed \
                base-devel \
                pkgconf \
                openssl \
                libudev \
                alsa-lib
        fi
        ;;
    Darwin)
        if command -v brew &> /dev/null; then
            brew install pkg-config openssl
        fi
        ;;
    *)
        echo "Unsupported platform: ${PLATFORM}"
        exit 1
        ;;
esac

echo "Dependencies installed!"
