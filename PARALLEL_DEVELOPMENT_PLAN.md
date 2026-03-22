# ZenClash Parallel Development Plan

> **Version**: 1.0  
> **Date**: 2024  
> **Target**: Comprehensive parallel development strategy for Rust + GPUI rewrite  
> **Team Size**: 4-5 developers

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Module Decomposition](#2-module-decomposition)
3. [Dependency Graph](#3-dependency-graph)
4. [Interface Contracts](#4-interface-contracts)
5. [Parallel Development Waves](#5-parallel-development-waves)
6. [Team Assignment Strategy](#6-team-assignment-strategy)
7. [Risk Mitigation](#7-risk-mitigation)

---

## 1. Executive Summary

### 1.1 Bottom Line

**ZenClash** is a complete rewrite of Clash Party from Electron+React+TypeScript to Rust+GPUI+GPUI Component. The project decomposes into **26 modules** across **3 crates** (core, ui, cli), organized into **4 parallel development waves** over ~22 weeks. Each wave has clear deliverables and module boundaries that enable 4-5 developers to work simultaneously with minimal conflicts.

### 1.2 Key Numbers

| Metric             | Value          |
| ------------------ | -------------- |
| Total Modules      | 26             |
| Core Modules       | 16             |
| UI Pages           | 13             |
| Parallel Waves     | 4              |
| Estimated Duration | 22 weeks       |
| Team Size          | 4-5 developers |

---

## 2. Module Decomposition

### 2.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        zenclash-ui (GPUI Application)               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Pages (13)     │  Components (8)     │  State (4)          │   │
│  │  - Proxies      │  - Sidebar          │  - AppState         │   │
│  │  - Profiles     │  - Header           │  - ConfigState      │   │
│  │  - Connections  │  - Card             │  - ProxyState       │   │
│  │  - Logs         │  - ProxyItem        │  - TrafficState     │   │
│  │  - Rules        │  - ProfileItem      │                      │   │
│  │  - Mihomo       │  - ConnectionItem   │                      │   │
│  │  - DNS          │  - TrafficChart     │                      │   │
│  │  - Sniffer      │  - Editor           │                      │   │
│  │  - SysProxy     │  - Toast            │                      │   │
│  │  - TUN          │                      │                      │   │
│  │  - Override     │                      │                      │   │
│  │  - Resources    │                      │                      │   │
│  │  - Settings     │                      │                      │   │
│  │  - SubStore     │                      │                      │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        zenclash-core (Core Library)                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────┐ │
│  │ Config (4)   │ │ Core (4)     │ │ System (3)   │ │ Utils (5)  │ │
│  │ - AppConfig  │ │ - Manager    │ │ - SysProxy   │ │ - Logger   │ │
│  │ - MihomoCfg  │ │ - Process    │ │ - TUN        │ │ - Dirs     │ │
│  │ - ProfileCfg │ │ - ApiClient  │ │ - DNS        │ │ - Http     │ │
│  │ - OverrideCfg│ │ - Factory    │ │              │ │ - Yaml     │ │
│  │              │ │              │ │              │ │ - Template │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────┘ │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│  │ Proxy (2)    │ │ Profile (2)  │ │ Traffic (2)  │                │
│  │ - Provider   │ │ - Manager    │ │ - Monitor    │                │
│  │ - Selector   │ │ - Updater    │ │ - Chart      │                │
│  └──────────────┘ └──────────────┘ └──────────────┘                │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        zenclash-cli (CLI Tool)                      │
│  - Start/Stop Core    - Profile Management    - Status Display      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Complete Module List

#### zenclash-core (Core Library) - 16 Modules

| Module ID | Module Name        | Description                                       | Priority | Complexity |
| --------- | ------------------ | ------------------------------------------------- | -------- | ---------- |
| C01       | `config::app`      | Application configuration (IAppConfig equivalent) | P0       | Medium     |
| C02       | `config::mihomo`   | Mihomo core configuration                         | P0       | High       |
| C03       | `config::profile`  | Profile/subscription configuration                | P0       | Medium     |
| C04       | `config::override` | Override script configuration                     | P1       | Medium     |
| C05       | `core::manager`    | Core process lifecycle management                 | P0       | High       |
| C06       | `core::process`    | Process spawn/monitor/kill                        | P0       | High       |
| C07       | `core::api_client` | Mihomo REST API + WebSocket client                | P0       | High       |
| C08       | `core::factory`    | Profile generation and merging                    | P1       | High       |
| C09       | `proxy::provider`  | Proxy provider management                         | P1       | Medium     |
| C10       | `proxy::selector`  | Proxy selection and delay testing                 | P1       | Medium     |
| C11       | `profile::manager` | Subscription CRUD operations                      | P1       | Medium     |
| C12       | `profile::updater` | Auto-update scheduler                             | P2       | Low        |
| C13       | `system::sysproxy` | System proxy settings                             | P1       | High       |
| C14       | `system::tun`      | TUN mode management                               | P1       | High       |
| C15       | `system::dns`      | DNS configuration                                 | P1       | Medium     |
| C16       | `traffic::monitor` | Traffic stats via WebSocket                       | P1       | Medium     |

#### zenclash-ui (GPUI Application) - 13 Page Modules + 8 Component Modules

| Module ID | Module Name          | Description                 | Priority | Depends On     |
| --------- | -------------------- | --------------------------- | -------- | -------------- |
| U01       | `pages::proxies`     | Proxy groups and selection  | P0       | C07, C09, C10  |
| U02       | `pages::profiles`    | Subscription management     | P0       | C03, C11       |
| U03       | `pages::connections` | Active connections list     | P0       | C07            |
| U04       | `pages::logs`        | Real-time log viewer        | P1       | C07            |
| U05       | `pages::rules`       | Rule list display           | P1       | C07            |
| U06       | `pages::mihomo`      | Core settings page          | P1       | C02, C05       |
| U07       | `pages::dns`         | DNS configuration           | P1       | C15            |
| U08       | `pages::sniffer`     | Sniffer settings            | P2       | C02            |
| U09       | `pages::sysproxy`    | System proxy settings       | P1       | C13            |
| U10       | `pages::tun`         | TUN mode settings           | P1       | C14            |
| U11       | `pages::override`    | Override script management  | P2       | C04            |
| U12       | `pages::resources`   | GeoData/Provider management | P2       | C07            |
| U13       | `pages::settings`    | Application settings        | P1       | C01            |
| U14       | `pages::substore`    | Sub-Store integration       | P2       | External       |
| U15-U22   | `components::*`      | Shared UI components        | P0       | GPUI Component |

#### Shared/Utility Modules - 5 Modules

| Module ID | Module Name       | Description              | Priority |
| --------- | ----------------- | ------------------------ | -------- |
| S01       | `utils::logger`   | Tracing-based logging    | P0       |
| S02       | `utils::dirs`     | Platform-specific paths  | P0       |
| S03       | `utils::http`     | HTTP client utilities    | P0       |
| S04       | `utils::yaml`     | YAML parsing utilities   | P0       |
| S05       | `utils::template` | Default config templates | P0       |

---

## 3. Dependency Graph

### 3.1 Module Dependency Graph

```
                    ┌─────────────────┐
                    │    S01 Logger   │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│    S02 Dirs     │ │    S03 HTTP     │ │    S04 YAML     │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  C01 AppConfig  │ │ C03 ProfileCfg  │ │  C02 MihomoCfg  │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         │         ┌─────────┴─────────┐         │
         │         │                   │         │
         │         ▼                   ▼         │
         │  ┌─────────────────┐ ┌─────────────────┐
         │  │ C11 ProfileMgr  │ │ C08 Factory     │
         │  └────────┬────────┘ └────────┬────────┘
         │           │                   │
         └───────────┼───────────────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │   C05 Core Manager    │◄─────────────────┐
         └───────────┬───────────┘                  │
                     │                              │
         ┌───────────┼───────────┐                  │
         │           │           │                  │
         ▼           ▼           ▼                  │
┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │
│ C06 Process │ │ C07 ApiClnt │ │ C04 Override│    │
└─────────────┘ └──────┬──────┘ └─────────────┘    │
                       │                           │
         ┌─────────────┼─────────────┐             │
         │             │             │             │
         ▼             ▼             ▼             │
┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │
│ C09 Provider│ │ C10 Selector│ │C13 SysProxy │    │
└─────────────┘ └─────────────┘ └─────────────┘    │
         │             │                           │
         └──────┬──────┴───────────────────────────┘
                │
                ▼
    ┌───────────────────────┐
    │    UI Pages (U01-U14) │
    └───────────────────────┘
```

### 3.2 Critical Path

```
S01→S02→C01→C05→C07→U01 (Proxies Page)
                ↓
            C13→U09 (SysProxy Page)
                ↓
            C14→U10 (TUN Page)
```

**Critical Path Duration**: ~10 weeks

### 3.3 Dependency Matrix

| Module               | Depends On    | Blocks                 |
| -------------------- | ------------- | ---------------------- |
| C01 (AppConfig)      | S01, S02, S04 | C05, C13, U13          |
| C02 (MihomoConfig)   | S01, S04      | C05, C08, U06          |
| C03 (ProfileConfig)  | S01, S02, S04 | C11, U02               |
| C04 (OverrideConfig) | S01, S02, S04 | C08, U11               |
| C05 (CoreManager)    | C01-C04, C06  | ALL UI                 |
| C06 (Process)        | S01, S02      | C05                    |
| C07 (ApiClient)      | S01, S03      | C09, C10, C16, U01-U05 |
| C08 (Factory)        | C02-C04, S04  | C05                    |
| C09 (Provider)       | C07           | U01                    |
| C10 (Selector)       | C07, C09      | U01                    |
| C11 (ProfileMgr)     | C03, S03      | U02                    |
| C13 (SysProxy)       | C01, S02      | U09                    |
| C14 (TUN)            | C05, C06      | U10                    |
| C16 (Traffic)        | C07           | U01, U03               |

---

## 4. Interface Contracts

### 4.1 Core Module Interfaces

#### C01: AppConfig

```rust
// zenclash-core/src/config/app.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration (config.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub core: CoreType,
    pub enable_smart_core: bool,
    pub enable_smart_override: bool,
    pub theme: String,
    pub language: String,
    pub auto_start: bool,
    pub silent_start: bool,
    pub sys_proxy: SysProxyConfig,
    pub sider_order: Vec<String>,
    pub sider_width: u32,
    // ... (full IAppConfig fields)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreType {
    #[serde(rename = "mihomo")]
    Mihomo,
    #[serde(rename = "mihomo-alpha")]
    MihomoAlpha,
    #[serde(rename = "mihomo-smart")]
    MihomoSmart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysProxyConfig {
    pub enable: bool,
    pub host: Option<String>,
    pub mode: SysProxyMode,
    pub bypass: Vec<String>,
}

/// PUBLIC INTERFACE
impl AppConfig {
    /// Load config from disk, apply defaults, migrate if needed
    pub async fn load() -> Result<Self>;

    /// Save config to disk
    pub async fn save(&self) -> Result<()>;

    /// Patch specific fields and save
    pub async fn patch(&mut self, patch: AppConfigPatch) -> Result<()>;

    /// Get config file path
    pub fn config_path() -> PathBuf;
}

/// Partial update structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfigPatch {
    pub core: Option<CoreType>,
    pub theme: Option<String>,
    // ... optional fields for partial updates
}
```

#### C05: CoreManager

```rust
// zenclash-core/src/core/manager.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::AppConfig;
use crate::core::{Process, ApiClient, Factory};

pub struct CoreState {
    pub running: bool,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub mode: OutboundMode,
}

/// Core process manager - THE CENTRAL ORCHESTRATOR
pub struct CoreManager {
    process: Option<Process>,
    api_client: Arc<ApiClient>,
    state: Arc<RwLock<CoreState>>,
    config: Arc<RwLock<AppConfig>>,
}

/// PUBLIC INTERFACE
impl CoreManager {
    /// Create new manager instance
    pub fn new(config: Arc<RwLock<AppConfig>>) -> Self;

    /// Start mihomo core process
    /// - Generates config via Factory
    /// - Spawns process via Process
    /// - Establishes API connection
    /// - Starts traffic/logs/connections monitors
    pub async fn start(&mut self) -> Result<()>;

    /// Stop mihomo core process
    pub async fn stop(&mut self) -> Result<()>;

    /// Restart core (stop + start)
    pub async fn restart(&mut self) -> Result<()>;

    /// Get current state
    pub async fn state(&self) -> CoreState;

    /// Get API client for direct operations
    pub fn api(&self) -> Arc<ApiClient>;

    /// Check if core is running
    pub async fn is_running(&self) -> bool;
}

/// Events emitted by CoreManager
pub enum CoreEvent {
    Started { pid: u32 },
    Stopped,
    Restarted,
    Error { message: String },
    ConfigUpdated,
}
```

#### C07: ApiClient

```rust
// zenclash-core/src/core/api_client.rs

use reqwest::Client;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// Mihomo REST API + WebSocket client
pub struct ApiClient {
    http: Client,
    socket_path: PathBuf,
    base_url: String,
}

/// WebSocket subscription receivers
pub struct WebSocketStreams {
    pub traffic: mpsc::Receiver<TrafficInfo>,
    pub connections: mpsc::Receiver<ConnectionsInfo>,
    pub logs: mpsc::Receiver<LogEntry>,
    pub memory: mpsc::Receiver<MemoryInfo>,
}

/// PUBLIC INTERFACE - REST API
impl ApiClient {
    pub fn new(socket_path: PathBuf) -> Self;

    // Version
    pub async fn get_version(&self) -> Result<VersionInfo>;

    // Proxies
    pub async fn get_proxies(&self) -> Result<ProxiesResponse>;
    pub async fn get_groups(&self) -> Result<Vec<ProxyGroup>>;
    pub async fn select_proxy(&self, group: &str, proxy: &str) -> Result<()>;
    pub async fn test_delay(&self, proxy: &str, url: &str, timeout: u32) -> Result<DelayResult>;
    pub async fn test_group_delay(&self, group: &str, url: &str) -> Result<GroupDelayResult>;

    // Connections
    pub async fn get_connections(&self) -> Result<ConnectionsInfo>;
    pub async fn close_connection(&self, id: &str) -> Result<()>;
    pub async fn close_all_connections(&self) -> Result<()>;

    // Rules
    pub async fn get_rules(&self) -> Result<RulesInfo>;

    // Config
    pub async fn patch_config(&self, patch: serde_json::Value) -> Result<()>;
    pub async fn upgrade_config(&self, path: &str) -> Result<()>;

    // Providers
    pub async fn get_proxy_providers(&self) -> Result<ProxyProviders>;
    pub async fn update_proxy_provider(&self, name: &str) -> Result<()>;
    pub async fn get_rule_providers(&self) -> Result<RuleProviders>;
    pub async fn update_rule_provider(&self, name: &str) -> Result<()>;

    // GeoData
    pub async fn upgrade_geo(&self) -> Result<()>;

    // Smart Core
    pub async fn get_smart_weights(&self, group: &str) -> Result<HashMap<String, f32>>;
    pub async fn flush_smart_cache(&self, config: Option<&str>) -> Result<()>;
}

/// PUBLIC INTERFACE - WebSocket Streams
impl ApiClient {
    /// Start WebSocket subscriptions, returns receivers
    pub async fn subscribe_all(&self) -> Result<WebSocketStreams>;

    /// Start individual subscription
    pub async fn subscribe_traffic(&self) -> Result<mpsc::Receiver<TrafficInfo>>;
    pub async fn subscribe_connections(&self) -> Result<mpsc::Receiver<ConnectionsInfo>>;
    pub async fn subscribe_logs(&self, level: LogLevel) -> Result<mpsc::Receiver<LogEntry>>;
    pub async fn subscribe_memory(&self) -> Result<mpsc::Receiver<MemoryInfo>>;
}

/// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficInfo {
    pub up: u64,
    pub down: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub now: String,
    pub all: Vec<ProxyItem>,
    pub history: Vec<HistoryEntry>,
    pub test_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyItem {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub alive: bool,
    pub history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub time: String,
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDetail {
    pub id: String,
    pub metadata: ConnectionMetadata,
    pub upload: u64,
    pub download: u64,
    pub start: String,
    pub chains: Vec<String>,
    pub rule: String,
}

// ... more structures
```

#### C13: SysProxy

```rust
// zenclash-core/src/system/sysproxy.rs

use crate::config::SysProxyConfig;

/// System proxy manager
pub struct SysProxyManager {
    current_state: SysProxyState,
    pac_server: Option<PacServer>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SysProxyState {
    Disabled,
    Manual,
    Auto,
}

/// PUBLIC INTERFACE
impl SysProxyManager {
    pub fn new() -> Self;

    /// Enable system proxy with mode
    pub async fn enable(&mut self, config: &SysProxyConfig, port: u16) -> Result<()>;

    /// Disable system proxy
    pub async fn disable(&mut self) -> Result<()>;

    /// Get current state
    pub fn state(&self) -> SysProxyState;

    /// Check if system proxy is enabled
    pub fn is_enabled(&self) -> bool;
}

/// Platform-specific implementations
#[cfg(target_os = "macos")]
impl SysProxyManager {
    /// macOS uses helper tool for privilege escalation
    async fn enable_macos(&mut self, config: &SysProxyConfig, port: u16) -> Result<()>;
    async fn disable_macos(&mut self) -> Result<()>;
}

#[cfg(target_os = "linux")]
impl SysProxyManager {
    /// Linux uses gsettings/dconf
    async fn enable_linux(&mut self, config: &SysProxyConfig, port: u16) -> Result<()>;
    async fn disable_linux(&mut self) -> Result<()>;
}

#[cfg(target_os = "windows")]
impl SysProxyManager {
    /// Windows uses registry
    async fn enable_windows(&mut self, config: &SysProxyConfig, port: u16) -> Result<()>;
    async fn disable_windows(&mut self) -> Result<()>;
}
```

### 4.2 UI Module Interfaces

#### U01: Proxies Page

```rust
// zenclash-ui/src/pages/proxies.rs

use gpui::*;
use gpui_component::*;
use zenclash_core::{ApiClient, ProxyGroup, ProxyItem};

pub struct ProxiesPage {
    // State
    groups: Vec<ProxyGroup>,
    selected_group: Option<usize>,
    search_query: String,
    sort_mode: SortMode,
    display_mode: DisplayMode,

    // Loading states
    loading: bool,
    delaying_groups: HashSet<String>,
    delaying_proxies: HashSet<String>,

    // Core reference
    api: Arc<ApiClient>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Default,
    Delay,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode {
    Simple,
    Full,
}

/// PUBLIC INTERFACE
impl ProxiesPage {
    pub fn new(api: Arc<ApiClient>) -> Self;

    /// Refresh proxy groups from API
    pub async fn refresh(&mut self, cx: &mut Context<Self>);

    /// Select a proxy in a group
    pub async fn select_proxy(&mut self, group: &str, proxy: &str);

    /// Test delay for single proxy
    pub async fn test_proxy_delay(&mut self, proxy: &str);

    /// Test delay for entire group
    pub async fn test_group_delay(&mut self, group: &str);

    /// Toggle group expansion
    pub fn toggle_group(&mut self, group_idx: usize);

    /// Set search filter
    pub fn set_search(&mut self, query: String);

    /// Set sort mode
    pub fn set_sort_mode(&mut self, mode: SortMode);
}

impl Render for ProxiesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // VirtualList-based rendering for 1000+ proxies
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.render_header(cx))
            .child(self.render_grouped_list(cx))
    }
}
```

#### Application State

```rust
// zenclash-ui/src/state/app_state.rs

use gpui::*;
use zenclash_core::{CoreManager, AppConfig, ApiClient};

/// Global application state
pub struct AppState {
    pub core_manager: Arc<RwLock<CoreManager>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub current_page: Page,
    pub sidebar_collapsed: bool,
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    Proxies,
    Profiles,
    Connections,
    Logs,
    Rules,
    Mihomo,
    Dns,
    Sniffer,
    SysProxy,
    Tun,
    Override,
    Resources,
    Settings,
    SubStore,
}

/// PUBLIC INTERFACE - Used by all pages
impl AppState {
    pub fn new(core_manager: CoreManager, config: AppConfig) -> Self;

    /// Navigate to page
    pub fn navigate(&mut self, page: Page, cx: &mut Context<Self>);

    /// Get current API client
    pub fn api(&self) -> Arc<ApiClient>;

    /// Update config and persist
    pub async fn patch_config(&mut self, patch: AppConfigPatch) -> Result<()>;

    /// Toggle sidebar
    pub fn toggle_sidebar(&mut self);

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme);
}
```

---

## 5. Parallel Development Waves

### 5.1 Wave Overview

```
Week:  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15  16  17  18  19  20  21  22
       ├───────────────────┼───────────────────┼───────────────────┼───────────────────┤
WAVE 1 │███████████████████│                   │                   │                   │
       │  Foundation       │                   │                   │                   │
       │  (Weeks 1-4)      │                   │                   │                   │
       ├───────────────────┼───────────────────┤                   │                   │
WAVE 2 │                   │███████████████████│                   │                   │
       │                   │  Core Services    │                   │                   │
       │                   │  (Weeks 5-9)      │                   │                   │
       ├───────────────────┼───────────────────┼───────────────────┤                   │
WAVE 3 │                   │                   │███████████████████│                   │
       │                   │                   │  UI Development   │                   │
       │                   │                   │  (Weeks 10-15)    │                   │
       ├───────────────────┼───────────────────┼───────────────────┼───────────────────┤
WAVE 4 │                   │                   │                   │███████████████████│
       │                   │                   │                   │  Integration      │
       │                   │                   │                   │  (Weeks 16-22)    │
       └───────────────────┴───────────────────┴───────────────────┴───────────────────┘
```

### 5.2 Wave 1: Foundation (Weeks 1-4)

**Goal**: Establish core infrastructure, all modules can start in parallel

#### Parallel Tracks (All Independent)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              WAVE 1 PARALLEL TRACKS                          │
├─────────────────┬─────────────────┬─────────────────┬───────────────────────┤
│    TRACK A      │    TRACK B      │    TRACK C      │      TRACK D          │
│    Config       │    Utils        │    Process      │      UI Scaffold      │
│    (Dev 1)      │    (Dev 2)      │    (Dev 3)      │      (Dev 4)          │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ C01 AppConfig   │ S01 Logger      │ C06 Process     │ UI Project Setup      │
│ C02 MihomoCfg   │ S02 Dirs        │ Process spawn   │ GPUI Component init   │
│ C03 ProfileCfg  │ S03 HTTP        │ Process monitor │ App scaffold          │
│ C04 OverrideCfg │ S04 YAML        │ Signal handling │ Sidebar skeleton      │
│                 │ S05 Template    │ IPC path mgmt   │ Page routing          │
│                 │                 │                 │ Theme system          │
├─────────────────┴─────────────────┴─────────────────┴───────────────────────┤
│                        DELIVERABLES (End of Week 4)                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✓ config.yaml parsing/saving                                                 │
│ ✓ Platform-specific directory management                                     │
│ ✓ Logging infrastructure                                                     │
│ ✓ Process spawn/kill (standalone test)                                       │
│ ✓ UI window appears with sidebar                                             │
│ ✓ Theme switching works                                                      │
│ ✓ All 4 config modules compile and pass tests                                │
│ ✓ All 5 utility modules compile and pass tests                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Track Assignments

| Developer | Track       | Modules       | Key Deliverables               |
| --------- | ----------- | ------------- | ------------------------------ |
| **Dev 1** | Track A     | C01-C04       | Config system fully functional |
| **Dev 2** | Track B     | S01-S05       | Utils ready for core usage     |
| **Dev 3** | Track C     | C06           | Process management tested      |
| **Dev 4** | Track D     | UI Scaffold   | App runs, navigates, themes    |
| **Dev 5** | Integration | Cross-cutting | CI/CD, tests, docs             |

#### Week-by-Week Breakdown

**Week 1-2: Setup + Basic Structure**

- Dev 1: Project setup, cargo workspace, C01 skeleton
- Dev 2: S01 logger, S02 dirs, S04 yaml
- Dev 3: C06 process skeleton, research tokio::process
- Dev 4: UI crate setup, GPUI hello world
- Dev 5: CI/CD pipeline, dev environment docs

**Week 3-4: Implementation**

- Dev 1: C01-C04 full implementation, tests
- Dev 2: S03 http client, S05 templates, integration tests
- Dev 3: C06 full impl, platform-specific (macOS first)
- Dev 4: Sidebar, routing, theme, i18n skeleton
- Dev 5: Integration tests, code review

### 5.3 Wave 2: Core Services (Weeks 5-9)

**Goal**: Complete all core functionality, ready for UI consumption

#### Parallel Tracks

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              WAVE 2 PARALLEL TRACKS                          │
├─────────────────┬─────────────────┬─────────────────┬───────────────────────┤
│    TRACK A      │    TRACK B      │    TRACK C      │      TRACK D          │
│    Core API     │    Profile      │    System       │      UI Components    │
│    (Dev 1)      │    (Dev 2)      │    (Dev 3)      │      (Dev 4)          │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ C07 ApiClient   │ C11 ProfileMgr  │ C13 SysProxy    │ ProxyItem component   │
│ C05 CoreManager │ C12 ProfileUpd  │ C14 TUN         │ ProfileItem component │
│ C08 Factory     │ C09 Provider    │ C15 DNS         │ ConnectionItem comp   │
│ C16 Traffic     │ C10 Selector    │ Permission mgmt │ TrafficChart comp     │
│                 │                 │                 │ Editor component      │
│                 │                 │                 │ Toast system          │
├─────────────────┴─────────────────┴─────────────────┴───────────────────────┤
│                        DELIVERABLES (End of Week 9)                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✓ CoreManager.start() spawns mihomo successfully                            │
│ ✓ ApiClient can call all mihomo APIs                                        │
│ ✓ WebSocket streams work (traffic, logs, connections)                       │
│ ✓ Profile create/update/delete works                                        │
│ ✓ System proxy enable/disable works (macOS)                                 │
│ ✓ TUN mode can be enabled (with permissions)                                │
│ ✓ All core modules compile, test coverage > 70%                             │
│ ✓ UI components ready for page integration                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Track Assignments

| Developer | Track       | Modules               | Key Deliverables         |
| --------- | ----------- | --------------------- | ------------------------ |
| **Dev 1** | Track A     | C05, C07, C08, C16    | Core fully functional    |
| **Dev 2** | Track B     | C09-C12               | Profile/Proxy management |
| **Dev 3** | Track C     | C13-C15 + Permissions | System integration       |
| **Dev 4** | Track D     | Components            | All shared components    |
| **Dev 5** | Integration | Tests, docs           | Core integration tests   |

#### Dependency Flow Within Wave 2

```
Week 5-6: Independent Work
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│C07 API   │  │C11 Prof  │  │C13 SysPxy│  │Components│
│Client    │  │Manager   │  │          │  │          │
└────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┘
     │             │             │
Week 7: Integration Points
     │             │             │
     ▼             ▼             ▼
┌──────────┐  ┌──────────┐  ┌──────────┐
│C05 Core  │  │C09/10    │  │C14 TUN   │
│Manager   │  │Proxy     │  │(needs C05)│
└────┬─────┘  └──────────┘  └──────────┘
     │
Week 8-9: Complete
     ▼
┌──────────┐
│C08 Fact  │
│C16 Traffic│
└──────────┘
```

### 5.4 Wave 3: UI Development (Weeks 10-15)

**Goal**: All pages functional, connected to core

#### Parallel Tracks

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              WAVE 3 PARALLEL TRACKS                          │
├─────────────────┬─────────────────┬─────────────────┬───────────────────────┤
│    TRACK A      │    TRACK B      │    TRACK C      │      TRACK D          │
│    Primary UI   │    Secondary UI │    Settings UI  │      System UI        │
│    (Dev 1)      │    (Dev 2)      │    (Dev 3)      │      (Dev 4)          │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ U01 Proxies     │ U04 Logs        │ U06 Mihomo      │ U09 SysProxy          │
│ U02 Profiles    │ U05 Rules       │ U07 DNS         │ U10 TUN               │
│ U03 Connections │ U11 Override    │ U08 Sniffer     │ U13 Settings          │
│                 │ U12 Resources   │                 │ U14 SubStore          │
├─────────────────┴─────────────────┴─────────────────┴───────────────────────┤
│                        DELIVERABLES (End of Week 15)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✓ All pages render correctly                                                 │
│ ✓ Proxies page: select, delay test, virtual list for 1000+ nodes            │
│ ✓ Profiles page: add, edit, delete, update subscriptions                    │
│ ✓ Connections page: real-time update, close connections                     │
│ ✓ Settings pages: all options functional                                     │
│ ✓ System pages: sysproxy/TUN toggles work                                    │
│ ✓ Theme switching works across all pages                                     │
│ ✓ i18n works (zh-CN, en-US)                                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Page Dependencies Within Wave

```
Week 10-11: Core Pages (Depends on Wave 2 completion)
┌──────────────────────────────────────────────┐
│  U01 Proxies  →  U02 Profiles  →  U03 Conn   │
│     (Dev 1)        (Dev 1)         (Dev 1)   │
└──────────────────────────────────────────────┘

Week 12-13: Secondary Pages
┌──────────────────────────────────────────────┐
│  U04 Logs  →  U05 Rules  →  U11 Override     │
│    (Dev 2)      (Dev 2)        (Dev 2)       │
└──────────────────────────────────────────────┘

Week 14-15: Settings & System Pages
┌──────────────────────────────────────────────┐
│  U06 Mihomo  →  U07 DNS  →  U08 Sniffer      │
│    (Dev 3)       (Dev 3)       (Dev 3)       │
└──────────────────────────────────────────────┘
┌──────────────────────────────────────────────┐
│  U09 SysProxy  →  U10 TUN  →  U13 Settings   │
│     (Dev 4)        (Dev 4)       (Dev 4)     │
└──────────────────────────────────────────────┘
```

### 5.5 Wave 4: Integration & Polish (Weeks 16-22)

**Goal**: Production-ready application

#### Parallel Tracks

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              WAVE 4 PARALLEL TRACKS                          │
├─────────────────┬─────────────────┬─────────────────┬───────────────────────┤
│    TRACK A      │    TRACK B      │    TRACK C      │      TRACK D          │
│    System Integ │    Polish       │    Platform     │      Release          │
│    (Dev 1)      │    (Dev 2)      │    (Dev 3)      │      (Dev 4)          │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ Tray icon       │ Performance opt │ macOS specific  │ Packaging             │
│ Shortcuts       │ Memory opt      │ entitlements    │ Code signing          │
│ Auto-update     │ Rendering opt   │ helper tool     │ Notarization          │
│ Auto-start      │ Accessibility   │ Linux support   │ Distribution          │
│ Floating window │ Error handling  │ desktop entry   │ Documentation         │
│ Backup (WebDAV) │ i18n complete   │                 │ Release notes         │
├─────────────────┴─────────────────┴─────────────────┴───────────────────────┤
│                        DELIVERABLES (End of Week 22)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✓ System tray with full menu                                                │
│ ✓ Global shortcuts configurable                                             │
│ ✓ Auto-update works                                                         │
│ ✓ Auto-start on login                                                       │
│ ✓ Floating window (optional)                                                │
│ ✓ WebDAV backup/restore                                                     │
│ ✓ Memory < 100MB                                                            │
│ ✓ Startup < 1s                                                              │
│ ✓ macOS: signed, notarized, helper tool                                     │
│ ✓ Linux: deb, rpm, AppImage                                                 │
│ ✓ User documentation complete                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Team Assignment Strategy

### 6.1 Role Definitions

| Role                      | Primary Skills       | Secondary Skills | Focus Areas           |
| ------------------------- | -------------------- | ---------------- | --------------------- |
| **Rust Core Engineer**    | Rust, Tokio, Systems | HTTP, WebSocket  | Core, Process, API    |
| **Rust Systems Engineer** | Rust, Platform APIs  | Networking       | SysProxy, TUN, DNS    |
| **GPUI Developer**        | Rust, GPUI           | UI/UX            | Pages, Components     |
| **Full-Stack Engineer**   | Rust, Testing        | CI/CD            | Integration, QA       |
| **Tech Lead**             | All of the above     | Architecture     | Coordination, Reviews |

### 6.2 Assignment Matrix

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEVELOPER ASSIGNMENT BY WAVE                          │
├─────────────────┬─────────────────┬─────────────────┬───────────────────────┤
│                 │     WAVE 1      │     WAVE 2      │      WAVE 3-4         │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ Dev 1 (Core)    │ Config System   │ Core/Api/Traffic│ System Integration    │
│                 │ C01-C04         │ C05, C07, C08   │ Tray, Shortcuts       │
│                 │                 │ C16             │ Auto-update           │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ Dev 2 (Core)    │ Utils           │ Profile/Proxy   │ Polish & Perf         │
│                 │ S01-S05         │ C09-C12         │ Optimization          │
│                 │                 │                 │ Memory, Rendering     │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ Dev 3 (Sys)     │ Process         │ System          │ Platform              │
│                 │ C06             │ C13-C15         │ macOS/Linux           │
│                 │                 │ Permissions     │ Packaging             │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ Dev 4 (UI)      │ UI Scaffold     │ Components      │ Pages                 │
│                 │ App, Routing    │ All shared      │ U01-U14               │
│                 │ Theme           │                 │ Settings, System      │
├─────────────────┼─────────────────┼─────────────────┼───────────────────────┤
│ Dev 5 (Lead)    │ CI/CD, Docs     │ Integration     │ Release               │
│                 │ Architecture    │ Tests           │ Code signing          │
│                 │ Reviews         │ Cross-cutting   │ Distribution          │
└─────────────────┴─────────────────┴─────────────────┴───────────────────────┘
```

### 6.3 Module Ownership

| Developer | Owns (Primary)   | Owns (Secondary) | Reviews          |
| --------- | ---------------- | ---------------- | ---------------- |
| **Dev 1** | C01-C08, C16     | -                | Dev 2, Dev 3 PRs |
| **Dev 2** | S01-S05, C09-C12 | -                | Dev 1, Dev 4 PRs |
| **Dev 3** | C06, C13-C15     | Platform code    | Dev 1 PRs        |
| **Dev 4** | All UI           | Theme, i18n      | UI-related PRs   |
| **Dev 5** | CI/CD, Docs      | Cross-cutting    | All PRs          |

### 6.4 Communication Protocol

```
Daily Standups (15 min):
├── What I completed yesterday
├── What I'm working on today
└── Blockers (if any)

Weekly Planning (1 hour):
├── Wave progress review
├── Next week priorities
├── Dependency resolution
└── Risk assessment

Bi-weekly Architecture Review (2 hours):
├── Interface changes
├── Performance review
├── Code quality review
└── Technical debt tracking

Async Communication:
├── GitHub Issues for tasks
├── PRs for code review
├── Slack/Discord for quick questions
└── Wiki for documentation
```

---

## 7. Risk Mitigation

### 7.1 Technical Risks

| Risk                                     | Probability | Impact | Mitigation                                             |
| ---------------------------------------- | ----------- | ------ | ------------------------------------------------------ |
| GPUI API changes                         | Medium      | High   | Pin version, monitor changelog, allocate buffer time   |
| Platform differences (macOS vs Linux)    | High        | Medium | Test early on both, abstract platform code             |
| TUN permission issues                    | High        | High   | Research early, have fallback (no-TUN mode)            |
| WebSocket reconnection bugs              | Medium      | Medium | Implement robust retry logic, use battle-tested crates |
| Memory leaks in long-running app         | Medium      | Medium | Regular profiling, use `tracing` for allocations       |
| Performance on large proxy lists (1000+) | Medium      | High   | Use virtual list from start, benchmark early           |

### 7.2 Project Risks

| Risk                         | Probability | Impact | Mitigation                                      |
| ---------------------------- | ----------- | ------ | ----------------------------------------------- |
| Developer unavailable        | Medium      | Medium | Cross-training, documentation, pair programming |
| Scope creep                  | High        | High   | Strict feature freeze after Wave 2, MVP mindset |
| Integration surprises        | Medium      | High   | Integration tests from Week 1, mock APIs        |
| Missing features vs original | Medium      | Medium | Feature checklist, user testing, prioritize     |

### 7.3 Contingency Plans

**If GPUI Windows support is delayed**:

- Focus on macOS/Linux release
- Plan Windows as Phase 3 (already in plan)

**If TUN permissions too complex**:

- Release without TUN initially
- Add TUN in update

**If performance issues**:

- Profile and optimize in Wave 4
- Consider reducing features

**If team member leaves**:

- Dev 5 (Lead) can cover any track
- Documentation enables quick onboarding

---

## 8. Quick Reference

### 8.1 Module Checklist

```
WAVE 1 (Weeks 1-4): Foundation
□ S01 Logger      □ S02 Dirs        □ S03 HTTP       □ S04 YAML
□ S05 Template    □ C01 AppConfig   □ C02 MihomoCfg  □ C03 ProfileCfg
□ C04 OverrideCfg □ C06 Process     □ UI Scaffold

WAVE 2 (Weeks 5-9): Core Services
□ C05 CoreManager □ C07 ApiClient   □ C08 Factory    □ C09 Provider
□ C10 Selector    □ C11 ProfileMgr  □ C12 ProfileUpd □ C13 SysProxy
□ C14 TUN         □ C15 DNS         □ C16 Traffic    □ Permissions

WAVE 3 (Weeks 10-15): UI Development
□ U01 Proxies     □ U02 Profiles    □ U03 Connections
□ U04 Logs        □ U05 Rules       □ U06 Mihomo
□ U07 DNS         □ U08 Sniffer     □ U09 SysProxy
□ U10 TUN         □ U11 Override    □ U12 Resources
□ U13 Settings    □ U14 SubStore    □ Components

WAVE 4 (Weeks 16-22): Integration
□ Tray            □ Shortcuts       □ Auto-update
□ Auto-start      □ Floating window □ WebDAV backup
□ Performance     □ Platform (macOS/Linux)
□ Packaging       □ Code signing    □ Documentation
```

### 8.2 Key Metrics

| Metric                  | Target           |
| ----------------------- | ---------------- |
| Startup time            | < 1 second       |
| Memory usage            | < 100 MB         |
| Binary size (release)   | ~12 MB           |
| Test coverage           | > 70%            |
| Proxy list (1000 nodes) | Smooth 60fps     |
| Log entries (100k)      | Smooth scrolling |

### 8.3 Success Criteria

- [ ] All Wave 1 modules compile and pass tests
- [ ] CoreManager can start/stop mihomo
- [ ] ApiClient successfully calls all mihomo APIs
- [ ] All UI pages render and function correctly
- [ ] System proxy toggle works (macOS)
- [ ] TUN mode works (with permissions)
- [ ] Application passes performance benchmarks
- [ ] macOS app is signed and notarized
- [ ] Linux packages are created (deb, rpm, AppImage)
- [ ] User documentation is complete

---

**Document Version**: 1.0  
**Last Updated**: 2024  
**Author**: ZenClash Team
