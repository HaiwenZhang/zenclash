use gpui::{
    div, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, ScrollHandle, Styled, Window,
};
use gpui_component::{
    button::Button,
    scrollable::Scrollable,
    v_flex, h_flex,
    ActiveTheme,
};

pub struct LogsPage {
    logs: Vec<String>,
    max_logs: usize,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    auto_scroll: bool,
}

impl LogsPage {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            logs: Vec::new(),
            max_logs: 1000,
            scroll_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            auto_scroll: true,
        }
    }

    pub fn add_log(&mut self, log: String, cx: &mut Context<Self>) {
        self.logs.push(log);
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
        cx.notify();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.logs.clear();
        cx.notify();
    }

    pub fn set_auto_scroll(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.auto_scroll = enabled;
        cx.notify();
    }
}

impl Focusable for LogsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LogsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .gap_0()
                            .children(self.logs.iter().map(|log| {
                                div()
                                    .p_1()
                                    .text_sm()
                                    .font_family(gpui::FamilyName::Monospace)
                                    .child(log.clone())
                            }))
                    )
            )
    }
}
