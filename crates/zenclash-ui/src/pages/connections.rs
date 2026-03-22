use gpui::{
    div, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::Button,
    v_flex, h_flex,
    ActiveTheme,
};

use zenclash_core::ConnectionItem;

pub struct ConnectionsPage {
    connections: Vec<ConnectionItem>,
    selected_connection: Option<String>,
    focus_handle: FocusHandle,
}

impl ConnectionsPage {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            connections: Vec::new(),
            selected_connection: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn update_connections(&mut self, connections: Vec<ConnectionItem>, cx: &mut Context<Self>) {
        self.connections = connections;
        cx.notify();
    }

    pub fn close_connection(&mut self, id: String, cx: &mut Context<Self>) {
        self.connections.retain(|c| c.id != id);
        cx.notify();
    }

    pub fn close_all(&mut self, cx: &mut Context<Self>) {
        self.connections.clear();
        cx.notify();
    }
}

impl Focusable for ConnectionsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConnectionsPage {
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
                            .child("Connections")
                    )
                    .child(
                        Button::new("close-all")
                            .label("Close All")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_all(cx);
                            }))
                    )
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} active connections", self.connections.len()))
            )
            .child(
                v_flex()
                    .gap_1()
                    .children(self.connections.iter().map(|conn| {
                        let id = conn.id.clone();
                        h_flex()
                            .gap_2()
                            .p_2()
                            .child(div().flex_1().child(format!("{} -> {}", conn.source, conn.destination)))
                            .child(
                                Button::new(format!("close-{}", conn.id))
                                    .ghost()
                                    .label("Close")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.close_connection(id.clone(), cx);
                                    }))
                            )
                    }))
            )
    }
}
