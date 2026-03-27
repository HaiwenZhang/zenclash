use std::collections::HashMap;

use gpui::{prelude::FluentBuilder, InteractiveElement, SharedString, actions, div, App, Context, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectState},
    switch::Switch,
    v_flex, ActiveTheme,
};

use zenclash_core::prelude::AppConfig;

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

pub struct ShortcutManager {
    shortcuts: HashMap<String, ShortcutAction>,
    recording: Option<String>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();

        shortcuts.insert("showWindow".to_string(), ShortcutAction::ShowWindow);
        shortcuts.insert(
            "showFloatingWindow".to_string(),
            ShortcutAction::ShowFloatingWindow,
        );
        shortcuts.insert("toggleSysProxy".to_string(), ShortcutAction::ToggleSysProxy);
        shortcuts.insert("toggleTun".to_string(), ShortcutAction::ToggleTun);
        shortcuts.insert("ruleMode".to_string(), ShortcutAction::RuleMode);
        shortcuts.insert("globalMode".to_string(), ShortcutAction::GlobalMode);
        shortcuts.insert("directMode".to_string(), ShortcutAction::DirectMode);
        shortcuts.insert(
            "quitWithoutCore".to_string(),
            ShortcutAction::QuitWithoutCore,
        );
        shortcuts.insert("restartApp".to_string(), ShortcutAction::RestartApp);

        Self {
            shortcuts,
            recording: None,
        }
    }

    pub fn register_shortcut(&mut self, key: &str, shortcut: String, cx: &mut Context<Self>) {
        if let Some(action) = self.shortcuts.get(key) {
            let action = *action;
            cx.spawn(async move |_, _| {
                register_global_shortcut(&shortcut, action).await;
            })
            .detach();
        }
    }

    pub fn unregister_all(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |_, _| {
            unregister_all_shortcuts().await;
        })
        .detach();
    }
}

impl Render for ShortcutManager {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Debug, Clone, Copy)]
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

pub struct ShortcutsPage {
    shortcuts: HashMap<String, String>,
    recording: Option<String>,
}

impl ShortcutsPage {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
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
}

impl Render for ShortcutsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
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
            .child(
                v_flex()
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
                            // Save shortcuts
                        })),
                ),
            )
    }
}

async fn register_global_shortcut(shortcut: &str, action: ShortcutAction) {
    // Platform-specific implementation
    #[cfg(target_os = "macos")]
    {
        // macOS implementation using CGEventTap or similar
    }
    #[cfg(target_os = "windows")]
    {
        // Windows implementation using RegisterHotKey
    }
    #[cfg(target_os = "linux")]
    {
        // Linux implementation using x11 or wayland
    }
}

async fn unregister_all_shortcuts() {
    // Platform-specific implementation
}
