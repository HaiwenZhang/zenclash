use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, AppContext, Context, Entity,
    IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    switch::Switch,
    v_flex, ActiveTheme, Disableable, Sizable,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::pages::PageTrait;
use zenclash_core::prelude::{AppConfig, CoreManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TunStack {
    #[default]
    Mixed,
    Gvisor,
    System,
}

impl TunStack {
    pub fn as_str(&self) -> &'static str {
        match self {
            TunStack::Mixed => "mixed",
            TunStack::Gvisor => "gvisor",
            TunStack::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "gvisor" => TunStack::Gvisor,
            "system" => TunStack::System,
            _ => TunStack::Mixed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunSettings {
    pub enabled: bool,
    pub stack: TunStack,
    pub device: String,
    pub auto_route: bool,
    pub auto_redirect: bool,
    pub auto_detect_interface: bool,
    pub dns_hijack: Vec<String>,
    pub strict_route: bool,
    pub route_exclude_address: Vec<String>,
    pub mtu: u32,
}

impl Default for TunSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            stack: TunStack::default(),
            device: if cfg!(target_os = "macos") {
                "utun1500".into()
            } else {
                "Mihomo".into()
            },
            auto_route: true,
            auto_redirect: false,
            auto_detect_interface: true,
            dns_hijack: vec!["any:53".into()],
            strict_route: false,
            route_exclude_address: vec![],
            mtu: 1500,
        }
    }
}

impl TunSettings {
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            enabled: config.tun_mode,
            ..Default::default()
        }
    }

    pub fn to_app_config_patch(&self) -> zenclash_core::prelude::AppConfigPatch {
        zenclash_core::prelude::AppConfigPatch {
            tun_mode: Some(self.enabled),
            ..Default::default()
        }
    }
}

pub struct TunPage {
    core_manager: Arc<RwLock<CoreManager>>,
    settings: Entity<TunSettings>,
    has_permission: bool,
}

impl TunPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, cx: &mut Context<Self>) -> Self {
        let settings = AppConfig::load().ok();
        let tun_settings = settings
            .map(|s| TunSettings::from_app_config(&s))
            .unwrap_or_default();

        Self {
            core_manager,
            settings: cx.new(|_| tun_settings),
            has_permission: false,
        }
    }

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let settings = self.settings.read(cx).clone();
        let enabled = settings.enabled;
        
        let mut config = AppConfig::load().unwrap_or_default();
        let patch = settings.to_app_config_patch();
        patch.apply(&mut config);
        config.save().ok();

        let core_manager = self.core_manager.clone();
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

    fn toggle_tun(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.settings.update(cx, |s, cx| {
            s.enabled = enabled;
            cx.notify();
        });

        let core_manager = self.core_manager.clone();
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

    fn render_permission_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child("TUN Permission"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child(if self.has_permission {
                        "Core has TUN permission"
                    } else {
                        "Core needs permission for TUN mode"
                    }))
                    .child(
                        Button::new("grant-permission")
                            .child("Grant Permission")
                            .when(self.has_permission, |this| this.disabled(true)),
                    ),
            )
            .when(cfg!(target_os = "windows"), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .py_2()
                        .child(div().text_sm().child("Windows Firewall"))
                        .child(Button::new("setup-firewall").child("Setup Firewall")),
                )
            })
    }

    fn render_basic_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);
        let stack_str = settings.stack.as_str().to_string();

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
                    .child("Basic Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("TUN Mode"))
                    .child(
                        Switch::new("tun-enabled")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.enabled)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.toggle_tun(*checked, cx);
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Stack Mode"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.primary)
                            .child(stack_str),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Device Name"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(div().child(settings.device.clone())),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("MTU"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(settings.mtu.to_string()),
                    ),
            )
    }

    fn render_route_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let settings = self.settings.read(cx);

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
                    .child("Route Settings"),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Auto Route"))
                    .child(
                        Switch::new("auto-route")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.auto_route)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.auto_route = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .when(cfg!(target_os = "linux"), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .py_2()
                        .child(div().text_sm().child("Auto Redirect"))
                        .child(
                            Switch::new("auto-redirect")
                                .with_size(gpui_component::Size::Small)
                                .checked(settings.auto_redirect)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.settings.update(cx, |s, cx| {
                                        s.auto_redirect = *checked;
                                        cx.notify();
                                    });
                                })),
                        ),
                )
            })
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Auto Detect Interface"))
                    .child(
                        Switch::new("auto-detect")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.auto_detect_interface)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.auto_detect_interface = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("Strict Route"))
                    .child(
                        Switch::new("strict-route")
                            .with_size(gpui_component::Size::Small)
                            .checked(settings.strict_route)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.settings.update(cx, |s, cx| {
                                    s.strict_route = *checked;
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_2()
                    .child(div().text_sm().child("DNS Hijack"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(settings.dns_hijack.join(", ")),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .py_2()
                    .child(div().text_sm().child("Exclude Addresses"))
                    .when(settings.route_exclude_address.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("No excluded addresses"),
                        )
                    })
                    .children(settings.route_exclude_address.iter().map(|addr| {
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(addr.clone())
                    })),
            )
    }

    fn render_dns_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child("DNS Settings"),
            )
            .when(cfg!(target_os = "macos"), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .py_2()
                        .child(div().text_sm().child("Auto Set DNS (macOS)"))
                        .child(
                            Switch::new("auto-set-dns")
                                .with_size(gpui_component::Size::Small)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.settings.update(cx, |_s, cx| {
                                        cx.notify();
                                    });
                                })),
                        ),
                )
            })
    }
}

impl PageTrait for TunPage {
    fn title() -> &'static str {
        "TUN"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Map
    }
}

impl Render for TunPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child("TUN Mode Settings"),
                    )
                    .child(
                        Button::new("save")
                            .child("Save & Restart")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.save_settings(cx);
                            })),
                    ),
            )
            .child(self.render_permission_section(cx))
            .child(self.render_basic_section(cx))
            .child(self.render_route_section(cx))
            .child(self.render_dns_section(cx))
    }
}