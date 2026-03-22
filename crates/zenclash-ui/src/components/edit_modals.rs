use gpui::{div, prelude::FluentBuilder, px, App, IntoElement, RenderOnce, Styled, Window};
use gpui_component::{
    button::Button, card::Card, chip::Chip, h_flex, input::TextInput, select::Select,
    switch::Switch, v_flex, ActiveTheme, Icon, IconName, Sizable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEditData {
    pub name: String,
    pub url: Option<String>,
    pub interval: Option<u32>,
    pub auth: Option<String>,
}

pub struct EditProfileModal {
    pub data: ProfileEditData,
    pub is_remote: bool,
    pub on_save: Option<Box<dyn Fn(ProfileEditData) + 'static>>,
    pub on_cancel: Option<Box<dyn Fn() + 'static>>,
}

impl EditProfileModal {
    pub fn new(data: ProfileEditData) -> Self {
        Self {
            data,
            is_remote: false,
            on_save: None,
            on_cancel: None,
        }
    }

    pub fn remote(mut self, is_remote: bool) -> Self {
        self.is_remote = is_remote;
        self
    }

    pub fn on_save(mut self, handler: impl Fn(ProfileEditData) + 'static) -> Self {
        self.on_save = Some(Box::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for EditProfileModal {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_save = self.on_save.take();
        let on_cancel = self.on_cancel.take();

        div()
            .fixed()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .z_index(1000)
            .child(
                Card::new()
                    .w(px(450.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Edit Profile"),
                    )
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child("Name"),
                                    )
                                    .child(
                                        div()
                                            .p_2()
                                            .rounded(theme.radius)
                                            .bg(theme.muted)
                                            .text_sm()
                                            .child(self.data.name.clone()),
                                    ),
                            )
                            .when(self.is_remote, |this| {
                                this.child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .child("URL"),
                                        )
                                        .child(
                                            div()
                                                .p_2()
                                                .rounded(theme.radius)
                                                .bg(theme.muted)
                                                .text_sm()
                                                .text_color(theme.muted_foreground)
                                                .child(self.data.url.clone().unwrap_or_default()),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .child("Update Interval (minutes)"),
                                        )
                                        .child(
                                            div()
                                                .p_2()
                                                .rounded(theme.radius)
                                                .bg(theme.muted)
                                                .text_sm()
                                                .child(
                                                    self.data
                                                        .interval
                                                        .map(|i| i.to_string())
                                                        .unwrap_or_else(|| "0".into()),
                                                ),
                                        ),
                                )
                            }),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("cancel").child("Cancel").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_cancel {
                                        handler();
                                    }
                                },
                            ))
                            .child(Button::new("save").primary().child("Save").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_save {
                                        handler(ProfileEditData::default());
                                    }
                                },
                            )),
                    ),
            )
    }
}

#[derive(Debug, Clone)]
pub struct RuleEditData {
    pub rule_type: String,
    pub payload: String,
    pub proxy: String,
}

pub struct EditRuleModal {
    pub data: RuleEditData,
    pub rule_types: Vec<String>,
    pub proxies: Vec<String>,
    pub on_save: Option<Box<dyn Fn(RuleEditData) + 'static>>,
    pub on_cancel: Option<Box<dyn Fn() + 'static>>,
}

impl EditRuleModal {
    pub fn new(data: RuleEditData) -> Self {
        Self {
            data,
            rule_types: vec![
                "DOMAIN".into(),
                "DOMAIN-SUFFIX".into(),
                "DOMAIN-KEYWORD".into(),
                "IP-CIDR".into(),
                "IP-CIDR6".into(),
                "SRC-IP-CIDR".into(),
                "GEOIP".into(),
                "GEOSITE".into(),
                "DST-PORT".into(),
                "SRC-PORT".into(),
                "PROCESS-NAME".into(),
                "MATCH".into(),
            ],
            proxies: Vec::new(),
            on_save: None,
            on_cancel: None,
        }
    }

    pub fn proxies(mut self, proxies: Vec<String>) -> Self {
        self.proxies = proxies;
        self
    }

    pub fn on_save(mut self, handler: impl Fn(RuleEditData) + 'static) -> Self {
        self.on_save = Some(Box::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for EditRuleModal {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_save = self.on_save.take();
        let on_cancel = self.on_cancel.take();

        div()
            .fixed()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .z_index(1000)
            .child(
                Card::new()
                    .w(px(500.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Edit Rule"),
                    )
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child("Rule Type"),
                                    )
                                    .child(h_flex().flex_wrap().gap_1().children(
                                        self.rule_types.iter().map(|t| {
                                            let is_selected = self.data.rule_type == *t;
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded(theme.radius)
                                                .when(is_selected, |this| {
                                                    this.bg(theme.primary)
                                                        .text_color(theme.primary_foreground)
                                                })
                                                .when(!is_selected, |this| {
                                                    this.bg(theme.muted)
                                                        .text_color(theme.muted_foreground)
                                                })
                                                .text_xs()
                                                .child(t.clone())
                                        }),
                                    )),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child("Payload"),
                                    )
                                    .child(
                                        div()
                                            .p_2()
                                            .rounded(theme.radius)
                                            .bg(theme.muted)
                                            .text_sm()
                                            .child(self.data.payload.clone()),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child("Proxy / Proxy Group"),
                                    )
                                    .child(h_flex().flex_wrap().gap_1().children(
                                        self.proxies.iter().map(|p| {
                                            let is_selected = self.data.proxy == *p;
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded(theme.radius)
                                                .when(is_selected, |this| {
                                                    this.bg(theme.primary)
                                                        .text_color(theme.primary_foreground)
                                                })
                                                .when(!is_selected, |this| {
                                                    this.bg(theme.muted)
                                                        .text_color(theme.muted_foreground)
                                                })
                                                .text_xs()
                                                .child(p.clone())
                                        }),
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("cancel").child("Cancel").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_cancel {
                                        handler();
                                    }
                                },
                            ))
                            .child(Button::new("save").primary().child("Save").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_save {
                                        handler(RuleEditData {
                                            rule_type: String::new(),
                                            payload: String::new(),
                                            proxy: String::new(),
                                        });
                                    }
                                },
                            )),
                    ),
            )
    }
}

#[derive(Debug, Clone)]
pub struct FileEditData {
    pub filename: String,
    pub content: String,
    pub language: String,
}

pub struct EditFileModal {
    pub data: FileEditData,
    pub read_only: bool,
    pub on_save: Option<Box<dyn Fn(FileEditData) + 'static>>,
    pub on_cancel: Option<Box<dyn Fn() + 'static>>,
}

impl EditFileModal {
    pub fn new(data: FileEditData) -> Self {
        Self {
            data,
            read_only: false,
            on_save: None,
            on_cancel: None,
        }
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn on_save(mut self, handler: impl Fn(FileEditData) + 'static) -> Self {
        self.on_save = Some(Box::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for EditFileModal {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_save = self.on_save.take();
        let on_cancel = self.on_cancel.take();

        div()
            .fixed()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .z_index(1000)
            .child(
                Card::new()
                    .w(px(700.))
                    .h(px(500.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(format!("Edit: {}", self.data.filename)),
                            )
                            .child(
                                Chip::new()
                                    .xsmall()
                                    .outlined()
                                    .child(self.data.language.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_2()
                            .rounded(theme.radius)
                            .bg(theme.muted)
                            .overflow_scroll()
                            .font_family("monospace")
                            .text_sm()
                            .child(self.data.content.clone()),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("cancel").child("Cancel").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_cancel {
                                        handler();
                                    }
                                },
                            ))
                            .when(!self.read_only, |this| {
                                this.child(Button::new("save").primary().child("Save").on_click(
                                    move |_, _, _| {
                                        if let Some(handler) = &on_save {
                                            handler(FileEditData {
                                                filename: String::new(),
                                                content: String::new(),
                                                language: String::new(),
                                            });
                                        }
                                    },
                                ))
                            }),
                    ),
            )
    }
}
