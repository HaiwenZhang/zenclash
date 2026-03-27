use gpui::{
    div, prelude::FluentBuilder, px, App, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{button::Button, h_flex, tag::Tag, v_flex, ActiveTheme, Icon, Sizable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub upload: u64,
    pub download: u64,
    pub upload_speed: Option<u64>,
    pub download_speed: Option<u64>,
    pub start: i64,
    pub chains: Vec<String>,
    pub is_active: bool,
    pub metadata: ConnectionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    #[serde(rename = "type")]
    pub conn_type: String,
    pub network: String,
    pub host: Option<String>,
    pub sniff_host: Option<String>,
    #[serde(rename = "destinationIP")]
    pub destination_ip: Option<String>,
    #[serde(rename = "remoteDestination")]
    pub remote_destination: Option<String>,
    #[serde(rename = "sourceIP")]
    pub source_ip: String,
    pub source_port: String,
    pub destination_port: String,
    pub process: Option<String>,
    pub process_path: Option<String>,
}

pub struct ConnectionItem {
    pub info: ConnectionInfo,
    pub show_icon: bool,
    pub on_close: Option<Box<dyn Fn() + 'static>>,
    pub on_detail: Option<Box<dyn Fn() + 'static>>,
}

impl ConnectionItem {
    pub fn new(info: ConnectionInfo) -> Self {
        Self {
            info,
            show_icon: false,
            on_close: None,
            on_detail: None,
        }
    }

    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn on_detail(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_detail = Some(Box::new(handler));
        self
    }

    fn format_traffic(bytes: u64) -> String {
        zenclash_core::prelude::format_traffic(bytes)
    }

    fn format_speed(bytes: Option<u64>) -> String {
        bytes
            .map(|b| zenclash_core::prelude::format_speed(b))
            .unwrap_or_default()
    }

    fn format_time_ago(timestamp: i64) -> String {
        zenclash_core::prelude::format_relative_time(timestamp)
    }

    fn destination(&self) -> String {
        self.info
            .metadata
            .host
            .clone()
            .or_else(|| self.info.metadata.sniff_host.clone())
            .or_else(|| self.info.metadata.destination_ip.clone())
            .or_else(|| self.info.metadata.remote_destination.clone())
            .unwrap_or_else(|| "Unknown".into())
    }

    fn process_name(&self) -> String {
        self.info
            .metadata
            .process
            .clone()
            .map(|p| p.replace(".exe", ""))
            .unwrap_or_else(|| self.info.metadata.source_ip.clone())
    }
}

impl RenderOnce for ConnectionItem {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let destination = self.destination();
        let process_name = self.process_name();
        let time_ago = Self::format_time_ago(self.info.start);
        let upload = Self::format_traffic(self.info.upload);
        let download = Self::format_traffic(self.info.download);
        let has_speed = self.info.upload_speed.is_some() || self.info.download_speed.is_some();
        let upload_speed = Self::format_speed(self.info.upload_speed);
        let download_speed = Self::format_speed(self.info.download_speed);

        let on_close = self.on_close.take();
        let on_detail = self.on_detail.take();

        div()
            .id("connection-item")
            .p_2()
            .gap_2()
            .cursor_pointer()
            .child(
                h_flex()
                    .gap_2()
                    .items_start()
                    .when(self.show_icon, |this| {
                        this.child(
                            div()
                                .size(px(48.))
                                .rounded(theme.radius)
                                .bg(theme.muted)
                                .items_center()
                                .justify_center()
                                .child(
                                    Icon::new(gpui_component::IconName::Globe)
                                        .size_5()
                                        .text_color(theme.muted_foreground),
                                ),
                        )
                    })
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_1()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex_1()
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .whitespace_nowrap()
                                            .child(format!("{} → {}", process_name, destination)),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .child(time_ago),
                                            )
                                            .child(
                                                Button::new("close")
                                                    .with_size(gpui_component::Size::XSmall)
                                                    .icon(Icon::new(if self.info.is_active {
                                                        gpui_component::IconName::Close
                                                    } else {
                                                        gpui_component::IconName::Delete
                                                    }))
                                                    .when(!self.info.is_active, |this| {
                                                        this.text_color(theme.danger)
                                                    })
                                                    .on_click(move |_, _, _| {
                                                        if let Some(handler) = &on_close {
                                                            handler();
                                                        }
                                                    }),
                                            ),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .flex_wrap()
                                    .child(
                                        Tag::new()
                                            .with_size(gpui_component::Size::XSmall)
                                            .with_variant(if self.info.is_active {
                                                gpui_component::tag::TagVariant::Primary
                                            } else {
                                                gpui_component::tag::TagVariant::Danger
                                            })
                                            .child(format!(
                                                "{}({})",
                                                self.info.metadata.conn_type,
                                                self.info.metadata.network.to_uppercase()
                                            )),
                                    )
                                    .when(!self.info.chains.is_empty(), |this| {
                                        this.child(
                                            Tag::new()
                                                .with_size(gpui_component::Size::XSmall)
                                                .outline()
                                                .child(
                                                    self.info
                                                        .chains
                                                        .first()
                                                        .unwrap_or(&"".into())
                                                        .clone(),
                                                ),
                                        )
                                    })
                                    .child(
                                        Tag::new()
                                            .with_size(gpui_component::Size::XSmall)
                                            .outline()
                                            .child(format!("↑ {} ↓ {}", upload, download)),
                                    )
                                    .when(has_speed, |this| {
                                        this.child(
                                            Tag::new()
                                                .with_size(gpui_component::Size::XSmall)
                                                .outline()
                                                .text_color(theme.primary)
                                                .child(format!(
                                                    "↑ {}/s ↓ {}/s",
                                                    upload_speed, download_speed
                                                )),
                                        )
                                    }),
                            ),
                    ),
            )
            .on_click(move |_, _, _| {
                if let Some(handler) = &on_detail {
                    handler();
                }
            })
    }
}
