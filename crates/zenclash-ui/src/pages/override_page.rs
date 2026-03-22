use gpui::{
    div, prelude::FluentBuilder, px, App, Context, Entity, IntoElement, Model, ParentElement,
    Render, Styled, Window,
};
use gpui_component::{
    button::Button, card::Card, h_flex, input::TextInput, switch::Switch, v_flex, ActiveTheme,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Page;

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
    items: Model<Vec<OverrideItem>>,
    new_url: String,
}

impl OverridePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            items: cx.new_model(|_| Vec::new()),
            new_url: String::new(),
        }
    }

    fn render_item(&self, item: &OverrideItem, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        Card::new()
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
                            .child(Switch::new(&item.id).xsmall().checked(item.enabled))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(&item.name),
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
                                Button::new(format!("edit-{}", item.id))
                                    .xsmall()
                                    .child("Edit"),
                            )
                            .child(
                                Button::new(format!("delete-{}", item.id))
                                    .xsmall()
                                    .child("Delete"),
                            ),
                    ),
            )
            .when(item.url.is_some(), |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(item.url.as_ref().unwrap()),
                )
            })
    }
}

impl Page for OverridePage {
    fn title() -> &'static str {
        "Override"
    }

    fn icon() -> gpui_component::icon::IconName {
        gpui_component::icon::IconName::FileCode
    }
}

impl Render for OverridePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let items = self.items.read(cx);

        v_flex()
            .size_full()
            .overflow_y_scroll()
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
                            .child(Button::new("new-js").xsmall().child("New JS"))
                            .child(Button::new("new-yaml").xsmall().child("New YAML"))
                            .child(Button::new("import").xsmall().child("Import")),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div().flex_1().child(
                            TextInput::new(&self.new_url).placeholder("Enter override URL..."),
                        ),
                    )
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
