use std::sync::Arc;

use gpui::{
    actions, div, prelude::FluentBuilder, px, App, AppContext, Context, Entity, Focusable, IntoElement,
    InteractiveElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled,
    Window, WindowBounds, WindowOptions,
};
use gpui_component::{h_flex, v_flex, ActiveTheme, Root, Theme, ThemeMode, TitleBar};
use parking_lot::RwLock;

use crate::components::sidebar::{OutboundMode, ZenSidebar};
use crate::pages::{
    backup::BackupPage, connections::ConnectionsPage, dns::DnsPage, logs::LogsPage,
    profiles::ProfilesPage, proxies::ProxiesPage, rules::RulesPage, settings::SettingsPage, Page,
};
use zenclash_core::prelude::{AppConfig, CoreManager, CoreState};

actions!(
    zenclash,
    [
        Quit,
        ToggleSidebar,
        NavigateProxies,
        NavigateProfiles,
        NavigateConnections,
        NavigateRules,
        NavigateLogs,
        NavigateSettings,
        NavigateDns,
        NavigateBackup,
        ToggleSysProxy,
        ToggleTun,
        StartCore,
        StopCore,
    ]
);

pub struct ZenClashApp {
    core_manager: Arc<RwLock<CoreManager>>,
    config: Arc<RwLock<AppConfig>>,
    core_state: CoreState,
    current_page: Page,
    sidebar_collapsed: bool,
    sidebar_width: gpui::Pixels,
    focus_handle: gpui::FocusHandle,
    sysproxy_enabled: bool,
    tun_enabled: bool,
    outbound_mode: OutboundMode,
    proxies_page: Entity<ProxiesPage>,
    profiles_page: Entity<ProfilesPage>,
    connections_page: Entity<ConnectionsPage>,
    rules_page: Entity<RulesPage>,
    logs_page: Entity<LogsPage>,
    settings_page: Entity<SettingsPage>,
    dns_page: Entity<DnsPage>,
    backup_page: Entity<BackupPage>,
}

impl ZenClashApp {
    pub fn new(
        core_manager: Arc<RwLock<CoreManager>>,
        config: Arc<RwLock<AppConfig>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let proxies_page = cx.new(|cx| ProxiesPage::new(core_manager.clone(), window, cx));
        let profiles_page = cx.new(|cx| ProfilesPage::new(window, cx));
        let connections_page = cx.new(|cx| ConnectionsPage::new(core_manager.clone(), cx));
        let rules_page = cx.new(|cx| RulesPage::new(window, cx));
        let logs_page = cx.new(|cx| LogsPage::new(cx));
        let settings_page = cx.new(|cx| SettingsPage::new(cx));
        let dns_page = cx.new(|cx| DnsPage::new(window, cx));
        let backup_page = cx.new(|cx| BackupPage::new(window, cx));

        Self {
            core_manager,
            config,
            core_state: CoreState::Stopped,
            current_page: Page::default(),
            sidebar_collapsed: false,
            sidebar_width: px(220.),
            focus_handle: cx.focus_handle(),
            sysproxy_enabled: false,
            tun_enabled: false,
            outbound_mode: OutboundMode::default(),
            proxies_page,
            profiles_page,
            connections_page,
            rules_page,
            logs_page,
            settings_page,
            dns_page,
            backup_page,
        }
    }

    pub fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        self.current_page = page;
        cx.notify();
    }

    pub fn core_state(&self) -> CoreState {
        self.core_state
    }

    pub fn start_core(&self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let result = {
                let manager = core_manager.read();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        manager.start().await
                    })
                })
            };
            
            if let Err(e) = result {
                eprintln!("Failed to start core: {}", e);
            }
            
            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn stop_core(&self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let result = {
                let manager = core_manager.read();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        manager.stop().await
                    })
                })
            };
            
            if let Err(e) = result {
                eprintln!("Failed to stop core: {}", e);
            }
            
            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }

    pub fn toggle_sysproxy(&mut self, cx: &mut Context<Self>) {
        self.sysproxy_enabled = !self.sysproxy_enabled;
        let core_manager = self.core_manager.clone();
        let enabled = self.sysproxy_enabled;

        cx.spawn(async move |_, _| {
            let manager = core_manager.read();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if enabled {
                        manager.enable_sysproxy().await.ok();
                    } else {
                        manager.disable_sysproxy().await.ok();
                    }
                })
            });
        })
        .detach();

        cx.notify();
    }

    pub fn toggle_tun(&mut self, cx: &mut Context<Self>) {
        self.tun_enabled = !self.tun_enabled;
        let core_manager = self.core_manager.clone();
        let enabled = self.tun_enabled;

        cx.spawn(async move |_, _| {
            let manager = core_manager.read();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if enabled {
                        manager.enable_tun().await.ok();
                    } else {
                        manager.disable_tun().await.ok();
                    }
                })
            });
        })
        .detach();

        cx.notify();
    }

    pub fn set_outbound_mode(&mut self, mode: OutboundMode, cx: &mut Context<Self>) {
        self.outbound_mode = mode;
        let core_manager = self.core_manager.clone();

        cx.spawn(async move |_, _| {
            let manager = core_manager.read();
            let mode_str = match mode {
                OutboundMode::Rule => "rule",
                OutboundMode::Global => "global",
                OutboundMode::Direct => "direct",
            };
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    manager.set_mode(mode_str).await.ok();
                })
            });
        })
        .detach();

        cx.notify();
    }

    fn on_navigate_proxies(&mut self, _: &NavigateProxies, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate(Page::Proxies, cx);
    }

    fn on_navigate_profiles(
        &mut self,
        _: &NavigateProfiles,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate(Page::Profiles, cx);
    }

    fn on_navigate_connections(
        &mut self,
        _: &NavigateConnections,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate(Page::Connections, cx);
    }

    fn on_navigate_logs(&mut self, _: &NavigateLogs, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate(Page::Logs, cx);
    }

    fn on_navigate_settings(
        &mut self,
        _: &NavigateSettings,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate(Page::Settings, cx);
    }

    fn on_navigate_rules(&mut self, _: &NavigateRules, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate(Page::Rules, cx);
    }

    fn on_navigate_dns(&mut self, _: &NavigateDns, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate(Page::Dns, cx);
    }

    fn on_navigate_backup(&mut self, _: &NavigateBackup, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate(Page::Backup, cx);
    }

    fn on_toggle_sidebar(&mut self, _: &ToggleSidebar, _: &mut Window, cx: &mut Context<Self>) {
        self.toggle_sidebar(cx);
    }

    fn on_toggle_sysproxy(&mut self, _: &ToggleSysProxy, _: &mut Window, cx: &mut Context<Self>) {
        self.toggle_sysproxy(cx);
    }

    fn on_toggle_tun(&mut self, _: &ToggleTun, _: &mut Window, cx: &mut Context<Self>) {
        self.toggle_tun(cx);
    }

    fn on_start_core(&mut self, _: &StartCore, _: &mut Window, cx: &mut Context<Self>) {
        self.start_core(cx);
    }

    fn on_stop_core(&mut self, _: &StopCore, _: &mut Window, cx: &mut Context<Self>) {
        self.stop_core(cx);
    }

    fn on_quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn render_sidebar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        ZenSidebar::new()
            .current_page(self.current_page)
            .collapsed(self.sidebar_collapsed)
            .sysproxy_enabled(self.sysproxy_enabled)
            .tun_enabled(self.tun_enabled)
            .outbound_mode(self.outbound_mode)
            .core_state(self.core_state)
    }

    fn render_content(&self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.current_page {
            Page::Proxies => self.proxies_page.clone().into_any_element(),
            Page::Profiles => self.profiles_page.clone().into_any_element(),
            Page::Connections => self.connections_page.clone().into_any_element(),
            Page::Rules => self.rules_page.clone().into_any_element(),
            Page::Logs => self.logs_page.clone().into_any_element(),
            Page::Settings => self.settings_page.clone().into_any_element(),
            Page::Dns => self.dns_page.clone().into_any_element(),
            Page::Backup => self.backup_page.clone().into_any_element(),
            Page::Mihomo => div().into_any_element(),
            Page::Tun => div().into_any_element(),
            Page::Sniffer => div().into_any_element(),
            Page::Resources => div().into_any_element(),
            Page::Override => div().into_any_element(),
            Page::Sysproxy => div().into_any_element(),
            Page::SubStore => div().into_any_element(),
        }
    }
}

impl Focusable for ZenClashApp {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ZenClashApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .id("zenclash-app")
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.render_sidebar(cx))
            .child(
                v_flex()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(self.render_content(window, cx)),
            )
    }
}

pub struct WindowConfig {
    pub title: SharedString,
    pub width: f32,
    pub height: f32,
    pub centered: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: SharedString::from("ZenClash"),
            width: 1200.,
            height: 800.,
            centered: true,
        }
    }
}

pub fn create_main_window(
    core_manager: Arc<RwLock<CoreManager>>,
    config: Arc<RwLock<AppConfig>>,
    window_config: WindowConfig,
    cx: &mut App,
) -> anyhow::Result<()> {
    let options = WindowOptions {
        window_bounds: if window_config.centered {
            Some(WindowBounds::centered(
                gpui::size(px(window_config.width), px(window_config.height)),
                cx,
            ))
        } else {
            None
        },
        titlebar: Some(TitleBar::title_bar_options()),
        ..Default::default()
    };

    cx.spawn(async move |cx| {
        cx.open_window(options, |window, cx| {
            Theme::change(ThemeMode::Dark, Some(window), cx);
            window.set_window_title(&window_config.title);
            window.activate_window();

            let view = cx.new(|cx| ZenClashApp::new(core_manager, config, window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
        .expect("Failed to open main window");
    })
    .detach();

    Ok(())
}

pub fn init(cx: &mut App) {
    gpui_component::init(cx);
    cx.on_action(|_: &Quit, cx| {
        cx.quit();
    });
}
