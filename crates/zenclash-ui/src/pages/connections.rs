use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, App, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex, v_flex, ActiveTheme,
};
use parking_lot::RwLock;

use zenclash_core::prelude::{ConnectionItem, CoreManager, CoreState};

pub struct ConnectionsPage {
    core_manager: Arc<RwLock<CoreManager>>,
    connections: Vec<ConnectionItem>,
    selected_connection: Option<String>,
    focus_handle: FocusHandle,
}

impl ConnectionsPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, cx: &mut Context<Self>) -> Self {
        Self {
            core_manager,
            connections: Vec::new(),
            selected_connection: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let manager = core_manager.read();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if manager.is_running().await {
                        manager.get_connections().await.ok()
                    } else {
                        None
                    }
                })
            });
            
            if let Some(response) = result {
                let _ = this.update(cx, |this, cx| {
                    this.connections = response.connections;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn update_connections(&mut self, connections: Vec<ConnectionItem>, cx: &mut Context<Self>) {
        self.connections = connections;
        cx.notify();
    }

    pub fn close_connection(&mut self, id: String, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        let id_clone = id.clone();
        cx.spawn(async move |_, _| {
            let manager = core_manager.read();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = manager.close_connection(&id_clone).await;
                })
            });
        })
        .detach();
        
        self.connections.retain(|c| c.id != id);
        cx.notify();
    }

    pub fn close_all(&mut self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |_, _| {
            let manager = core_manager.read();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = manager.close_all_connections().await;
                })
            });
        })
        .detach();
        
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
                            .child("Connections"),
                    )
                    .child(
                        Button::new("close-all")
                            .label("Close All")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_all(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} active connections", self.connections.len())),
            )
            .child(
                v_flex()
                    .gap_1()
                    .children(self.connections.iter().map(|conn| {
                        let id = conn.id.clone();
                        let source =
                            format!("{}:{}", conn.metadata.source_ip, conn.metadata.source_port);
                        let dest = conn.metadata.host.clone().unwrap_or_else(|| {
                            format!(
                                "{}:{}",
                                conn.metadata.destination_ip.as_deref().unwrap_or("?"),
                                conn.metadata.destination_port
                            )
                        });
                        h_flex()
                            .gap_2()
                            .p_2()
                            .child(div().flex_1().child(format!("{} -> {}", source, dest)))
                            .child(
                                Button::new(SharedString::from(format!("close-{}", conn.id)))
                                    .ghost()
                                    .label("Close")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.close_connection(id.clone(), cx);
                                    })),
                            )
                    })),
            )
    }
}
