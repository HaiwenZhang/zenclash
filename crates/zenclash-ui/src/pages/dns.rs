use gpui::{
    div, prelude::FluentBuilder, App, AppContext, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectState},
    switch::Switch,
    v_flex, ActiveTheme,
};

use zenclash_core::prelude::{DnsConfig, FallbackFilter};

pub struct DnsPage {
    config: DnsConfig,
    new_nameserver: Entity<InputState>,
    new_fallback: Entity<InputState>,
    focus_handle: FocusHandle,
}

impl DnsPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            config: DnsConfig::default(),
            new_nameserver: cx.new(|cx| InputState::new(window, cx)),
            new_fallback: cx.new(|cx| InputState::new(window, cx)),
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
            self.config.nameserver.push(server);
            self.new_nameserver.update(cx, |state, _| {});
            cx.notify();
        }
    }

    pub fn remove_nameserver(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.config.nameserver.len() {
            self.config.nameserver.remove(index);
            cx.notify();
        }
    }

    pub fn add_fallback(&mut self, cx: &mut Context<Self>) {
        let server = self.new_fallback.read(cx).text().to_string();
        if !server.is_empty() {
            self.config
                .fallback
                .get_or_insert_with(Vec::new)
                .push(server);
            cx.notify();
        }
    }

    pub fn remove_fallback(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(ref mut fallback) = self.config.fallback {
            if index < fallback.len() {
                fallback.remove(index);
                cx.notify();
            }
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

    pub fn set_mode(&mut self, mode: String, cx: &mut Context<Self>) {
        self.config.enhanced_mode = mode;
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
                                Switch::new("enable-dns")
                                    .checked(self.config.enable)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_dns(cx);
                                    })),
                            ),
                    )
                    .child(
                        h_flex().gap_4().child(div().w_32().child("IPv6")).child(
                            Switch::new("enable-ipv6")
                                .checked(self.config.ipv6)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_ipv6(cx);
                                })),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Listen"))
                            .child(div().child(self.config.listen.clone().unwrap_or_default())),
                    )
                    .child(
                        h_flex().gap_4().child(div().w_32().child("Mode")).child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("mode-normal")
                                        .when(self.config.enhanced_mode == "normal", |this| {
                                            this.primary()
                                        })
                                        .label("Normal")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_mode("normal".to_string(), cx);
                                        })),
                                )
                                .child(
                                    Button::new("mode-fakeip")
                                        .when(self.config.enhanced_mode == "fake-ip", |this| {
                                            this.primary()
                                        })
                                        .label("Fake-IP")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_mode("fake-ip".to_string(), cx);
                                        })),
                                )
                                .child(
                                    Button::new("mode-redir")
                                        .when(self.config.enhanced_mode == "redir-host", |this| {
                                            this.primary()
                                        })
                                        .label("Redir-Host")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_mode("redir-host".to_string(), cx);
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
                            .child(Input::new(&self.new_nameserver))
                            .child(Button::new("add-nameserver").label("Add").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.add_nameserver(cx);
                                }),
                            )),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .children(self.config.nameserver.iter().enumerate().map(
                                |(i, server)| {
                                    h_flex()
                                        .gap_2()
                                        .child(div().flex_1().child(server.clone()))
                                        .child(
                                            Button::new(SharedString::from(format!(
                                                "remove-ns-{}",
                                                i
                                            )))
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
                            .child(Input::new(&self.new_fallback))
                            .child(
                                Button::new("add-fallback")
                                    .label("Add")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_fallback(cx);
                                    })),
                            ),
                    )
                    .child(
                        v_flex().gap_1().children(
                            self.config
                                .fallback
                                .iter()
                                .flatten()
                                .enumerate()
                                .map(|(i, server)| {
                                    h_flex()
                                        .gap_2()
                                        .child(div().flex_1().child(server.clone()))
                                        .child(
                                            Button::new(SharedString::from(format!(
                                                "remove-fb-{}",
                                                i
                                            )))
                                            .ghost()
                                            .label("Remove")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.remove_fallback(i, cx);
                                            })),
                                        )
                                }),
                        ),
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
