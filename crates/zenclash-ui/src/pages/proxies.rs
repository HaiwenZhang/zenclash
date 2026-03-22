use std::sync::Arc;

use gpui::{
    div, px, App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
    virtual_list::VirtualList,
    ActiveTheme, Icon, IconName,
};
use tokio::sync::RwLock;

use zenclash_core::{CoreManager, DelayTestResult, DelayTester, Proxy, ProxyGroup};

pub struct ProxiesPage {
    proxy_groups: Vec<ProxyGroup>,
    selected_group: Option<String>,
    selected_proxy: Option<String>,
    search_query: Entity<InputState>,
    delay_results: std::collections::HashMap<String, u32>,
    is_testing_delay: bool,
    focus_handle: FocusHandle,
}

impl ProxiesPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_query =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search proxies..."));

        Self {
            proxy_groups: Vec::new(),
            selected_group: None,
            selected_proxy: None,
            search_query,
            delay_results: std::collections::HashMap::new(),
            is_testing_delay: false,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn with_proxy_groups(mut self, groups: Vec<ProxyGroup>) -> Self {
        self.proxy_groups = groups;
        if let Some(first) = self.proxy_groups.first() {
            self.selected_group = Some(first.name.clone());
        }
        self
    }

    pub fn update_proxy_groups(&mut self, groups: Vec<ProxyGroup>, cx: &mut Context<Self>) {
        self.proxy_groups = groups;
        cx.notify();
    }

    pub fn select_proxy(&mut self, group_name: String, proxy_name: String, cx: &mut Context<Self>) {
        self.selected_group = Some(group_name);
        self.selected_proxy = Some(proxy_name);
        cx.notify();

        cx.emit(ProxyPageEvent::ProxySelected {
            group: group_name,
            proxy: proxy_name,
        });
    }

    pub fn test_delay(&mut self, cx: &mut Context<Self>) {
        if self.is_testing_delay {
            return;
        }

        self.is_testing_delay = true;
        cx.notify();

        let groups = self.proxy_groups.clone();
        cx.spawn(async move |this, mut cx| {
            let tester = DelayTester::default();

            for group in &groups {
                for proxy in &group.proxies {
                    if let Ok(result) = tester.test_proxy(proxy).await {
                        this.update(&mut cx, |this, cx| {
                            this.delay_results
                                .insert(proxy.name.clone(), result.delay_ms);
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }

            this.update(&mut cx, |this, cx| {
                this.is_testing_delay = false;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_group_header(&self, group: &ProxyGroup, cx: &Context<Self>) -> impl IntoElement {
        let is_selected = self.selected_group.as_ref() == Some(&group.name);
        let delay_text = if let Some(delay) = group
            .proxies
            .iter()
            .filter_map(|p| self.delay_results.get(&p.name))
            .min()
        {
            format!("{} ms", delay)
        } else {
            "-".to_string()
        };

        h_flex()
            .gap_2()
            .p_2()
            .cursor_pointer()
            .when(is_selected, |this| this.bg(cx.theme().primary.opacity(0.1)))
            .child(
                div()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(group.name.clone()),
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("({})", group.proxies.len())),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_color(if delay_text != "-" {
                        cx.theme().success
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(delay_text),
            )
    }

    fn render_proxy_item(
        &self,
        proxy: &Proxy,
        group_name: &str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_proxy.as_ref() == Some(&proxy.name);
        let delay = self.delay_results.get(&proxy.name).copied();

        h_flex()
            .gap_2()
            .p_2()
            .pl_6()
            .cursor_pointer()
            .when(is_selected, |this| this.bg(cx.theme().primary.opacity(0.2)))
            .child(div().child(proxy.name.clone()))
            .child(div().flex_1())
            .child(
                div()
                    .text_color(match delay {
                        Some(d) if d < 100 => cx.theme().success,
                        Some(d) if d < 300 => cx.theme().warning,
                        Some(_) => cx.theme().destructive,
                        None => cx.theme().muted_foreground,
                    })
                    .child(match delay {
                        Some(d) => format!("{} ms", d),
                        None => "-".to_string(),
                    }),
            )
    }
}

impl Focusable for ProxiesPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ProxiesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let search = self.search_query.clone();

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
                            .child("Proxies"),
                    )
                    .child(
                        Button::new("test-delay")
                            .primary()
                            .when(self.is_testing_delay, |this| {
                                this.label("Testing...").disabled(true)
                            })
                            .when(!self.is_testing_delay, |this| this.label("Test Delay"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.test_delay(cx);
                            })),
                    ),
            )
            .child(Input::new(&search, window, cx).placeholder("Search proxies..."))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .gap_1()
                            .children(self.proxy_groups.iter().map(|group| {
                                let group_name = group.name.clone();
                                let group_clone = group.clone();

                                v_flex()
                                    .gap_0()
                                    .child(self.render_group_header(group, cx))
                                    .children(group.proxies.iter().map(move |proxy| {
                                        let proxy_name = proxy.name.clone();
                                        let group_name_clone = group_name.clone();

                                        self.render_proxy_item(proxy, &group_name, cx).on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                this.select_proxy(
                                                    group_name_clone.clone(),
                                                    proxy_name.clone(),
                                                    cx,
                                                );
                                            }),
                                        )
                                    }))
                            })),
                    ),
            )
    }
}

pub enum ProxyPageEvent {
    ProxySelected { group: String, proxy: String },
}

impl gpui::EventEmitter<ProxyPageEvent> for ProxiesPage {}
