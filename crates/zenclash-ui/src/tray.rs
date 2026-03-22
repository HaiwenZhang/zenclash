use std::sync::Arc;

use gpui::{
    actions, div, px, App, Context, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    h_flex, v_flex, ActiveTheme, Button, Collapsible, Icon, IconName, Root, Theme, ThemeMode,
    TitleBar,
};
use tokio::sync::RwLock;

use zenclash_core::{
    AppConfig, ConnectionItem, CoreManager, CoreState, ProfileItem, ProxyGroup, TrafficData,
};

use crate::components::sidebar::ZenSidebar;
use crate::pages::{ConnectionsPage, LogsPage, Page, ProfilesPage, ProxiesPage, SettingsPage};

actions!(
    zenclash,
    [
        Quit,
        ToggleSidebar,
        ShowWindow,
        HideWindow,
        ToggleSysProxy,
        ToggleTun,
        SetRuleMode,
        SetGlobalMode,
        SetDirectMode,
        UpdateTrayMenu,
    ]
);

pub struct TrayManager {
    core_manager: Arc<RwLock<CoreManager>>,
    config: Arc<RwLock<AppConfig>>,
    proxy_groups: Vec<ProxyGroup>,
    profiles: Vec<ProfileItem>,
    current_mode: OutboundMode,
    sysproxy_enabled: bool,
    tun_enabled: bool,
    traffic_up: u64,
    traffic_down: u64,
}

impl TrayManager {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, config: Arc<RwLock<AppConfig>>) -> Self {
        Self {
            core_manager,
            config,
            proxy_groups: Vec::new(),
            profiles: Vec::new(),
            current_mode: OutboundMode::Rule,
            sysproxy_enabled: false,
            tun_enabled: false,
            traffic_up: 0,
            traffic_down: 0,
        }
    }

    pub fn update_traffic(&mut self, up: u64, down: u64, cx: &mut Context<Self>) {
        self.traffic_up = up;
        self.traffic_down = down;
        cx.notify();
    }

    pub fn update_state(&mut self, state: TrayState, cx: &mut Context<Self>) {
        match state {
            TrayState::ModeChanged(mode) => self.current_mode = mode,
            TrayState::SysProxyChanged(enabled) => self.sysproxy_enabled = enabled,
            TrayState::TunChanged(enabled) => self.tun_enabled = enabled,
            TrayState::ProxyGroupsUpdated(groups) => self.proxy_groups = groups,
            TrayState::ProfilesUpdated(profiles) => self.profiles = profiles,
        }
        cx.notify();
    }

    fn build_menu(&self) -> TrayMenu {
        let mut menu = TrayMenu::new();

        menu.add_item(TrayMenuItem::label("ZenClash").disabled(true));
        menu.add_separator();

        menu.add_item(TrayMenuItem::action("Show Window", ShowWindow));
        menu.add_separator();

        menu.add_submenu(TraySubmenu::new(
            "Proxy Groups",
            self.build_proxy_groups_menu(),
        ));
        menu.add_submenu(TraySubmenu::new("Profiles", self.build_profiles_menu()));
        menu.add_separator();

        menu.add_item(TrayMenuItem::checkbox(
            "System Proxy",
            self.sysproxy_enabled,
            ToggleSysProxy,
        ));
        menu.add_item(TrayMenuItem::checkbox(
            "TUN Mode",
            self.tun_enabled,
            ToggleTun,
        ));
        menu.add_separator();

        menu.add_submenu(TraySubmenu::new("Mode", self.build_mode_menu()));
        menu.add_separator();

        if let Some(core) = self.core_manager.try_read() {
            let status = match core.state() {
                CoreState::Running => "Running",
                CoreState::Stopped => "Stopped",
                CoreState::Starting => "Starting...",
                CoreState::Error => "Error",
            };
            menu.add_item(TrayMenuItem::label(format!("Core: {}", status)).disabled(true));
        }

        let traffic_text = format!(
            "↑ {} ↓ {}",
            format_bytes(self.traffic_up),
            format_bytes(self.traffic_down)
        );
        menu.add_item(TrayMenuItem::label(traffic_text).disabled(true));
        menu.add_separator();

        menu.add_item(TrayMenuItem::action("Quit", Quit));

        menu
    }

    fn build_proxy_groups_menu(&self) -> Vec<TrayMenuItem> {
        self.proxy_groups
            .iter()
            .map(|group| {
                let selected = group.selected.clone().unwrap_or_default();
                TrayMenuItem::submenu(
                    group.name.clone(),
                    group
                        .proxies
                        .iter()
                        .map(|proxy| {
                            let is_selected = proxy.name == selected;
                            TrayMenuItem::checkbox(
                                proxy.name.clone(),
                                is_selected,
                                SelectProxy {
                                    group: group.name.clone(),
                                    proxy: proxy.name.clone(),
                                },
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn build_profiles_menu(&self) -> Vec<TrayMenuItem> {
        self.profiles
            .iter()
            .map(|profile| {
                TrayMenuItem::checkbox(
                    profile.name.clone(),
                    profile.used,
                    SelectProfile {
                        id: profile.id.clone(),
                    },
                )
            })
            .collect()
    }

    fn build_mode_menu(&self) -> Vec<TrayMenuItem> {
        vec![
            TrayMenuItem::radio("Rule", self.current_mode == OutboundMode::Rule, SetRuleMode),
            TrayMenuItem::radio(
                "Global",
                self.current_mode == OutboundMode::Global,
                SetGlobalMode,
            ),
            TrayMenuItem::radio(
                "Direct",
                self.current_mode == OutboundMode::Direct,
                SetDirectMode,
            ),
        ]
    }
}

impl Render for TrayManager {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub enum TrayState {
    ModeChanged(OutboundMode),
    SysProxyChanged(bool),
    TunChanged(bool),
    ProxyGroupsUpdated(Vec<ProxyGroup>),
    ProfilesUpdated(Vec<ProfileItem>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundMode {
    Rule,
    Global,
    Direct,
}

pub struct TrayMenu {
    items: Vec<TrayMenuItem>,
}

impl TrayMenu {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: TrayMenuItem) {
        self.items.push(item);
    }

    pub fn add_separator(&mut self) {
        self.items.push(TrayMenuItem::Separator);
    }

    pub fn add_submenu(&mut self, submenu: TraySubmenu) {
        self.items.push(TrayMenuItem::Submenu(submenu));
    }
}

pub enum TrayMenuItem {
    Label {
        text: String,
        disabled: bool,
    },
    Action {
        text: String,
        action: Box<dyn Fn()>,
    },
    Checkbox {
        text: String,
        checked: bool,
        action: Box<dyn Fn()>,
    },
    Radio {
        text: String,
        selected: bool,
        action: Box<dyn Fn()>,
    },
    Submenu(TraySubmenu),
    Separator,
}

impl TrayMenuItem {
    pub fn label(text: impl Into<String>) -> Self {
        Self::Label {
            text: text.into(),
            disabled: false,
        }
    }

    pub fn action(text: impl Into<String>, action: impl Fn() + 'static) -> Self {
        Self::Action {
            text: text.into(),
            action: Box::new(action),
        }
    }

    pub fn checkbox(text: impl Into<String>, checked: bool, action: impl Fn() + 'static) -> Self {
        Self::Checkbox {
            text: text.into(),
            checked,
            action: Box::new(action),
        }
    }

    pub fn radio(text: impl Into<String>, selected: bool, action: impl Fn() + 'static) -> Self {
        Self::Radio {
            text: text.into(),
            selected,
            action: Box::new(action),
        }
    }

    pub fn submenu(text: impl Into<String>, items: Vec<TrayMenuItem>) -> Self {
        Self::Submenu(TraySubmenu {
            label: text.into(),
            items,
        })
    }
}

pub struct TraySubmenu {
    label: String,
    items: Vec<TrayMenuItem>,
}

impl TraySubmenu {
    pub fn new(label: impl Into<String>, items: Vec<TrayMenuItem>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}
