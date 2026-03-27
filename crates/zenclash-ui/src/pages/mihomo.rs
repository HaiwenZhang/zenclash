use gpui::{
    div, prelude::FluentBuilder, px, App, AppContext, Context, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::Input,
    select::Select,
    switch::Switch,
    tab::Tab,
    tab::TabBar,
    v_flex, ActiveTheme, Disableable, Sizable,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::Page;
use crate::pages::PageTrait;

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
    settings: Entity<MihomoSettings>,
    smart_settings: Entity<SmartCoreSettings>,
    core_version: Option<String>,
    is_upgrading: bool,
}

impl MihomoPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            settings: cx.new(|_| MihomoSettings::default()),
            smart_settings: cx.new(|_| SmartCoreSettings::default()),
            core_version: None,
            is_upgrading: false,
        }
    }

    fn render_ports_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().w(px(80.)).child(settings.mixed_port.to_string()))
                            .child(
                                Switch::new("mixed-port").with_size(gpui_component::Size::Small),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("SOCKS Port"))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().w(px(80.)).child(settings.socks_port.to_string()))
                            .child(
                                Switch::new("socks-port").with_size(gpui_component::Size::Small),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("HTTP Port"))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().w(px(80.)).child(settings.http_port.to_string()))
                            .child(Switch::new("http-port").with_size(gpui_component::Size::Small)),
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
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().child("Core Type:"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.primary)
                                    .child(settings.core_type.as_str()),
                            ),
                    )
                    .child(
                        Button::new("upgrade-core")
                            .child("Upgrade Core")
                            .primary()
                            .when(self.is_upgrading, |this| this.disabled(true)),
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
                            .child(settings.log_level.as_str()),
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
                            .checked(smart.enabled),
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
                                        .checked(smart.use_lightgbm),
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
                                        .checked(smart.collect_data),
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
                            .checked(settings.allow_lan),
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
                            .checked(settings.ipv6),
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
                            .checked(settings.unified_delay),
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
                            .checked(settings.tcp_concurrent),
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
                            .checked(settings.store_selected),
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
        let theme = cx.theme();

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
                    .child(Button::new("save").child("Save & Restart").primary()),
            )
            .child(self.render_core_section(cx))
            .child(self.render_smart_core_section(cx))
            .child(self.render_ports_section(cx))
            .child(self.render_controller_section(cx))
            .child(self.render_misc_section(cx))
    }
}
