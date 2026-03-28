use gpui::{
    div, prelude::FluentBuilder, App, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};
use gpui_component::{
    button::Button, h_flex, switch::Switch, tag::Tag, ActiveTheme, Icon, IconName, Sizable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInfo {
    pub id: String,
    pub rule_type: String,
    pub payload: String,
    pub proxy: String,
    pub enabled: bool,
    pub hits: Option<u64>,
}

pub struct RuleItem {
    pub info: RuleInfo,
    pub on_toggle: Option<Box<dyn Fn(bool) + 'static>>,
    pub on_delete: Option<Box<dyn Fn() + 'static>>,
}

impl RuleItem {
    pub fn new(info: RuleInfo) -> Self {
        Self {
            info,
            on_toggle: None,
            on_delete: None,
        }
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_delete(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_delete = Some(Box::new(handler));
        self
    }

    fn rule_color(&self, theme: &gpui_component::Theme) -> gpui::Hsla {
        match self.info.rule_type.as_str() {
            "DOMAIN" | "DOMAIN-SUFFIX" | "DOMAIN-KEYWORD" => theme.primary,
            "IP-CIDR" | "IP-CIDR6" | "SRC-IP-CIDR" => theme.warning,
            "GEOIP" | "GEOSITE" => theme.success,
            "PROCESS-NAME" | "PROCESS-PATH" => theme.secondary,
            "DST-PORT" | "SRC-PORT" => theme.info,
            "MATCH" => theme.danger,
            _ => theme.muted_foreground,
        }
    }
}

impl RenderOnce for RuleItem {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let _rule_color = self.rule_color(theme);

        let on_toggle = self.on_toggle.take();
        let on_delete = self.on_delete.take();

        div()
            .p_3()
            .gap_2()
            .when(!self.info.enabled, |this| this.opacity(0.5))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        Switch::new(SharedString::from(self.info.id.clone()))
                            .with_size(gpui_component::Size::XSmall)
                            .checked(self.info.enabled)
                            .on_click(move |checked, _, _| {
                                if let Some(handler) = &on_toggle {
                                    handler(*checked);
                                }
                            }),
                    )
                    .child(
                        Tag::new()
                            .with_size(gpui_component::Size::XSmall)
                            .outline()
                            .child(self.info.rule_type.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .text_sm()
                            .child(self.info.payload.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("→ {}", self.info.proxy)),
                    )
                    .when_some(self.info.hits, |this, hits| {
                        this.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded(theme.radius)
                                .bg(theme.muted)
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("{} hits", hits)),
                        )
                    })
                    .child(
                        Button::new(SharedString::from(format!("delete-{}", self.info.id)))
                            .with_size(gpui_component::Size::XSmall)
                            .icon(Icon::new(IconName::Delete))
                            .on_click(move |_, _, _| {
                                if let Some(handler) = &on_delete {
                                    handler();
                                }
                            }),
                    ),
            )
    }
}
