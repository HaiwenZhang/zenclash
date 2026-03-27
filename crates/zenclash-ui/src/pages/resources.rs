use gpui::{
    div, prelude::FluentBuilder, px, App, AppContext, Context, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    button::Button, h_flex, progress::Progress, v_flex, ActiveTheme, Disableable, Sizable,
};
use serde::{Deserialize, Serialize};

use super::Page;
use crate::pages::PageTrait;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeoDataInfo {
    pub name: String,
    pub geo_type: GeoType,
    pub size: u64,
    pub updated_at: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GeoType {
    #[default]
    GeoIp,
    GeoSite,
    Mmdb,
    Asn,
}

impl GeoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GeoType::GeoIp => "geoip",
            GeoType::GeoSite => "geosite",
            GeoType::Mmdb => "mmdb",
            GeoType::Asn => "asn",
        }
    }
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
    geo_data: Entity<Vec<GeoDataInfo>>,
    proxy_providers: Entity<Vec<ProviderInfo>>,
    rule_providers: Entity<Vec<ProviderInfo>>,
    updating_geo: Entity<Option<GeoType>>,
}

impl ResourcesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            geo_data: cx.new(|_| {
                vec![
                    GeoDataInfo {
                        name: "Country.mmdb".into(),
                        geo_type: GeoType::Mmdb,
                        size: 5_800_000,
                        updated_at: Some("2024-01-15".into()),
                        version: Some("2024011500".into()),
                    },
                    GeoDataInfo {
                        name: "GeoSite.dat".into(),
                        geo_type: GeoType::GeoSite,
                        size: 4_200_000,
                        updated_at: Some("2024-01-14".into()),
                        version: Some("2024011400".into()),
                    },
                ]
            }),
            proxy_providers: cx.new(|_| Vec::new()),
            rule_providers: cx.new(|_| Vec::new()),
            updating_geo: cx.new(|_| None),
        }
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

    fn render_geo_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let geo_data = self.geo_data.read(cx);
        let updating = self.updating_geo.read(cx);

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
                            .child("Update All")
                            .when(updating.is_some(), |this| this.disabled(true)),
                    ),
            )
            .children(geo_data.iter().map(|geo| {
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
                                            .child(geo.geo_type.as_str().to_uppercase()),
                                    ),
                            )
                            .child(
                                Button::new(SharedString::from(format!("update-{}", geo.name)))
                                    .with_size(gpui_component::Size::XSmall)
                                    .child("Update")
                                    .when(
                                        updating
                                            .as_ref()
                                            .map(|t| *t == geo.geo_type)
                                            .unwrap_or(false),
                                        |this| this.disabled(true),
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
                            })
                            .when_some(geo.version.as_ref(), |this, v| {
                                this.child(format!("Version: {}", v))
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
                        Button::new(SharedString::from(format!("update-{}-all", title)))
                            .with_size(gpui_component::Size::XSmall)
                            .child("Update All"),
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
                                            .child(format!("{} items", provider.count)),
                                    ),
                            )
                            .child(
                                Button::new(SharedString::from(format!(
                                    "update-provider-{}",
                                    provider.name
                                )))
                                .with_size(gpui_component::Size::XSmall)
                                .child("Update"),
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
        let theme = cx.theme().clone();
        let proxy_providers = self.proxy_providers.read(cx).clone();
        let rule_providers = self.rule_providers.read(cx).clone();

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
            .child(self.render_provider_section("Rule Providers", &rule_providers, cx))
    }
}
