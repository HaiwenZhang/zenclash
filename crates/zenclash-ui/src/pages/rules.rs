use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, App, Context, Entity, FocusHandle, Focusable,
    IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::Button,
    h_flex, v_flex, ActiveTheme,
};
use parking_lot::RwLock;

use zenclash_core::prelude::{CoreManager, RuleItem};

pub struct RulesPage {
    core_manager: Arc<RwLock<CoreManager>>,
    rules: Vec<RuleItem>,
    focus_handle: FocusHandle,
}

impl RulesPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, cx: &mut Context<Self>) -> Self {
        Self {
            core_manager,
            rules: Vec::new(),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let manager = core_manager.read();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if manager.is_running().await {
                        manager.get_rules().await.ok()
                    } else {
                        None
                    }
                })
            });

            if let Some(response) = result {
                let _ = this.update(cx, |this, cx| {
                    this.rules = response.rules;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn update_rules(&mut self, rules: Vec<RuleItem>, cx: &mut Context<Self>) {
        self.rules = rules;
        cx.notify();
    }
}

impl Focusable for RulesPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RulesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Rules"),
                    )
                    .child(
                        Button::new("refresh-rules")
                            .label("Refresh")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} rules loaded", self.rules.len())),
            )
            .child(
                v_flex()
                    .gap_1()
                    .children(self.rules.iter().map(|rule| {
                        div()
                            .p_2()
                            .rounded_md()
                            .bg(cx.theme().list)
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w_24()
                                            .text_sm()
                                            .text_color(cx.theme().primary)
                                            .child(rule.rule_type.clone())
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .child(rule.payload.clone())
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(rule.proxy.clone())
                                    )
                            )
                    })),
            )
    }
}