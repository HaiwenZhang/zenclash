use gpui::{
    div, prelude::FluentBuilder, px, App, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{h_flex, tag::Tag, v_flex, ActiveTheme, Icon, IconName, Sizable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "silent")]
    Silent,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warning" => LogLevel::Warning,
            "debug" => LogLevel::Debug,
            "silent" => LogLevel::Silent,
            _ => LogLevel::Info,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Silent => "SILENT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogInfo {
    #[serde(rename = "type")]
    pub log_type: String,
    pub payload: String,
    pub timestamp: Option<i64>,
}

impl LogInfo {
    pub fn level(&self) -> LogLevel {
        LogLevel::from_str(&self.log_type)
    }
}

pub struct LogItem {
    pub info: LogInfo,
    pub show_timestamp: bool,
}

impl LogItem {
    pub fn new(info: LogInfo) -> Self {
        Self {
            info,
            show_timestamp: true,
        }
    }

    pub fn show_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    fn level_color(&self, theme: &gpui_component::Theme) -> gpui::Hsla {
        match self.info.level() {
            LogLevel::Error => theme.danger,
            LogLevel::Warning => theme.warning,
            LogLevel::Info => theme.primary,
            LogLevel::Debug => theme.muted_foreground,
            LogLevel::Silent => theme.muted_foreground,
        }
    }

    fn level_icon(&self) -> IconName {
        match self.info.level() {
            LogLevel::Error => IconName::Close,
            LogLevel::Warning => IconName::TriangleAlert,
            LogLevel::Info => IconName::Info,
            LogLevel::Debug => IconName::Settings,
            LogLevel::Silent => IconName::Minus,
        }
    }

    fn format_time(timestamp: i64) -> String {
        let dt =
            chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());
        dt.format("%H:%M:%S").to_string()
    }
}

impl RenderOnce for LogItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let level_color = self.level_color(theme);
        let timestamp = self.info.timestamp.map(Self::format_time);

        h_flex()
            .gap_2()
            .p_2()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .items_start()
            .when_some(timestamp, |this, ts| {
                if self.show_timestamp {
                    this.child(div().text_xs().text_color(theme.muted_foreground).child(ts))
                } else {
                    this
                }
            })
            .child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        Icon::new(self.level_icon())
                            .text_color(level_color)
                            .size_3(),
                    )
                    .child(
                        Tag::new()
                            .with_size(gpui_component::Size::XSmall)
                            .outline()
                            .text_color(level_color)
                            .child(self.info.level().as_str()),
                    ),
            )
            .child(
                div()
                    .id("payload")
                    .flex_1()
                    .text_sm()
                    .font_family("monospace")
                    .overflow_x_scroll()
                    .child(self.info.payload.clone()),
            )
    }
}
