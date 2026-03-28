use gpui::{
    div, prelude::FluentBuilder, AppContext, Context, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    switch::Switch,
    v_flex, ActiveTheme, Sizable,
};
use serde::{Deserialize, Serialize};

use crate::pages::PageTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OverrideType {
    #[default]
    Js,
    Yaml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: OverrideType,
    pub enabled: bool,
    pub file: Option<String>,
    pub url: Option<String>,
}

impl OverrideItem {
    pub fn new_js(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            item_type: OverrideType::Js,
            enabled: true,
            file: None,
            url: None,
        }
    }

    pub fn new_yaml(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            item_type: OverrideType::Yaml,
            enabled: true,
            file: None,
            url: None,
        }
    }
}

pub struct OverridePage {
    items: Entity<Vec<OverrideItem>>,
    new_url: String,
}

impl OverridePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            items: cx.new(|_| Vec::new()),
            new_url: String::new(),
        }
    }

    fn render_item(&self, item: &OverrideItem, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .p_3()
            .gap_2()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Switch::new(SharedString::from(item.id.clone()))
                                    .with_size(gpui_component::Size::XSmall)
                                    .checked(item.enabled),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(div().child(item.name.clone())),
                            )
                            .child(
                                div()
                                    .px_1()
                                    .rounded(theme.radius)
                                    .bg(if item.item_type == OverrideType::Js {
                                        theme.warning
                                    } else {
                                        theme.success
                                    })
                                    .text_xs()
                                    .text_color(theme.background)
                                    .child(if item.item_type == OverrideType::Js {
                                        "JS"
                                    } else {
                                        "YAML"
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new(SharedString::from(format!("edit-{}", item.id)))
                                    .with_size(gpui_component::Size::XSmall)
                                    .child("Edit"),
                            )
                            .child(
                                Button::new(SharedString::from(format!("delete-{}", item.id)))
                                    .with_size(gpui_component::Size::XSmall)
                                    .child("Delete"),
                            ),
                    ),
            )
            .when(item.url.is_some(), |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(item.url.as_ref().unwrap().clone()),
                )
            })
    }
}

impl PageTrait for OverridePage {
    fn title() -> &'static str {
        "Override"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::File
    }
}

impl Render for OverridePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let items = self.items.read(cx).clone();

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
                            .child("Override Management"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("new-js")
                                    .with_size(gpui_component::Size::XSmall)
                                    .child("New JS"),
                            )
                            .child(
                                Button::new("new-yaml")
                                    .with_size(gpui_component::Size::XSmall)
                                    .child("New YAML"),
                            )
                            .child(
                                Button::new("import")
                                    .with_size(gpui_component::Size::XSmall)
                                    .child("Import"),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(div().flex_1().child(self.new_url.clone()))
                    .child(Button::new("import-url").child("Import URL").primary()),
            )
            .when(items.is_empty(), |this| {
                this.child(
                    div().flex_1().items_center().justify_center().child(
                        div()
                            .text_color(theme.muted_foreground)
                            .child("No override items. Create or import one to get started."),
                    ),
                )
            })
            .when(!items.is_empty(), |this| {
                this.child(
                    v_flex()
                        .gap_2()
                        .children(items.iter().map(|item| self.render_item(item, cx))),
                )
            })
    }
}
