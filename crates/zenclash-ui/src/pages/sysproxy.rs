use gpui::{
    div, prelude::FluentBuilder, px, App, AppContext, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::Input,
    switch::Switch,
    tab::Tab,
    tab::TabBar,
    v_flex, ActiveTheme, Disableable, Sizable,
};
use serde::{Deserialize, Serialize};

use super::Page;
use crate::pages::PageTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProxyMode {
    #[default]
    Off,
    Manual,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysproxySettings {
    pub mode: ProxyMode,
    pub http_port: u16,
    pub https_port: u16,
    pub socks_port: u16,
    pub bypass: Vec<String>,
    pub pac_url: Option<String>,
}

impl Default for SysproxySettings {
    fn default() -> Self {
        Self {
            mode: ProxyMode::default(),
            http_port: 7890,
            https_port: 7890,
            socks_port: 7891,
            bypass: vec![
                "localhost".into(),
                "127.0.0.1".into(),
                "192.168.*".into(),
                "10.*".into(),
                "172.16.*".into(),
            ],
            pac_url: None,
        }
    }
}

pub struct SysproxyPage {
    settings: Entity<SysproxySettings>,
    enabled: bool,
    changed: bool,
    focus_handle: FocusHandle,
}

impl SysproxyPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            settings: cx.new(|_| SysproxySettings::default()),
            enabled: false,
            changed: false,
            focus_handle: cx.focus_handle(),
        }
    }

    fn render_mode_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);

        h_flex()
            .gap_1()
            .p_1()
            .rounded(theme.radius)
            .bg(theme.muted)
            .children(
                [ProxyMode::Off, ProxyMode::Manual, ProxyMode::Auto]
                    .into_iter()
                    .map(|mode| {
                        let is_active = settings.mode == mode;
                        let label = match mode {
                            ProxyMode::Off => "Off",
                            ProxyMode::Manual => "Manual",
                            ProxyMode::Auto => "Auto (PAC)",
                        };
                        div()
                            .px_3()
                            .py_1()
                            .rounded(theme.radius)
                            .when(is_active, |this| {
                                this.bg(theme.background).text_color(theme.foreground)
                            })
                            .when(!is_active, |this| {
                                this.text_color(theme.muted_foreground)
                                    .hover(|this| this.bg(theme.transparent))
                            })
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(label)
                    }),
            )
    }

    fn render_manual_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child("Manual Proxy Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("HTTP Proxy"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("127.0.0.1:{}", settings.http_port)),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("HTTPS Proxy"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("127.0.0.1:{}", settings.https_port)),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("SOCKS Proxy"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("127.0.0.1:{}", settings.socks_port)),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .py_2()
                    .child(div().text_sm().child("Bypass List"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(settings.bypass.join(", ")),
                    ),
            )
    }

    fn render_auto_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);
        let pac_url = settings
            .pac_url
            .clone()
            .unwrap_or_else(|| format!("http://127.0.0.1:10000/pac"));

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
                    .child("Auto Proxy (PAC)"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("PAC URL"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(pac_url),
                    ),
            )
            .child(
                h_flex().gap_2().justify_end().child(
                    Button::new("edit-pac")
                        .with_size(gpui_component::Size::XSmall)
                        .child("Edit PAC Script"),
                ),
            )
    }

    fn render_status_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("System Proxy Status"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(self.enabled, |this| {
                                        this.bg(theme.success).text_color(theme.background)
                                    })
                                    .when(!self.enabled, |this| {
                                        this.bg(theme.muted).text_color(theme.muted_foreground)
                                    })
                                    .text_xs()
                                    .child(if self.enabled { "Enabled" } else { "Disabled" }),
                            )
                            .child(
                                Switch::new("sysproxy-toggle")
                                    .with_size(gpui_component::Size::Small)
                                    .checked(self.enabled),
                            ),
                    ),
            )
    }
}

impl PageTrait for SysproxyPage {
    fn title() -> &'static str {
        "System Proxy"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Globe
    }
}

impl Focusable for SysproxyPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SysproxyPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let settings = self.settings.read(cx).clone();

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
                            .child("System Proxy"),
                    )
                    .child(
                        Button::new("save")
                            .child("Save")
                            .primary()
                            .when(!self.changed, |this| this.disabled(true)),
                    ),
            )
            .child(self.render_status_section(cx))
            .child(self.render_mode_selector(cx))
            .when(settings.mode == ProxyMode::Manual, |this| {
                this.child(self.render_manual_section(cx))
            })
            .when(settings.mode == ProxyMode::Auto, |this| {
                this.child(self.render_auto_section(cx))
            })
    }
}
