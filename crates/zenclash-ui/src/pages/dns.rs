use gpui::{
    div, App, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    Window,
};
use gpui_component::{
    button::Button,
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectState},
    switch::Switch,
    v_flex, ActiveTheme,
};

use zenclash_core::{DnsConfig, DnsMode, FakeIpFilter, FallbackFilter};

pub struct DnsPage {
    config: DnsConfig,
    new_nameserver: Entity<InputState>,
    new_fallback: Entity<InputState>,
    focus_handle: FocusHandle,
}

impl DnsPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let new_nameserver = cx.new(|cx| InputState::new(window, cx).placeholder("8.8.8.8"));
        let new_fallback = cx.new(|cx| InputState::new(window, cx).placeholder("tls://dns.google"));

        Self {
            config: DnsConfig::default(),
            new_nameserver,
            new_fallback,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn update_config(&mut self, config: DnsConfig, cx: &mut Context<Self>) {
        self.config = config;
        cx.notify();
    }

    pub fn add_nameserver(&mut self, cx: &mut Context<Self>) {
        let server = self.new_nameserver.read(cx).text().to_string();
        if !server.is_empty() {
            self.config.nameservers.push(server);
            self.new_nameserver.update(cx, |state, _| {
                state.set_text("");
            });
            cx.notify();
        }
    }

    pub fn remove_nameserver(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.config.nameservers.len() {
            self.config.nameservers.remove(index);
            cx.notify();
        }
    }

    pub fn add_fallback(&mut self, cx: &mut Context<Self>) {
        let server = self.new_fallback.read(cx).text().to_string();
        if !server.is_empty() {
            self.config.fallback.push(server);
            self.new_fallback.update(cx, |state, _| {
                state.set_text("");
            });
            cx.notify();
        }
    }

    pub fn remove_fallback(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.config.fallback.len() {
            self.config.fallback.remove(index);
            cx.notify();
        }
    }

    pub fn toggle_dns(&mut self, cx: &mut Context<Self>) {
        self.config.enable = !self.config.enable;
        cx.notify();
    }

    pub fn toggle_ipv6(&mut self, cx: &mut Context<Self>) {
        self.config.ipv6 = !self.config.ipv6;
        cx.notify();
    }

    pub fn set_mode(&mut self, mode: DnsMode, cx: &mut Context<Self>) {
        self.config.mode = mode;
        cx.notify();
    }
}

impl Focusable for DnsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DnsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("DNS Settings"),
            )
            .child(
                v_flex()
                    .gap_4()
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Enable DNS"))
                            .child(
                                Switch::new("enable-dns", self.config.enable.into()).on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.toggle_dns(cx);
                                    }),
                                ),
                            ),
                    )
                    .child(h_flex().gap_4().child(div().w_32().child("IPv6")).child(
                        Switch::new("enable-ipv6", self.config.ipv6.into()).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.toggle_ipv6(cx);
                            },
                        )),
                    ))
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Listen"))
                            .child(div().child(self.config.listen.clone())),
                    )
                    .child(
                        h_flex().gap_4().child(div().w_32().child("Mode")).child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("mode-normal")
                                        .when(self.config.mode == DnsMode::Normal, |this| {
                                            this.primary()
                                        })
                                        .label("Normal")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_mode(DnsMode::Normal, cx);
                                        })),
                                )
                                .child(
                                    Button::new("mode-fakeip")
                                        .when(self.config.mode == DnsMode::FakeIp, |this| {
                                            this.primary()
                                        })
                                        .label("Fake-IP")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_mode(DnsMode::FakeIp, cx);
                                        })),
                                )
                                .child(
                                    Button::new("mode-redir")
                                        .when(self.config.mode == DnsMode::RedirHost, |this| {
                                            this.primary()
                                        })
                                        .label("Redir-Host")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_mode(DnsMode::RedirHost, cx);
                                        })),
                                ),
                        ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Name Servers"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Input::new(&self.new_nameserver, window, cx))
                            .child(Button::new("add-nameserver").label("Add").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.add_nameserver(cx);
                                }),
                            )),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .children(self.config.nameservers.iter().enumerate().map(
                                |(i, server)| {
                                    h_flex()
                                        .gap_2()
                                        .child(div().flex_1().child(server.clone()))
                                        .child(
                                            Button::new(format!("remove-ns-{}", i))
                                                .ghost()
                                                .label("Remove")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.remove_nameserver(i, cx);
                                                })),
                                        )
                                },
                            )),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Fallback Servers"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Input::new(&self.new_fallback, window, cx))
                            .child(
                                Button::new("add-fallback")
                                    .label("Add")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_fallback(cx);
                                    })),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .children(self.config.fallback.iter().enumerate().map(
                                |(i, server)| {
                                    h_flex()
                                        .gap_2()
                                        .child(div().flex_1().child(server.clone()))
                                        .child(
                                            Button::new(format!("remove-fb-{}", i))
                                                .ghost()
                                                .label("Remove")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.remove_fallback(i, cx);
                                                })),
                                        )
                                },
                            )),
                    ),
            )
            .child(
                Button::new("save-dns")
                    .primary()
                    .label("Save DNS Settings")
                    .on_click(cx.listener(|_, _, _, _| {})),
            )
    }
}
