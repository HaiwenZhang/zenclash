use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, App, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::Input,
    select::Select,
    switch::Switch,
    tab::Tab,
    tab::TabBar,
    v_flex, ActiveTheme, Disableable, Sizable,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::Page;
use crate::pages::PageTrait;
use zenclash_core::prelude::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Mihomo,
    WebDav,
    Shortcuts,
    SubStore,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralSettings {
    pub language: String,
    pub auto_launch: bool,
    pub auto_check_update: bool,
    pub silent_start: bool,
    pub show_tray: bool,
    pub show_floating_window: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MihomoSettings {
    pub delay_test_url: String,
    pub delay_test_timeout: u32,
    pub user_agent: String,
    pub cpu_priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebDavSettings {
    pub url: String,
    pub username: String,
    pub password: String,
    pub backup_cron: String,
    pub auto_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShortcutSettings {
    pub show_window: String,
    pub toggle_sysproxy: String,
    pub toggle_tun: String,
    pub rule_mode: String,
    pub global_mode: String,
    pub direct_mode: String,
}

pub struct SettingsPage {
    config: Arc<RwLock<AppConfig>>,
    current_tab: SettingsTab,
    general: GeneralSettings,
    mihomo: MihomoSettings,
    webdav: WebDavSettings,
    shortcuts: ShortcutSettings,
    changed: bool,
    focus_handle: FocusHandle,
}

impl SettingsPage {
    pub fn new(config: Arc<RwLock<AppConfig>>, cx: &mut Context<Self>) -> Self {
        let app_config = config.read().clone();
        let general = GeneralSettings {
            language: app_config.language.clone(),
            auto_launch: app_config.auto_launch,
            auto_check_update: true,
            silent_start: app_config.silent_start,
            show_tray: true,
            show_floating_window: false,
            theme: app_config.theme.clone(),
        };
        let mihomo = MihomoSettings {
            delay_test_url: app_config.delay_test_url.clone(),
            delay_test_timeout: 5000,
            user_agent: String::new(),
            cpu_priority: "normal".into(),
        };
        let webdav = WebDavSettings::default();
        let shortcuts = ShortcutSettings {
            show_window: "CmdOrCtrl+Shift+W".into(),
            toggle_sysproxy: "CmdOrCtrl+Shift+S".into(),
            toggle_tun: "CmdOrCtrl+Shift+T".into(),
            rule_mode: "CmdOrCtrl+Shift+R".into(),
            global_mode: "CmdOrCtrl+Shift+G".into(),
            direct_mode: "CmdOrCtrl+Shift+D".into(),
        };

        Self {
            config,
            current_tab: SettingsTab::default(),
            general,
            mihomo,
            webdav,
            shortcuts,
            changed: false,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn save(&self, _cx: &mut Context<Self>) {
        let mut config = self.config.write();
        config.language = self.general.language.clone();
        config.auto_launch = self.general.auto_launch;
        config.silent_start = self.general.silent_start;
        config.theme = self.general.theme.clone();
        config.delay_test_url = self.mihomo.delay_test_url.clone();
        if let Err(e) = config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }

    fn render_general_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("General"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Language"))
                    .child(
                        h_flex()
                            .gap_1()
                            .children(["en", "zh-CN", "zh-TW"].iter().map(|lang| {
                                let is_active = self.general.language == *lang;
                                let lang_label = match *lang {
                                    "en" => "English",
                                    "zh-CN" => "简体中文",
                                    "zh-TW" => "繁體中文",
                                    _ => *lang,
                                };
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(is_active, |this| {
                                        this.bg(theme.primary).text_color(theme.primary_foreground)
                                    })
                                    .when(!is_active, |this| {
                                        this.text_color(theme.muted_foreground)
                                            .hover(|this| this.bg(theme.muted))
                                    })
                                    .text_xs()
                                    .child(lang_label)
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Auto Launch"))
                    .child(
                        Switch::new("auto-launch")
                            .with_size(gpui_component::Size::Small)
                            .checked(self.general.auto_launch),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Auto Check Update"))
                    .child(
                        Switch::new("auto-check-update")
                            .with_size(gpui_component::Size::Small)
                            .checked(self.general.auto_check_update),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Silent Start"))
                    .child(
                        Switch::new("silent-start")
                            .with_size(gpui_component::Size::Small)
                            .checked(self.general.silent_start),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Show Tray Icon"))
                    .child(
                        Switch::new("show-tray")
                            .with_size(gpui_component::Size::Small)
                            .checked(self.general.show_tray),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Show Floating Window"))
                    .child(
                        Switch::new("show-floating")
                            .with_size(gpui_component::Size::Small)
                            .checked(self.general.show_floating_window),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Theme"))
                    .child(
                        h_flex()
                            .gap_1()
                            .children(["system", "light", "dark"].iter().map(|t| {
                                let is_active = self.general.theme == *t;
                                let label = match *t {
                                    "system" => "Auto",
                                    "light" => "Light",
                                    "dark" => "Dark",
                                    _ => *t,
                                };
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .when(is_active, |this| {
                                        this.bg(theme.primary).text_color(theme.primary_foreground)
                                    })
                                    .when(!is_active, |this| {
                                        this.text_color(theme.muted_foreground)
                                            .hover(|this| this.bg(theme.muted))
                                    })
                                    .text_xs()
                                    .child(label)
                            })),
                    ),
            )
    }

    fn render_mihomo_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("Mihomo Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Delay Test URL"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.mihomo.delay_test_url.clone()),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Delay Test Timeout"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("{} ms", self.mihomo.delay_test_timeout)),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("CPU Priority"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.mihomo.cpu_priority.clone()),
                    ),
            )
    }

    fn render_webdav_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("WebDAV Backup"),
            )
            .when(self.webdav.url.is_empty(), |this| {
                this.child(
                    div()
                        .py_4()
                        .text_center()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("Configure WebDAV to enable cloud backup"),
                )
            })
            .when(!self.webdav.url.is_empty(), |this| {
                this.child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(div().text_xs().child("URL"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(self.webdav.url.clone()),
                                ),
                        )
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(div().text_xs().child("Username"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(self.webdav.username.clone()),
                                ),
                        )
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(div().text_xs().child("Auto Backup"))
                                .child(
                                    Switch::new("webdav-auto")
                                        .with_size(gpui_component::Size::XSmall)
                                        .checked(self.webdav.auto_backup),
                                ),
                        ),
                )
            })
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("webdav-config")
                            .with_size(gpui_component::Size::XSmall)
                            .child("Configure"),
                    )
                    .when(!self.webdav.url.is_empty(), |this| {
                        this.child(
                            Button::new("webdav-backup")
                                .with_size(gpui_component::Size::XSmall)
                                .child("Backup Now"),
                        )
                        .child(
                            Button::new("webdav-restore")
                                .with_size(gpui_component::Size::XSmall)
                                .child("Restore"),
                        )
                    }),
            )
    }

    fn render_shortcuts_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let shortcuts = [
            ("Show Window", &self.shortcuts.show_window),
            ("Toggle System Proxy", &self.shortcuts.toggle_sysproxy),
            ("Toggle TUN", &self.shortcuts.toggle_tun),
            ("Rule Mode", &self.shortcuts.rule_mode),
            ("Global Mode", &self.shortcuts.global_mode),
            ("Direct Mode", &self.shortcuts.direct_mode),
        ];

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("Keyboard Shortcuts"),
            )
            .children(shortcuts.into_iter().map(|(label, value)| {
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child(label))
                    .when(value.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("Not set"),
                        )
                    })
                    .when(!value.is_empty(), |this| {
                        this.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded(theme.radius)
                                .bg(theme.muted)
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(value.clone()),
                        )
                    })
            }))
    }
}

impl PageTrait for SettingsPage {
    fn title() -> &'static str {
        "Settings"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Settings
    }
}

impl Focusable for SettingsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .size_full()
            .overflow_y_hidden()
            .gap_4()
            .p_4()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Settings"),
                    )
                    .child(
                        Button::new("save")
                            .child("Save")
                            .primary()
                            .when(!self.changed, |this| this.disabled(true))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.save(cx);
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_1()
                    .p_1()
                    .rounded(theme.radius)
                    .bg(theme.muted)
                    .children(
                        [
                            SettingsTab::General,
                            SettingsTab::Mihomo,
                            SettingsTab::WebDav,
                            SettingsTab::Shortcuts,
                        ]
                        .into_iter()
                        .map(|tab| {
                            let is_active = self.current_tab == tab;
                            let label = match tab {
                                SettingsTab::General => "General",
                                SettingsTab::Mihomo => "Mihomo",
                                SettingsTab::WebDav => "WebDAV",
                                SettingsTab::Shortcuts => "Shortcuts",
                                SettingsTab::SubStore => "SubStore",
                            };
                            div()
                                .px_3()
                                .py_1()
                                .rounded(theme.radius)
                                .when(is_active, |this| {
                                    this.bg(theme.background).text_color(theme.foreground)
                                })
                                .when(!is_active, |this| {
                                    this.text_color(theme.muted_foreground)
                                        .hover(|this| this.bg(theme.transparent))
                                })
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(label)
                        }),
                    ),
            )
            .child(match self.current_tab {
                SettingsTab::General => self.render_general_section(cx).into_any_element(),
                SettingsTab::Mihomo => self.render_mihomo_section(cx).into_any_element(),
                SettingsTab::WebDav => self.render_webdav_section(cx).into_any_element(),
                SettingsTab::Shortcuts => self.render_shortcuts_section(cx).into_any_element(),
                SettingsTab::SubStore => div()
                    .child("SubStore settings coming soon")
                    .into_any_element(),
            })
    }
}
