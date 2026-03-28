use std::sync::Arc;

use gpui::{
    actions, div, px, App, AppContext, Context, Entity, IntoElement, KeyBinding, ParentElement, Render,
    Styled, Window, WindowBounds, WindowOptions,
};
use gpui_component::{h_flex, ActiveTheme, TitleBar};
use parking_lot::RwLock;

use crate::app::WindowConfig;
use zenclash_core::prelude::CoreManager;

actions!(zenclash_floating, [ToggleFloatingWindow]);

pub fn default_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::centered(gpui::size(px(1200.), px(800.)), cx)),
        titlebar: Some(TitleBar::title_bar_options()),
        ..Default::default()
    }
}

pub fn floating_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::centered(gpui::size(px(140.), px(50.)), cx)),
        titlebar: None,
        ..Default::default()
    }
}

pub fn setup_keybindings(cx: &mut App) {
    #[cfg(target_os = "macos")]
    {
        cx.bind_keys([
            KeyBinding::new("cmd-q", crate::app::Quit, None),
            KeyBinding::new("cmd-b", crate::app::ToggleSidebar, None),
            KeyBinding::new("cmd-1", crate::app::NavigateProxies, None),
            KeyBinding::new("cmd-2", crate::app::NavigateProfiles, None),
            KeyBinding::new("cmd-3", crate::app::NavigateConnections, None),
            KeyBinding::new("cmd-4", crate::app::NavigateLogs, None),
            KeyBinding::new("cmd-,", crate::app::NavigateSettings, None),
        ]);
    }

    #[cfg(not(target_os = "macos"))]
    {
        cx.bind_keys([
            KeyBinding::new("ctrl-q", crate::app::Quit, None),
            KeyBinding::new("ctrl-b", crate::app::ToggleSidebar, None),
            KeyBinding::new("ctrl-1", crate::app::NavigateProxies, None),
            KeyBinding::new("ctrl-2", crate::app::NavigateProfiles, None),
            KeyBinding::new("ctrl-3", crate::app::NavigateConnections, None),
            KeyBinding::new("ctrl-4", crate::app::NavigateLogs, None),
            KeyBinding::new("ctrl-,", crate::app::NavigateSettings, None),
        ]);
    }
}

pub fn center_window_on_screen(width: f32, height: f32, cx: &App) -> WindowBounds {
    WindowBounds::centered(gpui::size(px(width), px(height)), cx)
}

pub struct FloatingWindowView {
    core_manager: Arc<RwLock<CoreManager>>,
    upload_speed: u64,
    download_speed: u64,
}

impl FloatingWindowView {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = Self {
            core_manager,
            upload_speed: 0,
            download_speed: 0,
        };
        view.start_traffic_update(cx);
        view
    }

    fn start_traffic_update(&self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            loop {
                let traffic = {
                    let manager = core_manager.read();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            manager.get_traffic().await
                        })
                    })
                };

                if let Ok(stream) = traffic {
                    let mut stream = stream;
                    loop {
                        let data = tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                stream.next().await
                            })
                        });

                        match data {
                            Some(traffic_data) => {
                                let _ = this.update(cx, |this, cx| {
                                    this.upload_speed = traffic_data.up;
                                    this.download_speed = traffic_data.down;
                                    cx.notify();
                                });
                            }
                            None => break,
                        }
                    }
                }

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    })
                });
            }
        })
        .detach();
    }

    fn format_speed(bytes: u64) -> String {
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
}

impl Render for FloatingWindowView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        
        div()
            .size_full()
            .bg(theme.background.opacity(0.95))
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .flex()
            .items_center()
            .justify_center()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_color(gpui::rgb(0x4ade80))
                            .text_xs()
                            .child(format!("↑ {}", Self::format_speed(self.upload_speed)))
                    )
                    .child(
                        div()
                            .text_color(gpui::rgb(0x60a5fa))
                            .text_xs()
                            .child(format!("↓ {}", Self::format_speed(self.download_speed)))
                    )
            )
    }
}

pub fn create_floating_window(
    core_manager: Arc<RwLock<CoreManager>>,
    cx: &mut App,
) -> anyhow::Result<()> {
    let options = floating_window_options(cx);
    
    cx.spawn(async move |cx| {
        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| FloatingWindowView::new(core_manager, window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
    })
    .detach();

    Ok(())
}