use std::sync::Arc;

use gpui::{prelude::FluentBuilder, InteractiveElement,
    div, App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
    ActiveTheme, Disableable,
};
use parking_lot::RwLock;

use zenclash_core::prelude::{CoreManager, ProxyGroup};

pub struct ProxiesPage {
    core_manager: Arc<RwLock<CoreManager>>,
    proxy_groups: Vec<ProxyGroup>,
    selected_group: Option<String>,
    selected_proxy: Option<String>,
    search_query: Entity<InputState>,
    delay_results: std::collections::HashMap<String, u32>,
    is_testing_delay: bool,
    traffic_up: u64,
    traffic_down: u64,
    traffic_streaming: bool,
    focus_handle: FocusHandle,
}

impl ProxiesPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            core_manager,
            proxy_groups: Vec::new(),
            selected_group: None,
            selected_proxy: None,
            search_query: cx.new(|cx| InputState::new(window, cx).placeholder("Search proxies...")),
            delay_results: std::collections::HashMap::new(),
            is_testing_delay: false,
            traffic_up: 0,
            traffic_down: 0,
            traffic_streaming: false,
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

    pub fn refresh_proxies(&mut self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let manager = core_manager.read();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    manager.get_proxies().await
                })
            });
            
            if let Ok(response) = result {
                let groups: Vec<ProxyGroup> = response
                    .proxies
                    .into_iter()
                    .filter(|(_, proxy)| {
                        let t = proxy.proxy_type.to_lowercase();
                        t == "selector" || t == "urltest" || t == "fallback" || t == "loadbalance"
                    })
                    .map(|(name, proxy)| {
                        use std::collections::HashMap;
                        ProxyGroup {
                            name,
                            group_type: zenclash_core::prelude::ProxyType::Selector,
                            proxies: proxy.all.unwrap_or_default(),
                            url: None,
                            interval: None,
                            tolerance: None,
                            lazy: None,
                            timeout: None,
                            use_count: None,
                            use_providers: vec![],
                            current: proxy.now,
                            extra: HashMap::new(),
                        }
                    })
                    .collect();
                
                let _ = this.update(cx, |this, cx| {
                    this.proxy_groups = groups;
                    if this.selected_group.is_none() {
                        this.selected_group = this.proxy_groups.first().map(|g| g.name.clone());
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn select_proxy(&mut self, group_name: String, proxy_name: String, cx: &mut Context<Self>) {
        self.selected_group = Some(group_name.clone());
        self.selected_proxy = Some(proxy_name.clone());
        
        let core_manager = self.core_manager.clone();
        let group = group_name.clone();
        let proxy = proxy_name.clone();
        cx.spawn(async move |_, _| {
            let manager = core_manager.read();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = manager.select_proxy(&group, &proxy).await;
                })
            });
        })
        .detach();

        cx.emit(ProxyPageEvent::ProxySelected {
            group: group_name,
            proxy: proxy_name,
        });
        cx.notify();
    }

    pub fn test_delay(&mut self, cx: &mut Context<Self>) {
        if self.is_testing_delay {
            return;
        }

        self.is_testing_delay = true;
        cx.notify();

        let groups = self.proxy_groups.clone();
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let manager = core_manager.read();
            
            for group in &groups {
                for proxy in &group.proxies {
                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            manager.delay_test(proxy, None, None).await
                        })
                    });
                    if let Ok(delay_result) = result {
                        this.update(cx, |this, cx| {
                            this.delay_results
                                .insert(proxy.clone(), delay_result.delay);
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }

            this.update(cx, |this, cx| {
                this.is_testing_delay = false;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn start_traffic_stream(&mut self, cx: &mut Context<Self>) {
        if self.traffic_streaming {
            return;
        }
        self.traffic_streaming = true;

        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            loop {
                let stream_result = {
                    let manager = core_manager.read();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            manager.get_traffic().await
                        })
                    })
                };

                match stream_result {
                    Ok(stream) => {
                        loop {
                            let traffic = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    stream.next().await
                                })
                            });

                            match traffic {
                                Some(data) => {
                                    let _ = this.update(cx, |this, cx| {
                                        this.traffic_up = data.up;
                                        this.traffic_down = data.down;
                                        cx.notify();
                                    });
                                }
                                None => break,
                            }
                        }
                    }
                    Err(_) => {
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            })
                        });
                    }
                }
            }
        })
        .detach();
    }

    fn format_traffic(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B/s", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB/s", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB/s", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB/s", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn render_group_header(&self, group: &ProxyGroup, cx: &Context<Self>) -> impl IntoElement {
        let is_selected = self.selected_group.as_ref() == Some(&group.name);
        let delay_text = if let Some(delay) = group
            .proxies
            .iter()
            .filter_map(|p| self.delay_results.get(p))
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
        proxy_name: &str,
        group_name: &str,
        cx: &Context<Self>,
    ) -> impl IntoElement + StatefulInteractiveElement {
        let is_selected = self.selected_proxy.as_ref() == Some(&proxy_name.to_string());
        let delay = self.delay_results.get(proxy_name).copied();
        let group_name = group_name.to_string();
        let proxy_name = proxy_name.to_string();
        let proxy_name_for_closure = proxy_name.clone();

        h_flex()
            .id(SharedString::from(format!("proxy-item-{}-{}", group_name, proxy_name)))
            .gap_2()
            .p_2()
            .pl_6()
            .cursor_pointer()
            .when(is_selected, |this| this.bg(cx.theme().primary.opacity(0.2)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_proxy(group_name.clone(), proxy_name_for_closure.clone(), cx);
            }))
            .child(div().child(proxy_name))
            .child(div().flex_1())
            .child(
                div()
                    .text_color(match delay {
                        Some(d) if d < 100 => cx.theme().success,
                        Some(d) if d < 300 => cx.theme().warning,
                        Some(_) => cx.theme().danger,
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.traffic_streaming {
            self.start_traffic_stream(cx);
        }

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
                        h_flex()
                            .gap_4()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child("↑")
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child(Self::format_traffic(self.traffic_up))
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child("↓")
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child(Self::format_traffic(self.traffic_down))
                                    )
                            )
                            .child(
                                Button::new("test-delay")
                                    .primary()
                                    .when(self.is_testing_delay, |this| {
                                        this.child("Testing...").disabled(true)
                                    })
                                    .when(!self.is_testing_delay, |this| this.child("Test Delay"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.test_delay(cx);
                                    })),
                            ),
                    ),
            )
            .child(Input::new(&search))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .gap_1()
                            .children({
                                let proxy_groups = self.proxy_groups.clone();
                                proxy_groups.iter().map(|group| {
                                    let group_name = group.name.clone();
                                    let proxies: Vec<_> = group.proxies.iter().map(|proxy| {
                                        self.render_proxy_item(proxy, &group_name, cx)
                                    }).collect();

                                    v_flex()
                                        .gap_0()
                                        .child(self.render_group_header(group, cx))
                                        .children(proxies)
                                }).collect::<Vec<_>>()
                            }),
                    ),
            )
    }
}

pub enum ProxyPageEvent {
    ProxySelected { group: String, proxy: String },
}

impl gpui::EventEmitter<ProxyPageEvent> for ProxiesPage {}
