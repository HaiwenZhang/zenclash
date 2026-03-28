use std::sync::Arc;

use gpui::{
    actions, div, Action, App, AppContext, Context, Entity, IntoElement, Render, Window,
    WeakEntity,
};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem, CheckMenuItem, Submenu, PredefinedMenuItem, MenuEvent, MenuId},
};

use zenclash_core::prelude::{AppConfig, CoreManager, CoreState, ProfileItem, ProxyGroup};

use crate::app::{Quit, ToggleSysProxy, ToggleTun};

actions!(
    zenclash,
    [
        ShowWindow,
        HideWindow,
        SetRuleMode,
        SetGlobalMode,
        SetDirectMode,
        UpdateTrayMenu,
    ]
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectProxy {
    pub group: String,
    pub proxy: String,
}

impl Action for SelectProxy {
    fn boxed_clone(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
    fn partial_eq(&self, other: &dyn Action) -> bool {
        if let Some(other) = (other as &dyn std::any::Any).downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    fn name(&self) -> &'static str {
        "SelectProxy"
    }
    fn name_for_type() -> &'static str
    where
        Self: Sized,
    {
        "SelectProxy"
    }
    fn build(_value: serde_json::Value) -> Result<Box<dyn Action>, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            group: String::new(),
            proxy: String::new(),
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectProfile {
    pub id: String,
}

impl Action for SelectProfile {
    fn boxed_clone(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
    fn partial_eq(&self, other: &dyn Action) -> bool {
        if let Some(other) = (other as &dyn std::any::Any).downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    fn name(&self) -> &'static str {
        "SelectProfile"
    }
    fn name_for_type() -> &'static str
    where
        Self: Sized,
    {
        "SelectProfile"
    }
    fn build(_value: serde_json::Value) -> Result<Box<dyn Action>, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Box::new(Self { id: String::new() }))
    }
}

/// Event sent from tray-icon thread to GPUI context
#[derive(Debug, Clone)]
pub enum TrayActionEvent {
    ShowWindow,
    HideWindow,
    ToggleSysProxy,
    ToggleTun,
    SetRuleMode,
    SetGlobalMode,
    SetDirectMode,
    SelectProxy { group: String, proxy: String },
    SelectProfile { id: String },
    Quit,
}

/// Platform-specific tray implementation wrapping tray-icon
pub struct PlatformTray {
    tray: Option<TrayIcon>,
    menu_ids: Arc<RwLock<MenuIdRegistry>>,
    action_sender: mpsc::UnboundedSender<TrayActionEvent>,
}

/// Registry to map menu IDs to actions
pub struct MenuIdRegistry {
    ids: std::collections::HashMap<MenuId, TrayActionEvent>,
}

impl MenuIdRegistry {
    pub fn new() -> Self {
        Self {
            ids: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, id: MenuId, action: TrayActionEvent) {
        self.ids.insert(id, action);
    }

    pub fn get(&self, id: &MenuId) -> Option<&TrayActionEvent> {
        self.ids.get(id)
    }

    pub fn clear(&mut self) {
        self.ids.clear();
    }
}

impl PlatformTray {
    /// Create a new platform tray with event sender
    pub fn new(action_sender: mpsc::UnboundedSender<TrayActionEvent>) -> Self {
        Self {
            tray: None,
            menu_ids: Arc::new(RwLock::new(MenuIdRegistry::new())),
            action_sender,
        }
    }

    /// Initialize the tray icon
    pub fn init(&mut self, state: &TrayState) -> anyhow::Result<()> {
        // Create a simple icon (32x32 RGBA with gradient-like pattern)
        let icon = create_tray_icon();
        
        // Build the menu
        let menu = build_platform_menu(state, self.menu_ids.clone());
        
        // Build tooltip with traffic info
        let tooltip = format_tooltip(state);
        
        // Create tray icon
        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip(&tooltip)
            .with_menu_on_left_click(false)
            .build()?;
        
        self.tray = Some(tray);
        
        // Start menu event listener thread
        self.start_event_listener();
        
        Ok(())
    }

    /// Update tray menu and tooltip
    pub fn update(&self, state: &TrayState) -> anyhow::Result<()> {
        if let Some(tray) = &self.tray {
            // Update menu
            let menu = build_platform_menu(state, self.menu_ids.clone());
            tray.set_menu(Some(Box::new(menu)));
            
            // Update tooltip
            let tooltip = format_tooltip(state);
            tray.set_tooltip(Some(&tooltip))?;
        }
        Ok(())
    }

    /// Start the menu event listener in a background thread
    fn start_event_listener(&self) {
        let sender = self.action_sender.clone();
        let menu_ids = self.menu_ids.clone();
        
        std::thread::spawn(move || {
            loop {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    let ids = menu_ids.read();
                    if let Some(action) = ids.get(&event.id) {
                        if sender.send(action.clone()).is_err() {
                            break;
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    }
}

fn create_tray_icon() -> tray_icon::Icon {
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    
    for y in 0..size {
        for x in 0..size {
            let cx = size / 2;
            let cy = size / 2;
            let dist = ((x as f32 - cx as f32).powi(2) + (y as f32 - cy as f32).powi(2)).sqrt();
            let max_dist = (size / 2) as f32;
            
            if dist < max_dist * 0.7 {
                let intensity = 1.0 - (dist / max_dist) * 0.5;
                rgba.push((60.0 * intensity) as u8);
                rgba.push((80.0 * intensity) as u8);
                rgba.push((120.0 * intensity) as u8);
                rgba.push(255);
            } else if dist < max_dist * 0.85 {
                rgba.push(100);
                rgba.push(120);
                rgba.push(160);
                rgba.push(255);
            } else {
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }
    
    tray_icon::Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

/// Build the platform menu from tray state
fn build_platform_menu(state: &TrayState, menu_ids: Arc<RwLock<MenuIdRegistry>>) -> Menu {
    // Clear previous IDs
    menu_ids.write().clear();
    
    let menu = Menu::new();
    let mut ids = menu_ids.write();
    
    // Header label (disabled)
    let header = MenuItem::new("ZenClash", false, None);
    menu.append(&header).ok();
    
    menu.append(&PredefinedMenuItem::separator()).ok();
    
    // Show Window action
    let show_item = MenuItem::new("Show Window", true, None);
    ids.register(show_item.id().clone(), TrayActionEvent::ShowWindow);
    menu.append(&show_item).ok();
    
    menu.append(&PredefinedMenuItem::separator()).ok();
    
    // Proxy Groups submenu
    let proxy_groups_menu = build_proxy_groups_submenu(&state.proxy_groups, &mut ids);
    menu.append(&proxy_groups_menu).ok();
    
    // Profiles submenu
    let profiles_menu = build_profiles_submenu(&state.profiles, &mut ids);
    menu.append(&profiles_menu).ok();
    
    menu.append(&PredefinedMenuItem::separator()).ok();
    
    // System Proxy checkbox
    let sysproxy_item = CheckMenuItem::new("System Proxy", true, state.sysproxy_enabled, None);
    ids.register(sysproxy_item.id().clone(), TrayActionEvent::ToggleSysProxy);
    menu.append(&sysproxy_item).ok();
    
    // TUN checkbox
    let tun_item = CheckMenuItem::new("TUN Mode", true, state.tun_enabled, None);
    ids.register(tun_item.id().clone(), TrayActionEvent::ToggleTun);
    menu.append(&tun_item).ok();
    
    menu.append(&PredefinedMenuItem::separator()).ok();
    
    // Mode submenu
    let mode_menu = build_mode_submenu(state.current_mode, &mut ids);
    menu.append(&mode_menu).ok();
    
    menu.append(&PredefinedMenuItem::separator()).ok();
    
    // Core status (disabled label)
    let status_text = format!("Core: {}", format_core_state(state.core_state));
    let status_item = MenuItem::new(&status_text, false, None);
    menu.append(&status_item).ok();
    
    // Traffic display (disabled label)
    let traffic_text = format!(
        "↑ {} ↓ {}",
        format_bytes(state.traffic_up),
        format_bytes(state.traffic_down)
    );
    let traffic_item = MenuItem::new(&traffic_text, false, None);
    menu.append(&traffic_item).ok();
    
    menu.append(&PredefinedMenuItem::separator()).ok();
    
    // Quit
    let quit_item = MenuItem::new("Quit", true, None);
    ids.register(quit_item.id().clone(), TrayActionEvent::Quit);
    menu.append(&quit_item).ok();
    
    menu
}

/// Build proxy groups submenu
fn build_proxy_groups_submenu(
    groups: &[ProxyGroup],
    ids: &mut MenuIdRegistry,
) -> Submenu {
    let submenu = Submenu::new("Proxy Groups", true);
    
    for group in groups {
        let group_submenu = Submenu::new(&group.name, true);
        let selected = group.current.clone().unwrap_or_default();
        
        for proxy in &group.proxies {
            let is_selected = *proxy == selected;
            let item = CheckMenuItem::new(proxy, true, is_selected, None);
            ids.register(
                item.id().clone(),
                TrayActionEvent::SelectProxy {
                    group: group.name.clone(),
                    proxy: proxy.clone(),
                },
            );
            group_submenu.append(&item).ok();
        }
        
        submenu.append(&group_submenu).ok();
    }
    
    submenu
}

/// Build profiles submenu
fn build_profiles_submenu(
    profiles: &[ProfileItem],
    ids: &mut MenuIdRegistry,
) -> Submenu {
    let submenu = Submenu::new("Profiles", true);
    
    for profile in profiles {
        let item = CheckMenuItem::new(&profile.name, true, false, None);
        ids.register(
            item.id().clone(),
            TrayActionEvent::SelectProfile { id: profile.id.clone() },
        );
        submenu.append(&item).ok();
    }
    
    submenu
}

/// Build mode submenu
fn build_mode_submenu(current_mode: OutboundMode, ids: &mut MenuIdRegistry) -> Submenu {
    let submenu = Submenu::new("Mode", true);
    
    let rule_item = CheckMenuItem::new("Rule", true, current_mode == OutboundMode::Rule, None);
    ids.register(rule_item.id().clone(), TrayActionEvent::SetRuleMode);
    submenu.append(&rule_item).ok();
    
    let global_item = CheckMenuItem::new("Global", true, current_mode == OutboundMode::Global, None);
    ids.register(global_item.id().clone(), TrayActionEvent::SetGlobalMode);
    submenu.append(&global_item).ok();
    
    let direct_item = CheckMenuItem::new("Direct", true, current_mode == OutboundMode::Direct, None);
    ids.register(direct_item.id().clone(), TrayActionEvent::SetDirectMode);
    submenu.append(&direct_item).ok();
    
    submenu
}

/// Format tooltip with traffic info
fn format_tooltip(state: &TrayState) -> String {
    let status = format_core_state(state.core_state);
    let traffic = format!(
        "↑{} ↓{}",
        format_bytes(state.traffic_up),
        format_bytes(state.traffic_down)
    );
    format!("ZenClash - {} | {}", status, traffic)
}

/// Format core state for display
fn format_core_state(state: CoreState) -> &'static str {
    match state {
        CoreState::Running => "Running",
        CoreState::Stopped => "Stopped",
        CoreState::Starting => "Starting...",
        CoreState::Stopping => "Stopping...",
        CoreState::Error => "Error",
    }
}

/// Tray state passed to platform tray for menu construction
pub struct TrayState {
    pub proxy_groups: Vec<ProxyGroup>,
    pub profiles: Vec<ProfileItem>,
    pub current_mode: OutboundMode,
    pub sysproxy_enabled: bool,
    pub tun_enabled: bool,
    pub traffic_up: u64,
    pub traffic_down: u64,
    pub core_state: CoreState,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            proxy_groups: Vec::new(),
            profiles: Vec::new(),
            current_mode: OutboundMode::Rule,
            sysproxy_enabled: false,
            tun_enabled: false,
            traffic_up: 0,
            traffic_down: 0,
            core_state: CoreState::Stopped,
        }
    }
}

impl TrayState {
    pub fn new(
        proxy_groups: Vec<ProxyGroup>,
        profiles: Vec<ProfileItem>,
        current_mode: OutboundMode,
        sysproxy_enabled: bool,
        tun_enabled: bool,
        traffic_up: u64,
        traffic_down: u64,
        core_state: CoreState,
    ) -> Self {
        Self {
            proxy_groups,
            profiles,
            current_mode,
            sysproxy_enabled,
            tun_enabled,
            traffic_up,
            traffic_down,
            core_state,
        }
    }
}

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
    core_state: CoreState,
    platform_tray: Option<PlatformTray>,
    action_receiver: Option<mpsc::UnboundedReceiver<TrayActionEvent>>,
    main_window_handle: Option<gpui::AnyWindowHandle>,
}

impl TrayManager {
    pub fn new(
        core_manager: Arc<RwLock<CoreManager>>,
        config: Arc<RwLock<AppConfig>>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
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
            core_state: CoreState::Stopped,
            platform_tray: Some(PlatformTray::new(sender)),
            action_receiver: Some(receiver),
            main_window_handle: None,
        }
    }

    pub fn init_tray(&mut self, cx: &mut Context<Self>) -> anyhow::Result<()> {
        let state = TrayState::new(
            self.proxy_groups.clone(),
            self.profiles.clone(),
            self.current_mode,
            self.sysproxy_enabled,
            self.tun_enabled,
            self.traffic_up,
            self.traffic_down,
            self.core_state,
        );
        
        if let Some(platform_tray) = &mut self.platform_tray {
            platform_tray.init(&state)?;
        }
        self.start_event_processing(cx);
        Ok(())
    }

    pub fn set_main_window(&mut self, handle: gpui::AnyWindowHandle) {
        self.main_window_handle = Some(handle);
    }

    fn start_event_processing(&mut self, cx: &mut Context<Self>) {
        let receiver = self.action_receiver.take();
        let core_manager = self.core_manager.clone();
        let config = self.config.clone();
        
        cx.spawn(async move |this, cx| {
            if let Some(mut receiver) = receiver {
                while let Some(action) = receiver.recv().await {
                    match action {
                        TrayActionEvent::ShowWindow => {
                            let _ = this.update(cx, |this, _| {
                                let _ = &this.main_window_handle;
                            });
                        }
                        TrayActionEvent::HideWindow => {}
                        TrayActionEvent::ToggleSysProxy => {
                            let manager = core_manager.read();
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    manager.enable_sysproxy().await
                                })
                            });
                        }
                        TrayActionEvent::ToggleTun => {
                            let manager = core_manager.read();
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    manager.enable_tun().await
                                })
                            });
                        }
                        TrayActionEvent::SetRuleMode => {
                            let manager = core_manager.read();
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    manager.set_mode("rule").await
                                })
                            });
                        }
                        TrayActionEvent::SetGlobalMode => {
                            let manager = core_manager.read();
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    manager.set_mode("global").await
                                })
                            });
                        }
                        TrayActionEvent::SetDirectMode => {
                            let manager = core_manager.read();
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    manager.set_mode("direct").await
                                })
                            });
                        }
                        TrayActionEvent::SelectProxy { group, proxy } => {
                            let manager = core_manager.read();
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    manager.select_proxy(&group, &proxy).await
                                })
                            });
                        }
                        TrayActionEvent::SelectProfile { id } => {
                            let _ = id;
                        }
                        TrayActionEvent::Quit => {
                            let _ = cx.update(|cx| {
                                cx.quit();
                            });
                        }
                    }
                }
            }
        }).detach();
    }

    fn handle_tray_action(&mut self, action: TrayActionEvent, cx: &mut Context<Self>) {
        match action {
            TrayActionEvent::ShowWindow => {
                self.show_window(cx);
            }
            TrayActionEvent::HideWindow => {
                cx.dispatch_action(&HideWindow);
            }
            TrayActionEvent::ToggleSysProxy => {
                cx.dispatch_action(&ToggleSysProxy);
                self.sysproxy_enabled = !self.sysproxy_enabled;
                self.update_tray_menu(cx);
            }
            TrayActionEvent::ToggleTun => {
                cx.dispatch_action(&ToggleTun);
                self.tun_enabled = !self.tun_enabled;
                self.update_tray_menu(cx);
            }
            TrayActionEvent::SetRuleMode => {
                cx.dispatch_action(&SetRuleMode);
                self.current_mode = OutboundMode::Rule;
                self.update_tray_menu(cx);
            }
            TrayActionEvent::SetGlobalMode => {
                cx.dispatch_action(&SetGlobalMode);
                self.current_mode = OutboundMode::Global;
                self.update_tray_menu(cx);
            }
            TrayActionEvent::SetDirectMode => {
                cx.dispatch_action(&SetDirectMode);
                self.current_mode = OutboundMode::Direct;
                self.update_tray_menu(cx);
            }
            TrayActionEvent::SelectProxy { group, proxy } => {
                cx.dispatch_action(&SelectProxy { group, proxy });
            }
            TrayActionEvent::SelectProfile { id } => {
                cx.dispatch_action(&SelectProfile { id });
            }
            TrayActionEvent::Quit => {
                cx.dispatch_action(&Quit);
            }
        }
    }

    fn show_window(&mut self, _cx: &mut Context<Self>) {
        let _ = &self.main_window_handle;
    }

    fn update_tray_menu(&mut self, _cx: &mut Context<Self>) {
        if let Some(platform_tray) = &self.platform_tray {
            let state = TrayState::new(
                self.proxy_groups.clone(),
                self.profiles.clone(),
                self.current_mode,
                self.sysproxy_enabled,
                self.tun_enabled,
                self.traffic_up,
                self.traffic_down,
                self.core_state,
            );
            platform_tray.update(&state).ok();
        }
    }

    pub fn update_traffic(&mut self, up: u64, down: u64, cx: &mut Context<Self>) {
        self.traffic_up = up;
        self.traffic_down = down;
        self.update_tray_menu(cx);
    }

    pub fn update_state(&mut self, state_update: TrayStateUpdate, cx: &mut Context<Self>) {
        match state_update {
            TrayStateUpdate::ModeChanged(mode) => self.current_mode = mode,
            TrayStateUpdate::SysProxyChanged(enabled) => self.sysproxy_enabled = enabled,
            TrayStateUpdate::TunChanged(enabled) => self.tun_enabled = enabled,
            TrayStateUpdate::ProxyGroupsUpdated(groups) => self.proxy_groups = groups,
            TrayStateUpdate::ProfilesUpdated(profiles) => self.profiles = profiles,
            TrayStateUpdate::CoreStateChanged(core_state) => self.core_state = core_state,
        }
        self.update_tray_menu(cx);
    }

    pub fn get_state(&self) -> TrayState {
        TrayState::new(
            self.proxy_groups.clone(),
            self.profiles.clone(),
            self.current_mode,
            self.sysproxy_enabled,
            self.tun_enabled,
            self.traffic_up,
            self.traffic_down,
            self.core_state,
        )
    }
}

impl Render for TrayManager {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub enum TrayStateUpdate {
    ModeChanged(OutboundMode),
    SysProxyChanged(bool),
    TunChanged(bool),
    ProxyGroupsUpdated(Vec<ProxyGroup>),
    ProfilesUpdated(Vec<ProfileItem>),
    CoreStateChanged(CoreState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundMode {
    Rule,
    Global,
    Direct,
}

impl Default for OutboundMode {
    fn default() -> Self {
        OutboundMode::Rule
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

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn init_tray(
    core_manager: Arc<RwLock<CoreManager>>,
    config: Arc<RwLock<AppConfig>>,
    main_window: gpui::AnyWindowHandle,
    cx: &mut App,
) -> anyhow::Result<Entity<TrayManager>> {
    let tray_manager = cx.new(|cx| {
        let mut manager = TrayManager::new(core_manager, config);
        manager.set_main_window(main_window);
        manager.init_tray(cx).ok();
        manager
    });
    
    Ok(tray_manager)
}