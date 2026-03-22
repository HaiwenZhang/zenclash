# ZenClash - 完整项目文档

## 项目概述

ZenClash 是 Clash Party 的 Rust + GPUI 重写版本，提供高性能、低内存占用的代理管理体验。

## 完成状态

### Phase 0-5: 已完成 ✅

| Phase   | 内容                         | 状态    |
| ------- | ---------------------------- | ------- |
| Phase 0 | 多 Agent 并行开发框架        | ✅ 完成 |
| Phase 1 | 代码库结构分析               | ✅ 完成 |
| Phase 2 | 技术预研 (GPUI, Rust 生态)   | ✅ 完成 |
| Phase 3 | 详细开发计划                 | ✅ 完成 |
| Phase 4 | 核心模块开发 (zenclash-core) | ✅ 完成 |
| Phase 5 | UI 模块开发 (zenclash-ui)    | ✅ 完成 |
| Phase 6 | 集成测试与验证               | ✅ 完成 |

## 代码统计

- **总文件数**: 41 个 Rust 源文件
- **核心模块**: 28 个文件 (zenclash-core)
- **UI 模块**: 12 个文件 (zenclash-ui)
- **CLI 模块**: 1 个文件 (zenclash-cli)
- **集成测试**: 1 个文件
- **性能测试**: 1 个文件

## 项目结构

```
zenclash/
├── Cargo.toml                    - 工作区配置
├── Cargo.lock                    - 依赖锁定
├── Makefile                     - 构建脚本
├── .github/workflows/ci.yml      - CI配置
├── README.md                     - 项目说明
├── PARALLEL_DEVELOPMENT_PLAN.md  - 详细开发计划
├── INTERFACE_DEFINITIONS.rs      - 接口定义
├── crates/
│   ├── zenclash-core/           - 核心逻辑库
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   ├── utils/           - dirs, logger, http
│   │   │   ├── config/          - app, mihomo, profile, override
│   │   │   ├── core/            - manager, api, process
│   │   │   ├── proxy/           - proxy, selector, delay_test
│   │   │   ├── system/          - sysproxy, tun, dns
│   │   │   └── traffic/         - monitor
│   │   └── Cargo.toml
│   ├── zenclash-ui/             - GUI应用
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── app.rs
│   │   │   ├── window.rs
│   │   │   ├── components/      - sidebar
│   │   │   └── pages/           - proxies, profiles, connections, logs, settings
│   │   └── Cargo.toml
│   └── zenclash-cli/            - CLI工具
│       ├── src/main.rs
│       └── Cargo.toml
├── tests/
│   └── integration_test.rs      - 集成测试
├── benches/
│   └── benchmark.rs             - 性能测试
└── docs/
    └── ARCHITECTURE.md          - 架构文档
```

## 核心功能

### 配置管理

- AppConfig: 应用配置 (主题、语言、自动启动等)
- MihomoConfig: Mihomo 核心配置 (端口、TUN、DNS 等)
- ProfileConfig: 订阅配置管理
- OverrideConfig: 覆写规则配置

### 核心控制

- CoreManager: 管理 mihomo 进程生命周期
- Process: 进程管理 (启动/停止/监控)
- ApiClient: HTTP API 客户端 (通过 Unix Socket/Named Pipe)

### 代理管理

- ProxyGroup: 代理组管理
- ProxySelector: 代理选择器 (支持多种策略)
- DelayTester: 延迟测试

### 系统集成

- SysProxyManager: 系统代理设置 (跨平台)
- TunManager: TUN 设备管理
- DnsManager: DNS 配置管理

### 流量监控

- TrafficMonitor: 实时流量统计

## 技术栈

- **UI 框架**: GPUI 0.2.2 + GPUI Component 0.5.1
- **异步运行时**: Tokio 1.40
- **HTTP 客户端**: reqwest 0.12
- **WebSocket**: tokio-tungstenite 0.24
- **序列化**: serde + serde_yaml + serde_json
- **错误处理**: anyhow + thiserror
- **日志**: tracing + tracing-subscriber

## 构建说明

由于 GPUI 依赖 Metal SDK (macOS 图形框架)，需要 macOS + Xcode 环境才能编译。

```bash
# 安装依赖
make install-deps

# 构建调试版本
make build

# 构建发布版本
make build-release

# 运行测试
make test

# 运行性能测试
cargo bench

# 格式化代码
make fmt

# 运行代码检查
make clippy
```

## 跨平台支持

- **macOS**: 完整支持 (需要 Xcode)
- **Linux**: 支持 (需安装开发依赖)
- **Windows**: 等待 GPUI Windows 支持完善

## 性能优化

### 已实现

- 虚拟滚动: 支持大数据量代理列表
- 异步 I/O: 所有网络操作异步化
- 零拷贝: 使用 Rust 所有权系统避免拷贝

### 预期性能提升

- 启动时间: 5x (Electron 3-5s → GPUI <1s)
- 内存占用: 4x (Electron 200-500MB → GPUI 50-100MB)
- 包大小: 8x (Electron 100MB+ → GPUI ~12MB)

## 许可证

MIT License

## 致谢

- GPUI 框架由 Zed Industries 开发
- GPUI Component 由 Longbridge 开发
- 原 Clash Party 项目团队
