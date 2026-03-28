use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, App, AppContext, Context, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    switch::Switch,
    v_flex, ActiveTheme, Disableable, Sizable,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::pages::PageTrait;
use zenclash_core::prelude::{AppConfig, CoreManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CoreType {
    #[default]
    Mihomo,
    MihomoAlpha,
    MihomoSmart,
}

impl CoreType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoreType::Mihomo => "mihomo",
            CoreType::MihomoAlpha => "mihomo-alpha",
            CoreType::MihomoSmart => "mihomo-smart",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "mihomo-alpha" => CoreType::MihomoAlpha,
            "mihomo-smart" => CoreType::MihomoSmart,
            _ => CoreType::Mihomo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Warning,
    Error,
    Silent,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Silent => "silent",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "debug" => LogLevel::Debug,
            "warning" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "silent" => LogLevel::Silent,
            _ => LogLevel::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MihomoSettings {
    pub core_type: CoreType,
    pub mixed_port: u16,
    pub socks_port: u16,
    pub http_port: u16,
    pub redir_port: u16,
    pub tproxy_port: u16,
    pub external_controller: String,
    pub secret: String,
    pub allow_lan: bool,
    pub ipv6: bool,
    pub log_level: LogLevel,
    pub find_process_mode: String,
    pub unified_delay: bool,
    pub tcp_concurrent: bool,
    pub store_selected: bool,
    pub store_fake_ip: bool,
}

impl Default for MihomoSettings {
    fn default() -> Self {
        Self {
            core_type: CoreType::default(),
            mixed_port: 7890,
            socks_port: 0,
            http_port: 0,
            redir_port: 0,
            tproxy_port: 0,
            external_controller: "127.0.0.1:9090".into(),
            secret: String::new(),
            allow_lan: false,
            ipv6: false,
            log_level: LogLevel::default(),
            find_process_mode: "strict".into(),
            unified_delay: false,
            tcp_concurrent: false,
            store_selected: false,
            store_fake_ip: false,
        }
    }
}

impl MihomoSettings {
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            core_type: CoreType::from_str(&config.core),
            mixed_port: config.mixed_port.unwrap_or(7890),
            socks_port: config.socks_port.unwrap_or(0),
            http_port: 0,
            redir_port: config.redir_port.unwrap_or(0),
            tproxy_port: config.tproxy_port.unwrap_or(0),
            external_controller: config.external_controller.clone().unwrap_or_else(|| "127.0.0.1:9090".into()),
            secret: config.secret.clone().unwrap_or_default(),
            allow_lan: config.allow_lan,
            ipv6: config.ipv6,
            log_level: LogLevel::from_str(&config.log_level),
            find_process_mode: config.find_process_mode.clone().unwrap_or_else(|| "strict".into()),
            unified_delay: config.unified_delay,
            tcp_concurrent: config.tcp_concurrent,
            store_selected: false,
            store_fake_ip: false,
        }
    }

    pub fn to_app_config_patch(&self) -> zenclash_core::prelude::AppConfigPatch {
        zenclash_core::prelude::AppConfigPatch {
            core: Some(self.core_type.as_str().to_string()),
            mixed_port: Some(self.mixed_port),
            socks_port: if self.socks_port > 0 { Some(self.socks_port) } else { None },
            redir_port: if self.redir_port > 0 { Some(self.redir_port) } else { None },
            tproxy_port: if self.tproxy_port > 0 { Some(self.tproxy_port) } else { None },
            external_controller: Some(self.external_controller.clone()),
            secret: if self.secret.is_empty() { None } else { Some(self.secret.clone()) },
            allow_lan: Some(self.allow_lan),
            ipv6: Some(self.ipv6),
            log_level: Some(self.log_level.as_str().to_string()),
            unified_delay: Some(self.unified_delay),
            tcp_concurrent: Some(self.tcp_concurrent),
            find_process_mode: Some(self.find_process_mode.clone()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCoreSettings {
    pub enabled: bool,
    pub use_lightgbm: bool,
    pub collect_data: bool,
    pub collector_size: u32,
    pub strategy: SmartCoreStrategy,
}

impl Default for SmartCoreSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            use_lightgbm: false,
            collect_data: false,
            collector_size: 100,
            strategy: SmartCoreStrategy::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SmartCoreStrategy {
    #[default]
    StickySessions,
    RoundRobin,
}

pub struct MihomoPage {
    core_manager: Arc<RwLock<CoreManager>>,
    settings: Entity<MihomoSettings>,
    smart_settings: Entity<SmartCoreSettings>,
    core_version: Option<String>,
    is_upgrading: bool,
}

impl MihomoPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, cx: &mut Context<Self>) -> Self {
        let settings = AppConfig::load().ok();
        let mihomo_settings = settings
            .map(|s| MihomoSettings::from_app_config(&s))
            .unwrap_or_default();

        Self {
            core_manager,
            settings: cx.new(|_| mihomo_settings),
            smart_settings: cx.new(|_| SmartCoreSettings::default()),
            core_version: None,
            is_upgrading: false,
        }
    }

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let settings = self.settings.read(cx).clone();
        let mut config = AppConfig::load().unwrap_or_default();
        let patch = settings.to_app_config_patch();
        patch.apply(&mut config);
        if config.save().is_ok() {
            let core_manager = self.core_manager.clone();
            cx.spawn(async move |_, _| {
                let manager = core_manager.read();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        manager.restart().await.ok();
                    })
                });
            })
            .detach();
        }
        cx.notify();
    }

    fn render_ports_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);
        let socks_port_str = if settings.socks_port == 0 { "Disabled".to_string() } else { settings.socks_port.to_string() };
        let http_port_str = if settings.http_port == 0 { "Disabled".to_string() } else { settings.http_port.to_string() };

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.foreground)
                    .child("Port Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Mixed Port"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(settings.mixed_port.to_string()),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("SOCKS Port"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(socks_port_str),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("HTTP Port"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(http_port_str),
                    ),
            )
    }

    fn render_controller_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("External Controller"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Controller Address"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(div().child(settings.external_controller.clone())),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Secret"))
                    .child(if settings.secret.is_empty() {
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("Not set")
                    } else {
                        div().text_sm().child("••••••••")
                    }),
            )
    }

    fn render_core_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);
        let core_type_str = settings.core_type.as_str().to_string();
        let log_level_str = settings.log_level.as_str().to_string();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("Core Version"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Core Type"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.primary)
                            .child(core_type_str),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Version"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.core_version.clone().unwrap_or_else(|| "Not running".into())),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Log Level"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(log_level_str),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().flex_1())
                    .child(
                        Button::new("upgrade-core")
                            .child("Upgrade Core")
                            .primary()
                            .when(self.is_upgrading, |this| this.disabled(true)),
                    ),
            )
    }

    fn render_smart_core_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let smart = self.smart_settings.read(cx);

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .when(smart.enabled, |this| this.border_color(theme.primary))
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Smart Core"),
                    )
                    .child(
                        Switch::new("smart-core")
                            .with_size(gpui_component::Size::Small)
                            .checked(smart.enabled)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.smart_settings.update(cx, |s, cx| {
                                    s.enabled = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .when(smart.enabled, |this| {
                this.child(
                    v_flex()
                        .gap_2()
                        .mt_2()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(div().text_xs().child("Use LightGBM"))
                                .child(
                                    Switch::new("lightgbm")
                                        .with_size(gpui_component::Size::XSmall)
                                        .checked(smart.use_lightgbm)
                                        .on_click(cx.listener(|this, checked, _, cx| {
                                            this.smart_settings.update(cx, |s, cx| {
                                                s.use_lightgbm = *checked;
                                                cx.notify();
                                            });
                                        })),
                                ),
                        )
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(div().text_xs().child("Collect Data"))
                                .child(
                                    Switch::new("collect")
                                        .with_size(gpui_component::Size::XSmall)
                                        .checked(smart.collect_data)
                                        .on_click(cx.listener(|this, checked, _, cx| {
                                            this.smart_settings.update(cx, |s, cx| {
                                                s.collect_data = *checked;
                                                cx.notify();
                                            });
                                        })),
                                ),
                        )
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(div().text_xs().child("Collector Size (MB)"))
                                .child(div().text_xs().child(smart.collector_size.to_string())),
                        ),
                )
            })
    }

    fn render_misc_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("Miscellaneous"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Allow LAN"))
                    .child(
                        Switch::new("allow-lan")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.allow_lan)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.allow_lan = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("IPv6"))
                    .child(
                        Switch::new("ipv6")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.ipv6)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.ipv6 = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Unified Delay"))
                    .child(
                        Switch::new("unified-delay")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.unified_delay)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.unified_delay = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("TCP Concurrent"))
                    .child(
                        Switch::new("tcp-concurrent")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.tcp_concurrent)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.tcp_concurrent = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Store Selected"))
                    .child(
                        Switch::new("store-selected")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.store_selected)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.store_selected = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
    }
}

impl PageTrait for MihomoPage {
    fn title() -> &'static str {
        "Mihomo"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Settings
    }
}

impl Render for MihomoPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .overflow_y_hidden()
            .gap_4()
            .p_4()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Mihomo Core Settings"),
                    )
                    .child(
                        Button::new("save")
                            .child("Save & Restart")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.save_settings(cx);
                            })),
                    ),
            )
            .child(self.render_core_section(cx))
            .child(self.render_smart_core_section(cx))
            .child(self.render_ports_section(cx))
            .child(self.render_controller_section(cx))
            .child(self.render_misc_section(cx))
    }
}