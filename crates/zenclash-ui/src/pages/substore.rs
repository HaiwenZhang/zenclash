use gpui::{
    div, prelude::FluentBuilder, App, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    tag::Tag,
    v_flex, ActiveTheme, Icon, IconName, Sizable,
};
use serde::{Deserialize, Serialize};

use crate::pages::PageTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubStoreTab {
    #[default]
    Subscriptions,
    Collections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub sub_type: String,
    pub updated: Option<i64>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub subscriptions: Vec<String>,
    pub updated: Option<i64>,
}

pub struct SubStorePage {
    current_tab: SubStoreTab,
    subscriptions: Vec<Subscription>,
    collections: Vec<Collection>,
    server_running: bool,
    server_port: u16,
    focus_handle: FocusHandle,
}

impl SubStorePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            current_tab: SubStoreTab::default(),
            subscriptions: Vec::new(),
            collections: Vec::new(),
            server_running: false,
            server_port: 25500,
            focus_handle: cx.focus_handle(),
        }
    }

    fn render_server_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child("SubStore Server"),
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
                                    .when(self.server_running, |this| {
                                        this.bg(theme.success).text_color(theme.background)
                                    })
                                    .when(!self.server_running, |this| {
                                        this.bg(theme.muted).text_color(theme.muted_foreground)
                                    })
                                    .text_xs()
                                    .child(if self.server_running {
                                        "Running"
                                    } else {
                                        "Stopped"
                                    }),
                            )
                            .child(
                                Button::new("toggle-server")
                                    .with_size(gpui_component::Size::XSmall)
                                    .when(self.server_running, |this| this.child("Stop"))
                                    .when(!self.server_running, |this| {
                                        this.child("Start").primary()
                                    }),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_xs().child("Server URL"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("http://127.0.0.1:{}", self.server_port)),
                    ),
            )
    }

    fn render_tab_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .gap_1()
            .p_1()
            .rounded(theme.radius)
            .bg(theme.muted)
            .children(
                [SubStoreTab::Subscriptions, SubStoreTab::Collections]
                    .into_iter()
                    .map(|tab| {
                        let is_active = self.current_tab == tab;
                        let label = match tab {
                            SubStoreTab::Subscriptions => "Subscriptions",
                            SubStoreTab::Collections => "Collections",
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

    fn render_subscriptions_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(format!("Subscriptions ({})", self.subscriptions.len())),
                    )
                    .child(
                        Button::new("add-sub")
                            .with_size(gpui_component::Size::XSmall)
                            .child("Add Subscription"),
                    ),
            )
            .when(self.subscriptions.is_empty(), |this| {
                this.child(
                    div()
                        .py_8()
                        .text_center()
                        .text_color(theme.muted_foreground)
                        .child("No subscriptions. Add one to get started."),
                )
            })
            .children(self.subscriptions.iter().map(|sub| {
                div()
                    .p_3()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(sub.name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new("edit")
                                            .with_size(gpui_component::Size::XSmall)
                                            .icon(Icon::new(IconName::Settings)),
                                    )
                                    .child(
                                        Button::new("delete")
                                            .with_size(gpui_component::Size::XSmall)
                                            .icon(Icon::new(IconName::Delete))
                                            .danger(),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(
                                Tag::new()
                                    .with_size(gpui_component::Size::XSmall)
                                    .outline()
                                    .child(sub.sub_type.clone()),
                            )
                            .when_some(sub.updated, |this, ts| {
                                let dt = chrono::DateTime::from_timestamp(ts, 0);
                                this.child(
                                    div().child(
                                        dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                                            .unwrap_or_default(),
                                    ),
                                )
                            }),
                    )
            }))
    }

    fn render_collections_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(format!("Collections ({})", self.collections.len())),
                    )
                    .child(
                        Button::new("add-col")
                            .with_size(gpui_component::Size::XSmall)
                            .child("Add Collection"),
                    ),
            )
            .when(self.collections.is_empty(), |this| {
                this.child(
                    div()
                        .py_8()
                        .text_center()
                        .text_color(theme.muted_foreground)
                        .child("No collections. Create one to group subscriptions."),
                )
            })
            .children(self.collections.iter().map(|col| {
                div()
                    .p_3()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(col.name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new("edit")
                                            .with_size(gpui_component::Size::XSmall)
                                            .icon(Icon::new(IconName::Settings)),
                                    )
                                    .child(
                                        Button::new("delete")
                                            .with_size(gpui_component::Size::XSmall)
                                            .icon(Icon::new(IconName::Delete))
                                            .danger(),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("{} subscriptions", col.subscriptions.len())),
                    )
            }))
    }
}

impl PageTrait for SubStorePage {
    fn title() -> &'static str {
        "SubStore"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Folder
    }
}

impl Focusable for SubStorePage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SubStorePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .overflow_y_hidden()
            .gap_4()
            .p_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("SubStore"),
            )
            .child(self.render_server_section(cx))
            .child(self.render_tab_selector(cx))
            .when(self.current_tab == SubStoreTab::Subscriptions, |this| {
                this.child(self.render_subscriptions_section(cx))
            })
            .when(self.current_tab == SubStoreTab::Collections, |this| {
                this.child(self.render_collections_section(cx))
            })
    }
}
