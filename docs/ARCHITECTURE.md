# ZenClash Architecture

## Overview

ZenClash is a Mihomo GUI built with Rust and GPUI.

## Crates

### zenclash-core

Core functionality:

- Configuration management
- Proxy handling
- Rule matching

### zenclash-ui

GPUI-based UI:

- Window management
- Component library
- Theme system

### zenclash-cli

Command-line interface and application entry point.

## Data Flow

```
User Input → UI Layer → Core Logic → Mihomo Core
                ↑              ↓
            Configuration  ←─────┘
```

## Platform Support

- macOS (primary)
- Linux
