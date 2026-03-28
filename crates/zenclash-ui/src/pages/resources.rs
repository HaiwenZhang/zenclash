use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, AppContext, Context,
    IntoElement, ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::Button, h_flex, v_flex, ActiveTheme, Disableable, Sizable,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use zenclash_core::prelude::CoreManager;

use crate::pages::PageTrait;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeoDataInfo {
    pub name: String,
    pub geo_type: String,
    pub size: u64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderInfo {
    pub name: String,
    pub provider_type: String,
    pub vehicle_type: String,
    pub updated_at: Option<String>,
    pub count: usize,
}

pub struct ResourcesPage {
    core_manager: Arc<RwLock<CoreManager>>,
    geo_data: Vec<GeoDataInfo>,
    proxy_providers: Vec<ProviderInfo>,
    updating_geo: bool,
}

impl ResourcesPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, cx: &mut Context<Self>) -> Self {
        let mut page = Self {
            core_manager,
            geo_data: Vec::new(),
            proxy_providers: Vec::new(),
            updating_geo: false,
        };
        page.refresh_data(cx);
        page
    }

    fn refresh_data(&mut self, cx: &mut Context<Self>) {
        let core_manager = self.core_manager.clone();
        
        cx.spawn(async move |this, cx| {
            let providers_result = {
                let manager = core_manager.read();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        manager.get_providers_proxies().await
                    })
                })
            };

            match providers_result {
                Ok(providers) => {
                    let provider_infos: Vec<ProviderInfo> = providers.providers.into_iter().map(|(name, p)| {
                        ProviderInfo {
                            name,
                            provider_type: p.provider_type,
                            vehicle_type: p.vehicle_type.unwrap_or_default(),
                            updated_at: None,
                            count: p.proxies.len(),
                        }
                    }).collect();
                    
                    let _ = this.update(cx, |this, cx| {
                        this.proxy_providers = provider_infos;
                        cx.notify();
                    });
                }
                Err(_) => {}
            }
        })
        .detach();
    }

    fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        }
    }

    fn upgrade_geo(&mut self, cx: &mut Context<Self>) {
        self.updating_geo = true;
        cx.notify();
        
        let core_manager = self.core_manager.clone();
        
        cx.spawn(async move |this, cx| {
            let result = {
                let manager = core_manager.read();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        manager.upgrade_geo().await
                    })
                })
            };
            
            let _ = this.update(cx, |this, cx| {
                this.updating_geo = false;
                if result.is_ok() {
                    this.geo_data = vec![
                        GeoDataInfo {
                            name: "Country.mmdb".into(),
                            geo_type: "mmdb".into(),
                            size: 5_800_000,
                            updated_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                        },
                        GeoDataInfo {
                            name: "GeoSite.dat".into(),
                            geo_type: "geosite".into(),
                            size: 4_200_000,
                            updated_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                        },
                    ];
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn render_geo_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("GeoIP / GeoSite Data"),
                    )
                    .child(
                        Button::new("update-geo")
                            .child(if self.updating_geo { "Updating..." } else { "Update All" })
                            .when(self.updating_geo, |this| this.disabled(true))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.upgrade_geo(cx);
                            })),
                    ),
            )
            .when(self.geo_data.is_empty(), |this| {
                this.child(
                    div()
                        .py_4()
                        .text_center()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("No GeoIP data found. Click 'Update All' to download."),
                )
            })
            .children(self.geo_data.iter().map(|geo| {
                v_flex()
                    .gap_1()
                    .p_2()
                    .rounded(theme.radius)
                    .bg(theme.muted)
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child(div().child(geo.name.clone())),
                                    )
                                    .child(
                                        div()
                                            .px_1()
                                            .rounded(theme.radius)
                                            .bg(theme.primary)
                                            .text_xs()
                                            .text_color(theme.primary_foreground)
                                            .child(geo.geo_type.to_uppercase()),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("Size: {}", Self::format_size(geo.size)))
                            .when_some(geo.updated_at.as_ref(), |this, date| {
                                this.child(format!("Updated: {}", date))
                            }),
                    )
            }))
    }

    fn render_provider_section(
        &self,
        title: &str,
        providers: &[ProviderInfo],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .gap_2()
            .p_4()
            .rounded(theme.radius)
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(title.to_string()),
                    )
                    .child(
                        Button::new(SharedString::from(format!("refresh-{}", title)))
                            .with_size(gpui_component::Size::XSmall)
                            .child("Refresh")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh_data(cx);
                            })),
                    ),
            )
            .when(providers.is_empty(), |this| {
                this.child(
                    div()
                        .py_4()
                        .text_center()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("No providers configured"),
                )
            })
            .children(providers.iter().map(|provider| {
                v_flex()
                    .gap_1()
                    .p_2()
                    .rounded(theme.radius)
                    .bg(theme.muted)
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div().text_sm().child(div().child(provider.name.clone())),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child(format!("{} proxies", provider.count)),
                                    ),
                            )
                            .child(
                                div()
                                    .px_1()
                                    .rounded(theme.radius)
                                    .bg(theme.secondary)
                                    .text_xs()
                                    .text_color(theme.secondary_foreground)
                                    .child(provider.provider_type.clone()),
                            ),
                    )
                    .when_some(provider.updated_at.as_ref(), |this, date| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("Updated: {}", date)),
                        )
                    })
            }))
    }
}

impl PageTrait for ResourcesPage {
    fn title() -> &'static str {
        "Resources"
    }

    fn icon() -> gpui_component::IconName {
        gpui_component::IconName::Inbox
    }
}

impl Render for ResourcesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _theme = cx.theme().clone();
        let proxy_providers = self.proxy_providers.clone();

        v_flex()
            .size_full()
            .overflow_y_hidden()
            .gap_4()
            .p_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Resources"),
            )
            .child(self.render_geo_section(cx))
            .child(self.render_provider_section("Proxy Providers", &proxy_providers, cx))
    }
}