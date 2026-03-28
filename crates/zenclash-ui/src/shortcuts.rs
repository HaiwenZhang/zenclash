use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    actions, div, prelude::FluentBuilder, App, Context, IntoElement,
    ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    ActiveTheme,
};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::HotKey,
};
use parking_lot::Mutex;

actions!(
    zenclash_shortcuts,
    [
        ShowWindowShortcut,
        ShowFloatingWindowShortcut,
        ToggleSysProxyShortcut,
        ToggleTunShortcut,
        SetRuleModeShortcut,
        SetGlobalModeShortcut,
        SetDirectModeShortcut,
        QuitWithoutCoreShortcut,
        RestartAppShortcut,
    ]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    ShowWindow,
    ShowFloatingWindow,
    ToggleSysProxy,
    ToggleTun,
    RuleMode,
    GlobalMode,
    DirectMode,
    QuitWithoutCore,
    RestartApp,
}

struct RegisteredHotkey {
    hotkey: HotKey,
    action: ShortcutAction,
}

struct ShortcutManagerState {
    manager: GlobalHotKeyManager,
    registered: HashMap<u32, RegisteredHotkey>,
}

pub struct ShortcutManager {
    action_map: HashMap<String, ShortcutAction>,
    state: Arc<Mutex<Option<ShortcutManagerState>>>,
    initialized: bool,
}

impl gpui::Global for ShortcutManager {}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut action_map = HashMap::new();
        action_map.insert("showWindow".to_string(), ShortcutAction::ShowWindow);
        action_map.insert(
            "showFloatingWindow".to_string(),
            ShortcutAction::ShowFloatingWindow,
        );
        action_map.insert("toggleSysProxy".to_string(), ShortcutAction::ToggleSysProxy);
        action_map.insert("toggleTun".to_string(), ShortcutAction::ToggleTun);
        action_map.insert("ruleMode".to_string(), ShortcutAction::RuleMode);
        action_map.insert("globalMode".to_string(), ShortcutAction::GlobalMode);
        action_map.insert("directMode".to_string(), ShortcutAction::DirectMode);
        action_map.insert(
            "quitWithoutCore".to_string(),
            ShortcutAction::QuitWithoutCore,
        );
        action_map.insert("restartApp".to_string(), ShortcutAction::RestartApp);

        Self {
            action_map,
            state: Arc::new(Mutex::new(None)),
            initialized: false,
        }
    }

    pub fn init(cx: &mut App) {
        let manager = Self::new();
        cx.set_global(manager);
        
        Self::setup_event_handler(cx);
        Self::register_defaults(cx);
    }

    fn setup_event_handler(cx: &mut App) {
        let state = cx.global::<ShortcutManager>().state.clone();
        
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state == HotKeyState::Pressed {
                let guard = state.lock();
                if let Some(manager_state) = guard.as_ref() {
                    if let Some(registered) = manager_state.registered.get(&event.id) {
                        Self::dispatch_action(registered.action);
                    }
                }
            }
        }));
    }

    fn dispatch_action(action: ShortcutAction) {
        match action {
            ShortcutAction::ShowWindow => {
                tracing::info!("Dispatching ShowWindow action");
            }
            ShortcutAction::ShowFloatingWindow => {
                tracing::info!("Dispatching ShowFloatingWindow action");
            }
            ShortcutAction::ToggleSysProxy => {
                tracing::info!("Dispatching ToggleSysProxy action");
            }
            ShortcutAction::ToggleTun => {
                tracing::info!("Dispatching ToggleTun action");
            }
            ShortcutAction::RuleMode => {
                tracing::info!("Dispatching RuleMode action");
            }
            ShortcutAction::GlobalMode => {
                tracing::info!("Dispatching GlobalMode action");
            }
            ShortcutAction::DirectMode => {
                tracing::info!("Dispatching DirectMode action");
            }
            ShortcutAction::QuitWithoutCore => {
                tracing::info!("Dispatching QuitWithoutCore action");
            }
            ShortcutAction::RestartApp => {
                tracing::info!("Dispatching RestartApp action");
            }
        }
    }

    fn register_defaults(cx: &mut App) {
        let defaults = [
            ("showWindow", "CmdOrCtrl+Shift+KeyZ"),
            ("toggleSysProxy", "CmdOrCtrl+Shift+KeyS"),
            ("toggleTun", "CmdOrCtrl+Shift+KeyT"),
            ("ruleMode", "CmdOrCtrl+Shift+KeyR"),
            ("globalMode", "CmdOrCtrl+Shift+KeyG"),
            ("directMode", "CmdOrCtrl+Shift+KeyD"),
        ];

        for (key, shortcut) in defaults {
            Self::register_shortcut(key, shortcut, cx);
        }
    }

    pub fn register_shortcut(key: &str, shortcut: &str, cx: &mut App) {
        let manager = cx.global::<ShortcutManager>();
        
        if let Some(action) = manager.action_map.get(key) {
            let action = *action;
            let state = manager.state.clone();
            let shortcut_str = shortcut.to_string();
            
            cx.spawn(async move |_cx| {
                register_global_shortcut(&shortcut_str, action, state).await;
            })
            .detach();
        }
    }

    pub fn unregister_all(cx: &mut App) {
        let manager = cx.global::<ShortcutManager>();
        let state = manager.state.clone();
        
        cx.spawn(async move |_cx| {
            unregister_all_shortcuts(state).await;
        })
        .detach();
    }

    pub fn check_macos_accessibility() -> bool {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("osascript")
                .arg("-e")
                .arg("tell application \"System Events\" to get frontmost process")
                .output()
                .is_ok()
        }
        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }

    pub fn get_default_shortcuts() -> HashMap<String, String> {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("showWindow".to_string(), "CmdOrCtrl+Shift+Z".to_string());
        shortcuts.insert("toggleSysProxy".to_string(), "CmdOrCtrl+Shift+S".to_string());
        shortcuts.insert("toggleTun".to_string(), "CmdOrCtrl+Shift+T".to_string());
        shortcuts.insert("ruleMode".to_string(), "CmdOrCtrl+Shift+R".to_string());
        shortcuts.insert("globalMode".to_string(), "CmdOrCtrl+Shift+G".to_string());
        shortcuts.insert("directMode".to_string(), "CmdOrCtrl+Shift+D".to_string());
        shortcuts.insert("quitWithoutCore".to_string(), "CmdOrCtrl+Q".to_string());
        shortcuts
    }
}

impl Render for ShortcutManager {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

async fn register_global_shortcut(
    shortcut: &str,
    action: ShortcutAction,
    state: Arc<Mutex<Option<ShortcutManagerState>>>,
) {
    let hotkey: HotKey = match shortcut.parse() {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to parse hotkey '{}': {}", shortcut, e);
            return;
        }
    };

    let mut guard = state.lock();
    
    if guard.is_none() {
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to create GlobalHotKeyManager: {}", e);
                return;
            }
        };
        *guard = Some(ShortcutManagerState {
            manager,
            registered: HashMap::new(),
        });
    }

    if let Some(manager_state) = guard.as_mut() {
        match manager_state.manager.register(hotkey) {
            Ok(_) => {
                manager_state.registered.insert(
                    hotkey.id(),
                    RegisteredHotkey { hotkey, action },
                );
                tracing::info!("Registered global hotkey: {} for action {:?}", shortcut, action);
            }
            Err(e) => {
                tracing::error!("Failed to register hotkey '{}': {}", shortcut, e);
            }
        }
    }
}

async fn unregister_all_shortcuts(state: Arc<Mutex<Option<ShortcutManagerState>>>) {
    let mut guard = state.lock();
    
    if let Some(manager_state) = guard.take() {
        let hotkeys: Vec<HotKey> = manager_state.registered.values().map(|r| r.hotkey).collect();
        
        if let Err(e) = manager_state.manager.unregister_all(&hotkeys) {
            tracing::error!("Failed to unregister all hotkeys: {}", e);
        } else {
            tracing::info!("Unregistered all global hotkeys");
        }
    }
}

pub struct ShortcutsPage {
    shortcuts: HashMap<String, String>,
    recording: Option<String>,
}

impl ShortcutsPage {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("showWindow".to_string(), String::new());
        shortcuts.insert("showFloatingWindow".to_string(), String::new());
        shortcuts.insert("toggleSysProxy".to_string(), String::new());
        shortcuts.insert("toggleTun".to_string(), String::new());
        shortcuts.insert("ruleMode".to_string(), String::new());
        shortcuts.insert("globalMode".to_string(), String::new());
        shortcuts.insert("directMode".to_string(), String::new());
        shortcuts.insert("quitWithoutCore".to_string(), String::new());
        shortcuts.insert("restartApp".to_string(), String::new());

        Self {
            shortcuts,
            recording: None,
        }
    }

    pub fn start_recording(&mut self, key: String, cx: &mut Context<Self>) {
        self.recording = Some(key);
        cx.notify();
    }

    pub fn stop_recording(&mut self, cx: &mut Context<Self>) {
        self.recording = None;
        cx.notify();
    }

    pub fn set_shortcut(&mut self, key: String, shortcut: String, cx: &mut Context<Self>) {
        self.shortcuts.insert(key, shortcut);
        cx.notify();
    }

    fn render_shortcut_row(&self, label: &str, key: &str, cx: &Context<Self>) -> impl IntoElement {
        let shortcut = self.shortcuts.get(key).cloned().unwrap_or_default();
        let is_recording = self.recording.as_ref() == Some(&key.to_string());
        let key = key.to_string();
        let key_for_clear = key.clone();
        let label = label.to_string();

        h_flex()
            .gap_4()
            .items_center()
            .child(div().w_48().child(label))
            .child(
                h_flex()
                    .gap_2()
                    .flex_1()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(cx.theme().muted.opacity(0.2))
                            .when(is_recording, |this| {
                                this.bg(cx.theme().primary.opacity(0.2))
                            })
                            .child(if shortcut.is_empty() {
                                if is_recording {
                                    "Press keys...".to_string()
                                } else {
                                    "Not set".to_string()
                                }
                            } else {
                                shortcut.clone()
                            }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("record-{}", key)))
                            .label(if is_recording { "Stop" } else { "Record" })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if is_recording {
                                    this.stop_recording(cx);
                                } else {
                                    this.start_recording(key.clone(), cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(SharedString::from(format!("clear-{}", key_for_clear)))
                            .ghost()
                            .label("Clear")
                            .when(!shortcut.is_empty(), |this| this)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.set_shortcut(key_for_clear.clone(), String::new(), cx);
                            })),
                    ),
            )
    }

    #[cfg(target_os = "macos")]
    fn render_accessibility_warning(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .text_color(cx.theme().warning)
            .child("Note: macOS requires Accessibility permissions for global shortcuts")
    }

    #[cfg(not(target_os = "macos"))]
    fn render_accessibility_warning(&self, _cx: &Context<Self>) -> impl IntoElement {
        div()
    }
}

impl Render for ShortcutsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_component::v_flex()
            .size_full()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Shortcuts"),
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child("Configure global keyboard shortcuts"),
            )
            .child(self.render_accessibility_warning(cx))
            .child(
                gpui_component::v_flex()
                    .gap_3()
                    .child(self.render_shortcut_row("Show Window", "showWindow", cx))
                    .child(self.render_shortcut_row(
                        "Show Floating Window",
                        "showFloatingWindow",
                        cx,
                    ))
                    .child(self.render_shortcut_row("Toggle System Proxy", "toggleSysProxy", cx))
                    .child(self.render_shortcut_row("Toggle TUN Mode", "toggleTun", cx))
                    .child(self.render_shortcut_row("Rule Mode", "ruleMode", cx))
                    .child(self.render_shortcut_row("Global Mode", "globalMode", cx))
                    .child(self.render_shortcut_row("Direct Mode", "directMode", cx))
                    .child(self.render_shortcut_row("Quit Without Core", "quitWithoutCore", cx))
                    .child(self.render_shortcut_row("Restart App", "restartApp", cx)),
            )
            .child(
                div().mt_auto().child(
                    Button::new("save-shortcuts")
                        .primary()
                        .label("Save Shortcuts")
                        .on_click(cx.listener(|_, _, _, _| {
                        })),
                ),
            )
    }
}