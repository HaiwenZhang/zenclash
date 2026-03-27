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

use super::Page;
use crate::pages::PageTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TunStack {
    #[default]
    Mixed,
    Gvisor,
    System,
}

impl TunStack {
    pub fn as_str(&self) -> &'static str {
        match self {
            TunStack::Mixed => "mixed",
            TunStack::Gvisor => "gvisor",
            TunStack::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "gvisor" => TunStack::Gvisor,
            "system" => TunStack::System,
            _ => TunStack::Mixed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunSettings {
    pub enabled: bool,
    pub stack: TunStack,
    pub device: String,
    pub auto_route: bool,
    pub auto_redirect: bool,
    pub auto_detect_interface: bool,
    pub dns_hijack: Vec<String>,
    pub strict_route: bool,
    pub route_exclude_address: Vec<String>,
    pub mtu: u32,
}

impl Default for TunSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            stack: TunStack::default(),
            device: if cfg!(target_os = "macos") {
                "utun1500".into()
            } else {
                "Mihomo".into()
            },
            auto_route: true,
            auto_redirect: false,
            auto_detect_interface: true,
            dns_hijack: vec!["any:53".into()],
            strict_route: false,
            route_exclude_address: vec![],
            mtu: 1500,
        }
    }
}

pub struct TunPage {
    settings: Entity<TunSettings>,
    has_permission: bool,
}

impl TunPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            settings: cx.new(|_| TunSettings::default()),
            has_permission: false,
        }
    }

    fn render_permission_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

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
                    .child("TUN Permission"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child(if self.has_permission {
                        "Core has TUN permission"
                    } else {
                        "Core needs permission for TUN mode"
                    }))
                    .child(
                        Button::new("grant-permission")
                            .child("Grant Permission")
                            .when(self.has_permission, |this| this.disabled(true)),
                    ),
            )
            .when(cfg!(target_os = "windows"), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .py_2()
                        .child(div().text_sm().child("Windows Firewall"))
                        .child(Button::new("setup-firewall").child("Setup Firewall")),
                )
            })
    }

    fn render_basic_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child("Basic Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Stack Mode"))
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(settings.stack == TunStack::Mixed, |this| {
                                        this.bg(theme.primary).text_color(theme.primary_foreground)
                                    })
                                    .when(settings.stack != TunStack::Mixed, |this| {
                                        this.bg(theme.muted).text_color(theme.muted_foreground)
                                    })
                                    .text_xs()
                                    .child("Mixed"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(settings.stack == TunStack::Gvisor, |this| {
                                        this.bg(theme.primary).text_color(theme.primary_foreground)
                                    })
                                    .when(settings.stack != TunStack::Gvisor, |this| {
                                        this.bg(theme.muted).text_color(theme.muted_foreground)
                                    })
                                    .text_xs()
                                    .child("gVisor"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(settings.stack == TunStack::System, |this| {
                                        this.bg(theme.primary).text_color(theme.primary_foreground)
                                    })
                                    .when(settings.stack != TunStack::System, |this| {
                                        this.bg(theme.muted).text_color(theme.muted_foreground)
                                    })
                                    .text_xs()
                                    .child("System"),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Device Name"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(div().child(settings.device.clone())),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("MTU"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(settings.mtu.to_string()),
                    ),
            )
    }

    fn render_route_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child("Route Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Auto Route"))
                    .child(
                        Switch::new("auto-route")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.auto_route),
                    ),
            )
            .when(cfg!(target_os = "linux"), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .py_2()
                        .child(div().text_sm().child("Auto Redirect"))
                        .child(
                            Switch::new("auto-redirect")
                                .with_size(gpui_component::Size::Small)
                                .checked(settings.auto_redirect),
                        ),
                )
            })
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Auto Detect Interface"))
                    .child(
                        Switch::new("auto-detect")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.auto_detect_interface),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Strict Route"))
                    .child(
                        Switch::new("strict-route")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.strict_route),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("DNS Hijack"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(settings.dns_hijack.join(", ")),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .py_2()
                    .child(div().text_sm().child("Exclude Addresses"))
                    .when(settings.route_exclude_address.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("No excluded addresses"),
                        )
                    })
                    .children(settings.route_exclude_address.iter().map(|addr| {
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(addr.clone())
                    })),
            )
    }

    fn render_dns_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

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
                    .child("DNS Settings"),
            )
            .when(cfg!(target_os = "macos"), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .py_2()
                        .child(div().text_sm().child("Auto Set DNS (macOS)"))
                        .child(Switch::new("auto-set-dns").with_size(gpui_component::Size::Small)),
                )
            })
    }
}

impl PageTrait for TunPage {
    fn title() -> &'static str {
        "TUN"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Map
    }
}

impl Render for TunPage {
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
                            .child("TUN Mode Settings"),
                    )
                    .child(Button::new("save").child("Save & Restart").primary()),
            )
            .child(self.render_permission_section(cx))
            .child(self.render_basic_section(cx))
            .child(self.render_route_section(cx))
            .child(self.render_dns_section(cx))
    }
}
