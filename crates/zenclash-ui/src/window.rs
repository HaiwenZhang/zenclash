use gpui::{px, App, KeyBinding, Window, WindowBounds, WindowOptions};
use gpui_component::TitleBar;

use crate::app::WindowConfig;

pub fn default_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::centered(gpui::size(px(1200.), px(800.)), cx)),
        titlebar: Some(TitleBar::title_bar_options()),
        ..Default::default()
    }
}

pub fn floating_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::centered(gpui::size(px(120.), px(42.)), cx)),
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

pub struct FloatingWindowState {
    pub show_traffic: bool,
    pub position: Option<(f32, f32)>,
}

impl Default for FloatingWindowState {
    fn default() -> Self {
        Self {
            show_traffic: true,
            position: None,
        }
    }
}

pub struct FloatingWindow;

impl FloatingWindow {
    pub fn toggle(cx: &mut App) {
        cx.update_global(|state: &mut FloatingWindowState, _| {
            state.show_traffic = !state.show_traffic;
        });
    }

    pub fn is_visible(cx: &App) -> bool {
        cx.read_global::<FloatingWindowState>().show_traffic
    }
}
