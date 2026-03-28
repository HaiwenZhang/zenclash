use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, App, Context, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex, v_flex, ActiveTheme, Disableable, Icon, IconName,
};
use parking_lot::RwLock;

use crate::components::sidebar_cards::CoreInfo;
use zenclash_core::prelude::{format_speed, format_traffic, CoreManager, CoreState};

pub struct DashboardPage {
    core_manager: Arc<RwLock<CoreManager>>,
    core_state: CoreState,
    core_info: Option<CoreInfo>,
    connection_count: usize,
    upload_speed: u64,
    download_speed: u64,
    profile_name: Option<String>,
    traffic_used: u64,
    traffic_total: u64,
    rule_count: usize,
    focus_handle: FocusHandle,
}

impl DashboardPage {
    pub fn new(
        core_manager: Arc<RwLock<CoreManager>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let view = Self {
            core_manager,
            core_state: CoreState::Stopped,
            core_info: None,
            connection_count: 0,
            upload_speed: 0,
            download_speed: 0,
            profile_name: None,
            traffic_used: 0,
            traffic_total: 0,
            rule_count: 0,
            focus_handle: cx.focus_handle(),
        };
        view.start_polling(cx);
        view
    }

    fn start_polling(&self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                let manager = core_manager.read();
                let is_running = manager.is_running().await;

                let (connection_count, rule_count) = if is_running {
                    let conn_count = manager.get_connections().await.map(|c| c.connections.len()).unwrap_or(0);
                    let rule_count = manager.get_rules().await.map(|r| r.rules.len()).unwrap_or(0);
                    (conn_count, rule_count)
                } else {
                    (0, 0)
                };

                let _ = this.update(cx, |this, cx| {
                    this.core_state = if is_running {
                        CoreState::Running
                    } else {
                        CoreState::Stopped
                    };
                    this.connection_count = connection_count;
                    this.rule_count = rule_count;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn update_core_state(&mut self, state: CoreState, cx: &mut Context<Self>) {
        self.core_state = state;
        cx.notify();
    }

    pub fn update_profile(
        &mut self,
        name: String,
        used: u64,
        total: u64,
        cx: &mut Context<Self>,
    ) {
        self.profile_name = Some(name);
        self.traffic_used = used;
        self.traffic_total = total;
        cx.notify();
    }

    fn render_core_card(
        &self,
        theme: &gpui_component::Theme,
        is_running: bool,
    ) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(200.))
            .p_4()
            .rounded(theme.radius)
            .bg(theme.muted.opacity(0.5))
            .border_1()
            .border_color(theme.border)
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(if is_running {
                                        theme.success
                                    } else {
                                        theme.muted_foreground
                                    }),
                            )
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(if is_running { "Running" } else { "Stopped" }),
                            ),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Core")
                            .child(if let Some(info) = &self.core_info {
                                format_traffic(info.memory)
                            } else {
                                "-".to_string()
                            }),
                    ),
            )
    }

    fn render_profile_card(&self, theme: &gpui_component::Theme) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(200.))
            .p_4()
            .rounded(theme.radius)
            .bg(theme.muted.opacity(0.5))
            .border_1()
            .border_color(theme.border)
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child(
                                self.profile_name
                                    .clone()
                                    .unwrap_or_else(|| "No Profile".into()),
                            ),
                    )
                    .when(self.traffic_total > 0, |this| {
                        this.child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!(
                                    "{}/{}",
                                    format_traffic(self.traffic_used),
                                    format_traffic(self.traffic_total)
                                ))
                                .child("-"),
                        )
                    }),
            )
    }

    fn render_connection_card(&self, theme: &gpui_component::Theme) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(200.))
            .p_4()
            .rounded(theme.radius)
            .bg(theme.muted.opacity(0.5))
            .border_1()
            .border_color(theme.border)
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .child(format!("{} Connections", self.connection_count)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!(
                                        "↑{} ↓{}",
                                        format_speed(self.upload_speed),
                                        format_speed(self.download_speed)
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Icon::new(IconName::ExternalLink)
                                    .size_4()
                                    .text_color(theme.muted_foreground),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child("Active connections"),
                            ),
                    ),
            )
    }

    fn render_rule_card(&self, theme: &gpui_component::Theme) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(200.))
            .p_4()
            .rounded(theme.radius)
            .bg(theme.muted.opacity(0.5))
            .border_1()
            .border_color(theme.border)
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Icon::new(IconName::Menu).size_4())
                            .child(
                                div().text_xs().child(format!("{} Rules", self.rule_count)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Routing rules loaded"),
                    ),
            )
    }

    fn render_status_cards(
        &self,
        theme: &gpui_component::Theme,
        is_running: bool,
    ) -> impl IntoElement {
        h_flex()
            .gap_4()
            .flex_wrap()
            .child(self.render_core_card(theme, is_running))
            .child(self.render_profile_card(theme))
            .child(self.render_connection_card(theme))
            .child(self.render_rule_card(theme))
    }

    fn render_quick_actions(
        &self,
        _theme: &gpui_component::Theme,
        is_running: bool,
    ) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Quick Actions"),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        Button::new("test-delay-btn")
                            .child("Test All Delays")
                            .primary()
                            .when(!is_running, |this| this.disabled(true)),
                    )
                    .child(
                        Button::new("update-profile-btn")
                            .child("Update Profile")
                            .when(self.profile_name.is_none(), |this| this.disabled(true)),
                    )
                    .child(
                        Button::new("restart-core-btn")
                            .child("Restart Core")
                            .when(!is_running, |this| this.disabled(true)),
                    ),
            )
    }
}

impl Focusable for DashboardPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DashboardPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_running = self.core_state == CoreState::Running;

        v_flex()
            .size_full()
            .gap_6()
            .p_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Dashboard"),
            )
            .child(self.render_status_cards(&theme, is_running))
            .child(self.render_quick_actions(&theme, is_running))
    }
}