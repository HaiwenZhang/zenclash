use gpui::{AppContext, InteractiveElement, 
    div, prelude::FluentBuilder, px, App, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window,
};
use gpui_component::{Sizable, button::{Button, ButtonVariants}, h_flex, input::Input, switch::Switch, v_flex, ActiveTheme};
use serde::{Deserialize, Serialize};

use super::Page;
use crate::pages::PageTrait;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnifferProtocolConfig {
    pub ports: Vec<u16>,
    #[serde(rename = "override-destination")]
    pub override_destination: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnifferSettings {
    pub enable: bool,
    #[serde(rename = "parse-pure-ip")]
    pub parse_pure_ip: bool,
    #[serde(rename = "force-dns-mapping")]
    pub force_dns_mapping: bool,
    #[serde(rename = "override-destination")]
    pub override_destination: bool,
    pub sniff: SnifferProtocols,
    #[serde(rename = "skip-domain")]
    pub skip_domain: Vec<String>,
    #[serde(rename = "force-domain")]
    pub force_domain: Vec<String>,
    #[serde(rename = "skip-dst-address")]
    pub skip_dst_address: Vec<String>,
    #[serde(rename = "skip-src-address")]
    pub skip_src_address: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnifferProtocols {
    #[serde(default)]
    pub HTTP: SnifferProtocolConfig,
    #[serde(default)]
    pub TLS: SnifferProtocolConfig,
    #[serde(default)]
    pub QUIC: SnifferProtocolConfig,
}

impl Default for SnifferProtocols {
    fn default() -> Self {
        Self {
            HTTP: SnifferProtocolConfig {
                ports: vec![80, 443],
                override_destination: false,
            },
            TLS: SnifferProtocolConfig {
                ports: vec![443],
                override_destination: false,
            },
            QUIC: SnifferProtocolConfig {
                ports: vec![],
                override_destination: false,
            },
        }
    }
}

pub struct SnifferPage {
    settings: Entity<SnifferSettings>,
    changed: bool,
}

impl SnifferPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            settings: cx.new(|_| SnifferSettings {
                enable: true,
                parse_pure_ip: true,
                force_dns_mapping: true,
                override_destination: false,
                sniff: SnifferProtocols::default(),
                skip_domain: vec!["+.push.apple.com".into()],
                force_domain: vec![],
                skip_dst_address: vec![
                    "91.105.192.0/23".into(),
                    "91.108.4.0/22".into(),
                    "149.154.160.0/20".into(),
                ],
                skip_src_address: vec![],
            }),
            changed: false,
        }
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
                    .child(div().text_sm().child("Enable Sniffer"))
                    .child(Switch::new("enable").with_size(gpui_component::Size::Small).checked(settings.enable)),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Override Destination"))
                    .child(
                        Switch::new("override-dest")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.override_destination),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Force DNS Mapping"))
                    .child(
                        Switch::new("force-dns")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.force_dns_mapping),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Parse Pure IP"))
                    .child(
                        Switch::new("parse-pure-ip")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.parse_pure_ip),
                    ),
            )
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
                    .child("Protocol Ports"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("HTTP Ports"))
                    .child(
                        div().text_sm().text_color(theme.muted_foreground).child(
                            settings
                                .sniff
                                .HTTP
                                .ports
                                .iter()
                                .map(|p| p.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("TLS Ports"))
                    .child(
                        div().text_sm().text_color(theme.muted_foreground).child(
                            settings
                                .sniff
                                .TLS
                                .ports
                                .iter()
                                .map(|p| p.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("QUIC Ports"))
                    .child(div().text_sm().text_color(theme.muted_foreground).child(
                        if settings.sniff.QUIC.ports.is_empty() {
                            "None".into()
                        } else {
                            settings
                                .sniff
                                .QUIC
                                .ports
                                .iter()
                                .map(|p| p.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        },
                    )),
            )
    }

    fn render_skip_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child("Skip / Force Domains"),
            )
            .child(
                v_flex()
                    .gap_1()
                    .py_2()
                    .child(div().text_sm().child("Skip Domains"))
                    .children(settings.skip_domain.iter().map(|d| {
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(d.clone())
                    })),
            )
            .child(
                v_flex()
                    .gap_1()
                    .py_2()
                    .child(div().text_sm().child("Force Domains"))
                    .when(settings.force_domain.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("None"),
                        )
                    })
                    .children(settings.force_domain.iter().map(|d| {
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(d.clone())
                    })),
            )
            .child(
                v_flex()
                    .gap_1()
                    .py_2()
                    .child(div().text_sm().child("Skip Destination Addresses"))
                    .children(settings.skip_dst_address.iter().map(|a| {
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(a.clone())
                    })),
            )
    }
}

impl PageTrait for SnifferPage {
    fn title() -> &'static str {
        "Sniffer"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Search
    }
}

impl Render for SnifferPage {
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
                            .child("Sniffer Settings"),
                    )
                    .child(Button::new("save").child("Save").primary()),
            )
            .child(self.render_basic_section(cx))
            .child(self.render_ports_section(cx))
            .child(self.render_skip_section(cx))
    }
}
