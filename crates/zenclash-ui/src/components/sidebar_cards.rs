use gpui::{
    div, prelude::FluentBuilder, App, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{h_flex, ActiveTheme, Icon, IconName, Sizable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub version: String,
    pub memory: u64,
}

pub struct CoreCard {
    pub info: Option<CoreInfo>,
    pub is_selected: bool,
    pub on_click: Option<Box<dyn Fn() + 'static>>,
    pub on_restart: Option<Box<dyn Fn() + 'static>>,
}

impl CoreCard {
    pub fn new() -> Self {
        Self {
            info: None,
            is_selected: false,
            on_click: None,
            on_restart: None,
        }
    }

    pub fn info(mut self, info: CoreInfo) -> Self {
        self.info = Some(info);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_restart(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_restart = Some(Box::new(handler));
        self
    }
}

impl Default for CoreCard {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for CoreCard {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let version = self
            .info
            .as_ref()
            .map(|i| i.version.clone())
            .unwrap_or_else(|| "-".into());
        let memory = self
            .info
            .as_ref()
            .map(|i| zenclash_core::prelude::format_traffic(i.memory))
            .unwrap_or_else(|| "-".into());

        let on_click = self.on_click.take();
        let on_restart = self.on_restart.take();

        div()
            .id("core-card")
            .p_3()
            .gap_1()
            .when(self.is_selected, |this| this.bg(theme.primary))
            .cursor_pointer()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .when(self.is_selected, |this| {
                                this.text_color(theme.primary_foreground)
                            })
                            .child(version),
                    )
                    .child(
                        gpui_component::button::Button::new("restart")
                            .with_size(gpui_component::Size::XSmall)
                            .icon(Icon::new(IconName::LoaderCircle))
                            .when(self.is_selected, |this| {
                                this.text_color(theme.primary_foreground)
                            })
                            .on_click(move |_, _, _| {
                                if let Some(handler) = &on_restart {
                                    handler();
                                }
                            }),
                    ),
            )
            .child(
                h_flex()
                    .justify_between()
                    .text_xs()
                    .when(self.is_selected, |this| {
                        this.text_color(theme.primary_foreground)
                    })
                    .when(!self.is_selected, |this| this.text_color(theme.foreground))
                    .child("Core")
                    .child(memory),
            )
            .on_click(move |_, _, _| {
                if let Some(handler) = &on_click {
                    handler();
                }
            })
    }
}

pub struct ConnectionCard {
    pub count: usize,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub is_selected: bool,
    pub on_click: Option<Box<dyn Fn() + 'static>>,
}

impl ConnectionCard {
    pub fn new() -> Self {
        Self {
            count: 0,
            upload_speed: 0,
            download_speed: 0,
            is_selected: false,
            on_click: None,
        }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn speed(mut self, upload: u64, download: u64) -> Self {
        self.upload_speed = upload;
        self.download_speed = download;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Default for ConnectionCard {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ConnectionCard {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_click = self.on_click.take();

        div()
            .id("connection-card")
            .p_3()
            .gap_1()
            .when(self.is_selected, |this| this.bg(theme.primary))
            .cursor_pointer()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .when(self.is_selected, |this| {
                                this.text_color(theme.primary_foreground)
                            })
                            .when(!self.is_selected, |this| this.text_color(theme.foreground))
                            .child(format!("{} Connections", self.count)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .when(self.is_selected, |this| {
                                this.text_color(theme.primary_foreground)
                            })
                            .when(!self.is_selected, |this| {
                                this.text_color(theme.muted_foreground)
                            })
                            .child(format!(
                                "↑ {} ↓ {}",
                                zenclash_core::prelude::format_speed(self.upload_speed),
                                zenclash_core::prelude::format_speed(self.download_speed)
                            )),
                    ),
            )
            .on_click(move |_, _, _| {
                if let Some(handler) = &on_click {
                    handler();
                }
            })
    }
}

pub struct ProfileCard {
    pub name: Option<String>,
    pub traffic_used: u64,
    pub traffic_total: u64,
    pub expire: Option<i64>,
    pub is_selected: bool,
    pub on_click: Option<Box<dyn Fn() + 'static>>,
}

impl ProfileCard {
    pub fn new() -> Self {
        Self {
            name: None,
            traffic_used: 0,
            traffic_total: 0,
            expire: None,
            is_selected: false,
            on_click: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn traffic(mut self, used: u64, total: u64) -> Self {
        self.traffic_used = used;
        self.traffic_total = total;
        self
    }

    pub fn expire(mut self, expire: Option<i64>) -> Self {
        self.expire = expire;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Default for ProfileCard {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ProfileCard {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let name = self.name.unwrap_or_else(|| "No Profile".into());
        let has_traffic = self.traffic_total > 0;

        let on_click = self.on_click.take();

        div()
            .id("profile-card")
            .p_3()
            .gap_1()
            .when(self.is_selected, |this| this.bg(theme.primary))
            .cursor_pointer()
            .child(
                div()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .when(self.is_selected, |this| {
                        this.text_color(theme.primary_foreground)
                    })
                    .child(name),
            )
            .when(has_traffic, |this| {
                let used = zenclash_core::prelude::format_traffic(self.traffic_used);
                let total = zenclash_core::prelude::format_traffic(self.traffic_total);
                let expire_text = self
                    .expire
                    .map(|e| {
                        chrono::DateTime::from_timestamp(e, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "Unknown".into())
                    })
                    .unwrap_or_else(|| "Never".into());

                this.child(
                    h_flex()
                        .justify_between()
                        .text_xs()
                        .when(self.is_selected, |this| {
                            this.text_color(theme.primary_foreground)
                        })
                        .when(!self.is_selected, |this| {
                            this.text_color(theme.muted_foreground)
                        })
                        .child(format!("{}/{}", used, total))
                        .child(expire_text),
                )
            })
            .on_click(move |_, _, _| {
                if let Some(handler) = &on_click {
                    handler();
                }
            })
    }
}

pub struct RuleCard {
    pub count: usize,
    pub is_selected: bool,
    pub on_click: Option<Box<dyn Fn() + 'static>>,
}

impl RuleCard {
    pub fn new() -> Self {
        Self {
            count: 0,
            is_selected: false,
            on_click: None,
        }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Default for RuleCard {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for RuleCard {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_click = self.on_click.take();

        div()
            .id("rule-card")
            .p_3()
            .gap_1()
            .when(self.is_selected, |this| this.bg(theme.primary))
            .cursor_pointer()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::Menu).size_4())
                    .child(
                        div()
                            .text_xs()
                            .when(self.is_selected, |this| {
                                this.text_color(theme.primary_foreground)
                            })
                            .when(!self.is_selected, |this| this.text_color(theme.foreground))
                            .child(format!("{} Rules", self.count)),
                    ),
            )
            .on_click(move |_, _, _| {
                if let Some(handler) = &on_click {
                    handler();
                }
            })
    }
}
