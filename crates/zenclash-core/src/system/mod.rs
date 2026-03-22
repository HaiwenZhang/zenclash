//! System integration modules for ZenClash
//!
//! This module provides cross-platform system integration functionality including:
//! - System proxy management
//! - TUN device management
//! - DNS configuration

mod dns;
mod sysproxy;
mod tun;

pub use dns::{DnsError, DnsManager};
pub use sysproxy::{ProxyConfig, ProxyType, SysProxyError, SysProxyManager};
pub use tun::{TunConfig, TunDevice, TunError, TunManager};
