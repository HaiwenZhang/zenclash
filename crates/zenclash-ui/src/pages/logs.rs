use std::sync::Arc;

use gpui::{
    div, px, prelude::FluentBuilder, InteractiveElement, StatefulInteractiveElement,
    App, Context, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, ScrollHandle, Styled, Window,
};
use gpui_component::{
    button::Button,
    v_flex, h_flex,
    ActiveTheme,
};
use parking_lot::RwLock;
use zenclash_core::prelude::{CoreManager, LogItem};

pub struct LogsPage {
    core_manager: Arc<RwLock<CoreManager>>,
    logs: Vec<LogEntry>,
    max_logs: usize,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    streaming: bool,
}

#[derive(Debug, Clone)]
struct LogEntry {
    level: String,
    payload: String,
}

impl LogsPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            core_manager,
            logs: Vec::new(),
            max_logs: 1000,
            scroll_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            streaming: false,
        }
    }

    fn add_log(&mut self, log: LogItem, cx: &mut Context<Self>) {
        self.logs.push(LogEntry {
            level: log.level,
            payload: log.payload,
        });
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
        cx.notify();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.logs.clear();
        cx.notify();
    }

    fn start_streaming(&mut self, cx: &mut Context<Self>) {
        if self.streaming {
            return;
        }
        self.streaming = true;

        let core_manager = self.core_manager.clone();

        cx.spawn(async move |this, cx| {
            loop {
                let stream_result = {
                    let manager = core_manager.read();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            manager.get_logs(Some("info")).await
                        })
                    })
                };

                match stream_result {
                    Ok(stream) => {
                        loop {
                            let log_item = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    stream.next().await
                                })
                            });

                            match log_item {
                                Some(item) => {
                                    let _ = this.update(cx, |this, cx| {
                                        this.add_log(item, cx);
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
}

impl Focusable for LogsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LogsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.streaming {
            self.start_streaming(cx);
        }

        let theme = cx.theme();
        let error_color = theme.danger;
        let warn_color = theme.warning;
        let info_color = theme.accent;
        let debug_color = theme.muted_foreground;
        let default_color = theme.foreground;

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
                            .child("Logs")
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("clear-logs")
                                    .label("Clear")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_logs(cx);
                                    }))
                            )
                    )
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} log entries", self.logs.len()))
            )
            .child(
                div()
                    .id("logs-container")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .bg(cx.theme().background)
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_2()
                    .child(
                        v_flex()
                            .gap_0()
                            .children(self.logs.iter().map(|log| {
                                let level_color = match log.level.to_lowercase().as_str() {
                                    "error" | "fatal" => error_color,
                                    "warn" | "warning" => warn_color,
                                    "info" => info_color,
                                    "debug" | "trace" => debug_color,
                                    _ => default_color,
                                };
                                h_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .min_w(px(60.0))
                                            .text_color(level_color)
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child(log.level.clone())
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .font_family("monospace")
                                            .child(log.payload.clone())
                                    )
                            }))
                    )
            )
    }
}