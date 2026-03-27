use gpui::{
    div, prelude::FluentBuilder, px, App, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{
    h_flex,
    sidebar::{Sidebar, SidebarFooter, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex, ActiveTheme, Icon, IconName, Side,
};
use zenclash_core::prelude::CoreState;

use crate::app::{StartCore, StopCore};
use crate::pages::Page;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutboundMode {
    #[default]
    Rule,
    Global,
    Direct,
}

impl OutboundMode {
    pub fn label(&self) -> &'static str {
        match self {
            OutboundMode::Rule => "Rule",
            OutboundMode::Global => "Global",
            OutboundMode::Direct => "Direct",
        }
    }
}

#[derive(IntoElement)]
pub struct ZenSidebar {
    current_page: Page,
    collapsed: bool,
    sysproxy_enabled: bool,
    tun_enabled: bool,
    outbound_mode: OutboundMode,
    core_state: CoreState,
}

impl ZenSidebar {
    pub fn new() -> Self {
        Self {
            current_page: Page::default(),
            collapsed: false,
            sysproxy_enabled: false,
            tun_enabled: false,
            outbound_mode: OutboundMode::default(),
            core_state: CoreState::Stopped,
        }
    }

    pub fn current_page(mut self, page: Page) -> Self {
        self.current_page = page;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn sysproxy_enabled(mut self, enabled: bool) -> Self {
        self.sysproxy_enabled = enabled;
        self
    }

    pub fn tun_enabled(mut self, enabled: bool) -> Self {
        self.tun_enabled = enabled;
        self
    }

    pub fn outbound_mode(mut self, mode: OutboundMode) -> Self {
        self.outbound_mode = mode;
        self
    }

    pub fn core_state(mut self, state: CoreState) -> Self {
        self.core_state = state;
        self
    }
}

impl Default for ZenSidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ZenSidebar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let pages = [
            Page::Proxies,
            Page::Profiles,
            Page::Connections,
            Page::Rules,
            Page::Logs,
            Page::Settings,
        ];

        let is_running = self.core_state == CoreState::Running;
        let state_color = match self.core_state {
            CoreState::Running => theme.success,
            CoreState::Starting => theme.warning,
            CoreState::Stopping => theme.warning,
            CoreState::Error => theme.danger,
            CoreState::Stopped => theme.muted_foreground,
        };
        let state_text = match self.core_state {
            CoreState::Running => "Running",
            CoreState::Starting => "Starting...",
            CoreState::Stopping => "Stopping...",
            CoreState::Error => "Error",
            CoreState::Stopped => "Stopped",
        };

        Sidebar::<SidebarMenu>::new(Side::Left)
            .collapsed(self.collapsed)
            .w(px(if self.collapsed { 48. } else { 220. }))
            .gap_0()
            .header(
                SidebarHeader::new()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_center()
                            .rounded(theme.radius)
                            .bg(theme.primary)
                            .text_color(theme.primary_foreground)
                            .size_8()
                            .flex_shrink_0()
                            .when(!self.collapsed, |this| {
                                this.child(Icon::new(IconName::Star).size_4())
                            })
                            .when(self.collapsed, |this| {
                                this.size_4()
                                    .bg(theme.transparent)
                                    .text_color(theme.foreground)
                                    .child(Icon::new(IconName::Star).size_4())
                            }),
                    )
                    .when(!self.collapsed, |this| {
                        this.child(
                            v_flex()
                                .gap_0()
                                .text_sm()
                                .flex_1()
                                .line_height(gpui::relative(1.25))
                                .overflow_hidden()
                                .text_ellipsis()
                                .child("ZenClash")
                                .child(
                                    div()
                                        .child("Proxy Manager")
                                        .text_xs()
                                        .text_color(theme.muted_foreground),
                                ),
                        )
                    }),
            )
            .footer(
                SidebarFooter::new()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .p_2()
                            .child(div().size_2().rounded_full().bg(state_color))
                            .when(!self.collapsed, |this| {
                                this.child(div().flex_1().text_xs().child(state_text))
                                    .child(
                                        h_flex().gap_1().child(
                                            div()
                                                .id("toggle-core-btn")
                                                .px_2()
                                                .py_1()
                                                .rounded(theme.radius)
                                                .cursor_pointer()
                                                .bg(if is_running {
                                                    theme.danger
                                                } else {
                                                    theme.primary
                                                })
                                                .text_color(if is_running {
                                                    theme.background
                                                } else {
                                                    theme.primary_foreground
                                                })
                                                .text_xs()
                                                .child(if is_running { "Stop" } else { "Start" })
                                                .on_click(move |_, window, cx| {
                                                    if is_running {
                                                        window.dispatch_action(
                                                            Box::new(StopCore),
                                                            cx,
                                                        );
                                                    } else {
                                                        window.dispatch_action(
                                                            Box::new(StartCore),
                                                            cx,
                                                        );
                                                    }
                                                }),
                                        ),
                                    )
                            }),
                    )
                    .child(SidebarMenu::new().children(pages.into_iter().map(|page| {
                        let is_active = self.current_page == page;
                        SidebarMenuItem::new(page.label())
                            .icon(page.icon())
                            .active(is_active)
                    }))),
            )
    }
}
