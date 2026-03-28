use gpui::{
    div, prelude::FluentBuilder, App, ClickEvent, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{
    button::Button, h_flex, v_flex, ActiveTheme, Disableable, Icon, IconName, Sizable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub delay: Option<i32>,
    pub tfo: bool,
    pub udp: bool,
    pub xudp: bool,
    pub mptcp: bool,
    pub smux: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroupInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Vec<ProxyInfo>,
    pub test_url: Option<String>,
    pub fixed: Option<String>,
}

pub struct ProxyItem {
    pub proxy: ProxyInfo,
    pub group: ProxyGroupInfo,
    pub is_selected: bool,
    pub is_fixed: bool,
    pub display_mode: ProxyDisplayMode,
    pub is_testing: bool,
    pub on_select: Option<Box<dyn Fn() + 'static>>,
    pub on_delay_test: Option<Box<dyn Fn() + 'static>>,
    pub on_unpin: Option<Box<dyn Fn() + 'static>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyDisplayMode {
    #[default]
    Simple,
    Full,
}

impl ProxyItem {
    pub fn new(proxy: ProxyInfo, group: ProxyGroupInfo) -> Self {
        Self {
            is_selected: false,
            is_fixed: false,
            display_mode: ProxyDisplayMode::default(),
            is_testing: false,
            on_select: None,
            on_delay_test: None,
            on_unpin: None,
            proxy,
            group,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn fixed(mut self, fixed: bool) -> Self {
        self.is_fixed = fixed;
        self
    }

    pub fn display_mode(mut self, mode: ProxyDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    pub fn testing(mut self, testing: bool) -> Self {
        self.is_testing = testing;
        self
    }

    pub fn on_select(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_delay_test(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_delay_test = Some(Box::new(handler));
        self
    }

    pub fn on_unpin(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_unpin = Some(Box::new(handler));
        self
    }

    fn delay_color(&self, theme: &gpui_component::Theme) -> gpui::Hsla {
        match self.proxy.delay {
            Some(d) if d <= 0 => theme.danger,
            Some(d) if d < 500 => theme.success,
            Some(_) => theme.warning,
            None => theme.muted_foreground,
        }
    }

    fn delay_text(&self) -> String {
        match self.proxy.delay {
            Some(d) if d < 0 => "Test".into(),
            Some(d) if d == 0 => "Timeout".into(),
            Some(d) => format!("{}ms", d),
            None => "Test".into(),
        }
    }

    fn render_protocol_tags(&self, theme: &gpui_component::Theme) -> impl IntoElement {
        let mut tags: Vec<&str> = vec![];

        if self.proxy.tfo {
            tags.push("TFO");
        }
        if self.proxy.udp {
            tags.push("UDP");
        }
        if self.proxy.xudp {
            tags.push("XUDP");
        }
        if self.proxy.mptcp {
            tags.push("MPTCP");
        }
        if self.proxy.smux {
            tags.push("SMUX");
        }

        h_flex().gap_1().children(tags.into_iter().map(|tag| {
            div()
                .px_1()
                .rounded(theme.radius)
                .bg(theme.muted)
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(tag)
        }))
    }
}

impl RenderOnce for ProxyItem {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let delay_color = self.delay_color(theme);
        let delay_text = self.delay_text();
        let proxy_name = self.proxy.name.clone();
        let proxy_type = self.proxy.proxy_type.clone();

        let on_select = self.on_select.take();
        let on_delay_test = self.on_delay_test.take();
        let on_unpin = self.on_unpin.take();

        let border_color = if self.is_fixed {
            theme.secondary
        } else if self.is_selected {
            theme.primary
        } else {
            theme.transparent
        };

        div()
            .id("proxy-item")
            .p_1()
            .border_l_2()
            .border_r_2()
            .border_color(border_color)
            .when(self.display_mode == ProxyDisplayMode::Full, |this| {
                this.child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .whitespace_nowrap()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(proxy_name.clone()),
                                )
                                .child(h_flex().gap_1().when(self.is_fixed, |this| {
                                    this.child(
                                        Button::new("unpin")
                                            .with_size(gpui_component::Size::XSmall)
                                            .icon(Icon::new(IconName::Star))
                                            .on_click(move |_, _, _| {
                                                if let Some(handler) = &on_unpin {
                                                    handler();
                                                }
                                            }),
                                    )
                                })),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .items_center()
                                        .child(
                                            div()
                                                .px_1()
                                                .rounded(theme.radius)
                                                .bg(theme.muted)
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .child(proxy_type),
                                        )
                                        .child(self.render_protocol_tags(theme)),
                                )
                                .child(
                                    Button::new("delay")
                                        .with_size(gpui_component::Size::XSmall)
                                        .when(self.is_testing, |this| this.disabled(true))
                                        .text_color(delay_color)
                                        .child(delay_text.clone())
                                        .on_click(move |_, _, _| {
                                            if let Some(handler) = &on_delay_test {
                                                handler();
                                            }
                                        }),
                                ),
                        ),
                )
            })
            .when(self.display_mode == ProxyDisplayMode::Simple, |this| {
                this.child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .flex_1()
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .child(proxy_name),
                        )
                        .child(
                            h_flex()
                                .gap_1()
                                .when(self.is_fixed, |this| {
                                    this.child(
                                        Button::new("unpin")
                                            .with_size(gpui_component::Size::XSmall)
                                            .icon(Icon::new(IconName::Star)),
                                    )
                                })
                                .child(
                                    Button::new("delay")
                                        .with_size(gpui_component::Size::XSmall)
                                        .when(self.is_testing, |this| this.disabled(true))
                                        .text_color(delay_color)
                                        .child(delay_text),
                                ),
                        ),
                )
            })
            .on_click(move |_: &ClickEvent, _, _| {
                if let Some(handler) = &on_select {
                    handler();
                }
            })
    }
}
