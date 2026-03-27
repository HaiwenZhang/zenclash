use gpui::{
    div, prelude::FluentBuilder, px, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    tag::Tag,
    v_flex, ActiveTheme, Icon, IconName, Sizable,
};

use super::ConnectionInfo;

pub struct ConnectionDetail {
    pub info: ConnectionInfo,
    pub on_close: Option<Box<dyn Fn() + 'static>>,
    pub on_close_connection: Option<Box<dyn Fn() + 'static>>,
    pub on_copy_rule: Option<Box<dyn Fn(String) + 'static>>,
}

impl ConnectionDetail {
    pub fn new(info: ConnectionInfo) -> Self {
        Self {
            info,
            on_close: None,
            on_close_connection: None,
            on_copy_rule: None,
        }
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn on_close_connection(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close_connection = Some(Box::new(handler));
        self
    }

    pub fn on_copy_rule(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_copy_rule = Some(Box::new(handler));
        self
    }

    fn render_field(
        &self,
        label: &str,
        value: &str,
        theme: &gpui_component::Theme,
    ) -> impl IntoElement {
        h_flex()
            .gap_2()
            .items_start()
            .child(
                div()
                    .w(px(100.))
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(label.to_string()),
            )
            .child(div().flex_1().text_sm().child(value.to_string()))
    }

    fn generate_rule(&self) -> String {
        let dest = self
            .info
            .metadata
            .host
            .clone()
            .or_else(|| self.info.metadata.destination_ip.clone())
            .unwrap_or_default();

        let port = self.info.metadata.destination_port.clone();
        let proxy = self
            .info
            .chains
            .first()
            .cloned()
            .unwrap_or_else(|| "DIRECT".into());

        if dest.contains('.') && !dest.parse::<std::net::IpAddr>().is_ok() {
            format!("DOMAIN-SUFFIX,{},{}", dest, proxy)
        } else if let Ok(ip) = dest.parse::<std::net::IpAddr>() {
            if ip.is_ipv4() {
                format!("IP-CIDR,{}/{},{}", dest, port, proxy)
            } else {
                format!("IP-CIDR6,{}/{},{}", dest, port, proxy)
            }
        } else {
            format!("MATCH,{}", proxy)
        }
    }
}

impl RenderOnce for ConnectionDetail {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let rule = self.generate_rule();

        let upload = zenclash_core::prelude::format_traffic(self.info.upload);
        let download = zenclash_core::prelude::format_traffic(self.info.download);
        let start_time = chrono::DateTime::from_timestamp(self.info.start, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        let on_close = self.on_close.take();
        let on_close_conn = self.on_close_connection.take();
        let on_copy_rule = self.on_copy_rule.take();

        div()
            .relative()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(500.))
                    .max_h(px(600.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .overflow_y_scrollbar()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Connection Details"),
                            )
                            .child(
                                Button::new("close")
                                    .with_size(gpui_component::Size::XSmall)
                                    .icon(Icon::new(IconName::Close))
                                    .on_click(move |_, _, _| {
                                        if let Some(handler) = &on_close {
                                            handler();
                                        }
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Tag::new()
                                    .with_size(gpui_component::Size::Small)
                                    .with_variant(if self.info.is_active {
                                        gpui_component::tag::TagVariant::Success
                                    } else {
                                        gpui_component::tag::TagVariant::Danger
                                    })
                                    .child(if self.info.is_active {
                                        "Active"
                                    } else {
                                        "Closed"
                                    }),
                            )
                            .child(
                                Tag::new()
                                    .with_size(gpui_component::Size::Small)
                                    .outline()
                                    .child(format!(
                                        "{}({})",
                                        self.info.metadata.conn_type,
                                        self.info.metadata.network.to_uppercase()
                                    )),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(self.render_field(
                                "Process",
                                self.info.metadata.process.as_deref().unwrap_or("-"),
                                theme,
                            ))
                            .child(self.render_field(
                                "Source",
                                &format!(
                                    "{}:{}",
                                    self.info.metadata.source_ip, self.info.metadata.source_port
                                ),
                                theme,
                            ))
                            .child(
                                self.render_field(
                                    "Destination",
                                    self.info
                                        .metadata
                                        .host
                                        .as_deref()
                                        .or(self.info.metadata.destination_ip.as_deref())
                                        .unwrap_or("-"),
                                    theme,
                                ),
                            )
                            .child(self.render_field(
                                "Destination Port",
                                &self.info.metadata.destination_port,
                                theme,
                            ))
                            .child(self.render_field(
                                "Sniffed Host",
                                self.info.metadata.sniff_host.as_deref().unwrap_or("-"),
                                theme,
                            ))
                            .child(self.render_field("Chain", &self.info.chains.join(" → "), theme))
                            .child(self.render_field("Start Time", &start_time, theme))
                            .child(self.render_field("Upload", &upload, theme))
                            .child(self.render_field("Download", &download, theme)),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("Generated Rule"),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded(theme.radius)
                                    .bg(theme.muted)
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(rule.clone()),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("copy-rule").child("Copy Rule").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_copy_rule {
                                        handler(rule.clone());
                                    }
                                },
                            ))
                            .when(self.info.is_active, |this| {
                                this.child(
                                    Button::new("close-conn")
                                        .danger()
                                        .child("Close Connection")
                                        .on_click(move |_, _, _| {
                                            if let Some(handler) = &on_close_conn {
                                                handler();
                                            }
                                        }),
                                )
                            }),
                    ),
            )
    }
}
