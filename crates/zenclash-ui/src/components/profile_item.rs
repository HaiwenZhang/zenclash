use gpui::{
    div, prelude::FluentBuilder, App, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{
    button::Button, h_flex, progress::Progress, tag::Tag, ActiveTheme, Disableable, Icon,
    IconName, Sizable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub profile_type: ProfileType,
    pub url: Option<String>,
    pub home: Option<String>,
    pub updated: i64,
    pub extra: Option<ProfileExtra>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProfileType {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "remote")]
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExtra {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: Option<i64>,
}

pub struct ProfileItem {
    pub info: ProfileInfo,
    pub is_current: bool,
    pub is_updating: bool,
    pub on_select: Option<Box<dyn Fn() + 'static>>,
    pub on_update: Option<Box<dyn Fn() + 'static>>,
    pub on_delete: Option<Box<dyn Fn() + 'static>>,
    pub on_edit: Option<Box<dyn Fn() + 'static>>,
    pub on_open_file: Option<Box<dyn Fn() + 'static>>,
    pub on_open_home: Option<Box<dyn Fn() + 'static>>,
}

impl ProfileItem {
    pub fn new(info: ProfileInfo) -> Self {
        Self {
            info,
            is_current: false,
            is_updating: false,
            on_select: None,
            on_update: None,
            on_delete: None,
            on_edit: None,
            on_open_file: None,
            on_open_home: None,
        }
    }

    pub fn current(mut self, current: bool) -> Self {
        self.is_current = current;
        self
    }

    pub fn updating(mut self, updating: bool) -> Self {
        self.is_updating = updating;
        self
    }

    pub fn on_select(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_update(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_update = Some(Box::new(handler));
        self
    }

    pub fn on_delete(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_delete = Some(Box::new(handler));
        self
    }

    pub fn on_edit(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_edit = Some(Box::new(handler));
        self
    }

    pub fn on_open_file(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open_file = Some(Box::new(handler));
        self
    }

    pub fn on_open_home(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open_home = Some(Box::new(handler));
        self
    }

    fn format_traffic(bytes: u64) -> String {
        zenclash_core::prelude::format_traffic(bytes)
    }

    fn usage_percent(&self) -> f32 {
        if let Some(ref extra) = self.info.extra {
            zenclash_core::prelude::calc_percent(extra.upload + extra.download, extra.total)
        } else {
            0.0
        }
    }
}

impl RenderOnce for ProfileItem {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let has_extra = self.info.extra.is_some();
        let is_remote = self.info.profile_type == ProfileType::Remote;

        let on_select = self.on_select.take();
        let on_update = self.on_update.take();

        let fg_color = if self.is_current {
            theme.primary_foreground
        } else {
            theme.foreground
        };

        div()
            .id("profile-item")
            .p_3()
            .gap_1()
            .when(self.is_current, |this| this.bg(theme.primary))
            .cursor_pointer()
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
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(fg_color)
                            .child(self.info.name.clone()),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .when(is_remote, |this| {
                                let updating = self.is_updating;
                                this.child(
                                    Button::new("update")
                                        .with_size(gpui_component::Size::XSmall)
                                        .icon(Icon::new(IconName::LoaderCircle))
                                        .when(updating, |this| this.disabled(true))
                                        .text_color(fg_color)
                                        .on_click(move |_, _, _| {
                                            if let Some(handler) = &on_update {
                                                handler();
                                            }
                                        }),
                                )
                            })
                            .child(
                                Button::new("menu")
                                    .with_size(gpui_component::Size::XSmall)
                                    .icon(Icon::new(IconName::EllipsisVertical))
                                    .text_color(fg_color),
                            ),
                    ),
            )
            .when(is_remote && has_extra, |this| {
                if let Some(ref extra) = self.info.extra {
                    let usage = Self::format_traffic(extra.upload + extra.download);
                    let total = Self::format_traffic(extra.total);

                    this.child(
                        h_flex()
                            .justify_between()
                            .text_xs()
                            .text_color(fg_color)
                            .child(format!("{}/{}", usage, total))
                            .child(
                                extra
                                    .expire
                                    .map(|e| {
                                        let dt = chrono::DateTime::from_timestamp(e, 0)
                                            .unwrap_or_else(|| chrono::Utc::now());
                                        dt.format("%Y-%m-%d").to_string()
                                    })
                                    .unwrap_or_else(|| "Never expire".into()),
                            ),
                    )
                } else {
                    this
                }
            })
            .when(is_remote && !has_extra, |this| {
                this.child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            Tag::new()
                                .with_size(gpui_component::Size::XSmall)
                                .outline()
                                .text_color(if self.is_current {
                                    theme.primary_foreground
                                } else {
                                    theme.primary
                                })
                                .child("Remote"),
                        )
                        .child(div().text_xs().text_color(fg_color).child(
                            zenclash_core::prelude::format_relative_time(self.info.updated),
                        )),
                )
            })
            .when(self.info.profile_type == ProfileType::Local, |this| {
                this.child(
                    Tag::new()
                        .with_size(gpui_component::Size::XSmall)
                        .outline()
                        .text_color(if self.is_current {
                            theme.primary_foreground
                        } else {
                            theme.primary
                        })
                        .child("Local"),
                )
            })
            .when(has_extra, |this| {
                this.child(
                    Progress::new()
                        .value(self.usage_percent())
                        .when(self.is_current, |this| this.bg(theme.primary_foreground)),
                )
            })
            .on_click(move |_, _, _| {
                if let Some(handler) = &on_select {
                    handler();
                }
            })
    }
}
