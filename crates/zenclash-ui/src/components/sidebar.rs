use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, IntoElement, RenderOnce, Styled, Window,
};
use gpui_component::{
    h_flex,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    switch::Switch,
    v_flex, ActiveTheme, Icon, IconName, Sizable,
};
use std::rc::Rc;

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

pub struct ZenSidebar {
    current_page: Page,
    collapsed: bool,
    sysproxy_enabled: bool,
    tun_enabled: bool,
    outbound_mode: OutboundMode,
    on_navigate: Option<Rc<dyn Fn(Page)>>,
    on_toggle_sysproxy: Option<Rc<dyn Fn(bool)>>,
    on_toggle_tun: Option<Rc<dyn Fn(bool)>>,
    on_change_outbound_mode: Option<Rc<dyn Fn(OutboundMode)>>,
}

impl ZenSidebar {
    pub fn new() -> Self {
        Self {
            current_page: Page::default(),
            collapsed: false,
            sysproxy_enabled: false,
            tun_enabled: false,
            outbound_mode: OutboundMode::default(),
            on_navigate: None,
            on_toggle_sysproxy: None,
            on_toggle_tun: None,
            on_change_outbound_mode: None,
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

    pub fn on_navigate(mut self, handler: impl Fn(Page) + 'static) -> Self {
        self.on_navigate = Some(Rc::new(handler));
        self
    }

    pub fn on_toggle_sysproxy(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.on_toggle_sysproxy = Some(Rc::new(handler));
        self
    }

    pub fn on_toggle_tun(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.on_toggle_tun = Some(Rc::new(handler));
        self
    }

    pub fn on_change_outbound_mode(mut self, handler: impl Fn(OutboundMode) + 'static) -> Self {
        self.on_change_outbound_mode = Some(Rc::new(handler));
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
        let on_navigate = self.on_navigate.clone();
        let on_toggle_sysproxy = self.on_toggle_sysproxy.clone();
        let on_toggle_tun = self.on_toggle_tun.clone();
        let on_change_mode = self.on_change_outbound_mode.clone();

        let pages = [
            Page::Proxies,
            Page::Profiles,
            Page::Connections,
            Page::Rules,
            Page::Logs,
            Page::Settings,
        ];

        Sidebar::new("zen-sidebar")
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
                                this.child(Icon::new(IconName::Zap).size_4())
                            })
                            .when(self.collapsed, |this| {
                                this.size_4()
                                    .bg(theme.transparent)
                                    .text_color(theme.foreground)
                                    .child(Icon::new(IconName::Zap).size_4())
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
            .child(
                v_flex()
                    .gap_2()
                    .p_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Icon::new(IconName::Globe)
                                            .size_4()
                                            .text_color(theme.muted_foreground),
                                    )
                                    .when(!self.collapsed, |this| this.child("System Proxy")),
                            )
                            .child(
                                Switch::new("sysproxy")
                                    .xsmall()
                                    .checked(self.sysproxy_enabled)
                                    .on_click(move |checked, _, _| {
                                        if let Some(handler) = &on_toggle_sysproxy {
                                            handler(*checked);
                                        }
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Icon::new(IconName::Route)
                                            .size_4()
                                            .text_color(theme.muted_foreground),
                                    )
                                    .when(!self.collapsed, |this| this.child("TUN Mode")),
                            )
                            .child(
                                Switch::new("tun")
                                    .xsmall()
                                    .checked(self.tun_enabled)
                                    .on_click(move |checked, _, _| {
                                        if let Some(handler) = &on_toggle_tun {
                                            handler(*checked);
                                        }
                                    }),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .p_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .when(!self.collapsed, |this| {
                        this.child(
                            div()
                                .px_2()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("Outbound Mode"),
                        )
                    })
                    .child(
                        h_flex().gap_1().px_1().children(
                            [
                                OutboundMode::Rule,
                                OutboundMode::Global,
                                OutboundMode::Direct,
                            ]
                            .map(|mode| {
                                let is_active = self.outbound_mode == mode;
                                let on_change = on_change_mode.clone();
                                div()
                                    .flex_1()
                                    .items_center()
                                    .justify_center()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(is_active, |this| {
                                        this.bg(theme.secondary)
                                            .text_color(theme.secondary_foreground)
                                    })
                                    .when(!is_active, |this| {
                                        this.text_color(theme.muted_foreground)
                                            .hover(|this| this.bg(theme.muted))
                                    })
                                    .text_xs()
                                    .child(mode.label())
                                    .on_click(move |_: &ClickEvent, _, _| {
                                        if let Some(handler) = &on_change {
                                            handler(mode);
                                        }
                                    })
                            }),
                        ),
                    ),
            )
            .child(
                SidebarGroup::new("").child(SidebarMenu::new().children(pages.into_iter().map(
                    |page| {
                        let is_active = self.current_page == page;
                        let on_nav = on_navigate.clone();
                        SidebarMenuItem::new(page.label())
                            .icon(page.icon())
                            .active(is_active)
                            .on_click(move |_window, _cx| {
                                if let Some(handler) = &on_nav {
                                    handler(page);
                                }
                            })
                    },
                ))),
            )
            .footer(
                SidebarFooter::new().child(
                    SidebarMenuItem::new("Settings")
                        .icon(IconName::Settings)
                        .active(self.current_page == Page::Settings)
                        .on_click(move |_window, _cx| {
                            if let Some(handler) = &on_navigate {
                                handler(Page::Settings);
                            }
                        }),
                ),
            )
    }
}
