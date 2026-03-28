use gpui::{
    div, prelude::FluentBuilder, px, App, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{h_flex, v_flex, ActiveTheme, Icon, IconName};
use zenclash_core::prelude::{format_speed, CoreState};

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
    upload_speed: u64,
    download_speed: u64,
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
            upload_speed: 0,
            download_speed: 0,
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

    pub fn traffic(mut self, upload: u64, download: u64) -> Self {
        self.upload_speed = upload;
        self.download_speed = download;
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
        let width = px(if self.collapsed { 48. } else { 220. });
        let is_running = self.core_state == CoreState::Running;
        let current_page = self.current_page;
        let collapsed = self.collapsed;

        v_flex()
            .w(width)
            .h_full()
            .bg(theme.background)
            .border_r_1()
            .border_color(theme.border)
            .child(Self::render_header(&theme, collapsed))
            .child(Self::render_core_status(&theme, is_running, collapsed))
            .child(Self::render_nav(&theme, current_page, collapsed))
            .child(Self::render_footer(
                &theme,
                collapsed,
                self.upload_speed,
                self.download_speed,
            ))
    }
}

impl ZenSidebar {
    fn render_header(theme: &gpui_component::Theme, collapsed: bool) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap_2()
            .p_2()
            .border_b_1()
            .border_color(theme.border)
            .child(Icon::new(IconName::Star).size_5().text_color(theme.primary))
            .when(!collapsed, |this| {
                this.child(
                    v_flex()
                        .gap_0()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("ZenClash"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("Proxy Manager"),
                        ),
                )
            })
    }

    fn render_core_status(
        theme: &gpui_component::Theme,
        is_running: bool,
        collapsed: bool,
    ) -> impl IntoElement {
        let state_color = if is_running {
            theme.success
        } else {
            theme.muted_foreground
        };

        h_flex()
            .items_center()
            .gap_2()
            .p_2()
            .border_b_1()
            .border_color(theme.border)
            .child(div().size_2().rounded_full().bg(state_color))
            .when(!collapsed, |this| {
                this.child(div().flex_1().text_xs().child(if is_running {
                    "Running"
                } else {
                    "Stopped"
                }))
            })
    }

    fn render_nav(
        theme: &gpui_component::Theme,
        current_page: Page,
        collapsed: bool,
    ) -> impl IntoElement {
        let pages = [
            Page::Dashboard,
            Page::Proxies,
            Page::Profiles,
            Page::Connections,
            Page::Rules,
            Page::Logs,
            Page::Mihomo,
            Page::Tun,
            Page::Sniffer,
            Page::Dns,
            Page::Resources,
            Page::Override,
            Page::Sysproxy,
            Page::Backup,
            Page::SubStore,
            Page::Settings,
        ];

        v_flex()
            .flex_1()
            .gap_0()
            .overflow_hidden()
            .p_1()
            .children(pages.iter().map(move |page| {
                let is_active = *page == current_page;
                let page_label = page.label();
                let page_for_click = *page;

                let bg = if is_active {
                    theme.primary
                } else {
                    theme.transparent
                };
                let text_color = if is_active {
                    theme.primary_foreground
                } else {
                    theme.foreground
                };

                div()
                    .id(page_label)
                    .w_full()
                    .px_2()
                    .py_1()
                    .rounded(theme.radius)
                    .cursor_pointer()
                    .bg(bg)
                    .text_color(text_color)
                    .hover(|this| {
                        if !is_active {
                            this.bg(theme.muted)
                        } else {
                            this
                        }
                    })
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Icon::new(page.icon()).size_4())
                            .when(!collapsed, |this| {
                                this.child(div().text_xs().child(page_label))
                            }),
                    )
                    .on_click(move |_, window, cx| {
                        dispatch_navigate(page_for_click, window, cx);
                    })
            }))
    }

    fn render_footer(
        theme: &gpui_component::Theme,
        collapsed: bool,
        upload_speed: u64,
        download_speed: u64,
    ) -> impl IntoElement {
        h_flex()
            .items_center()
            .justify_center()
            .gap_2()
            .p_2()
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .when(collapsed, |this| {
                this.child(
                    v_flex()
                        .gap_1()
                        .items_center()
                        .child(div().text_color(gpui::rgb(0x4ade80)).child("↑"))
                        .child(div().text_color(gpui::rgb(0x60a5fa)).child("↓")),
                )
            })
            .when(!collapsed, |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .flex_1()
                        .child(
                            div()
                                .text_color(gpui::rgb(0x4ade80))
                                .child(format!("↑ {}", format_speed(upload_speed))),
                        )
                        .child(
                            div()
                                .text_color(gpui::rgb(0x60a5fa))
                                .child(format!("↓ {}", format_speed(download_speed))),
                        ),
                )
            })
    }
}

fn dispatch_navigate(page: Page, window: &mut Window, cx: &mut App) {
    use crate::app::*;
    match page {
        Page::Dashboard => window.dispatch_action(Box::new(NavigateDashboard), cx),
        Page::Proxies => window.dispatch_action(Box::new(NavigateProxies), cx),
        Page::Profiles => window.dispatch_action(Box::new(NavigateProfiles), cx),
        Page::Connections => window.dispatch_action(Box::new(NavigateConnections), cx),
        Page::Rules => window.dispatch_action(Box::new(NavigateRules), cx),
        Page::Logs => window.dispatch_action(Box::new(NavigateLogs), cx),
        Page::Mihomo => window.dispatch_action(Box::new(NavigateMihomo), cx),
        Page::Tun => window.dispatch_action(Box::new(NavigateTun), cx),
        Page::Sniffer => window.dispatch_action(Box::new(NavigateSniffer), cx),
        Page::Dns => window.dispatch_action(Box::new(NavigateDns), cx),
        Page::Resources => window.dispatch_action(Box::new(NavigateResources), cx),
        Page::Override => window.dispatch_action(Box::new(NavigateOverride), cx),
        Page::Sysproxy => window.dispatch_action(Box::new(NavigateSysproxy), cx),
        Page::Backup => window.dispatch_action(Box::new(NavigateBackup), cx),
        Page::SubStore => window.dispatch_action(Box::new(NavigateSubStore), cx),
        Page::Settings => window.dispatch_action(Box::new(NavigateSettings), cx),
    }
}
