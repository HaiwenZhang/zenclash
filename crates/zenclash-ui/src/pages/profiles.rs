use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, App, AppContext, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex, ActiveTheme, Disableable, Icon, IconName, Sizable,
};
use parking_lot::RwLock;

use zenclash_core::config::ProfileExtra;
use zenclash_core::prelude::{CoreManager, HttpClient, ProfileConfig, ProfileItem, ProfileType};

pub struct ProfilesPage {
    core_manager: Arc<RwLock<CoreManager>>,
    profiles: Vec<ProfileItem>,
    current_profile: Option<String>,
    new_profile_url: Entity<InputState>,
    new_profile_name: Entity<InputState>,
    update_status: Option<String>,
    focus_handle: FocusHandle,
}

impl ProfilesPage {
    pub fn new(core_manager: Arc<RwLock<CoreManager>>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = ProfileConfig::load().unwrap_or_default();
        let profiles = config.items.clone();
        let current_profile = config.current.clone();

        Self {
            core_manager,
            profiles,
            current_profile,
            new_profile_url: cx.new(|cx| InputState::new(window, cx)),
            new_profile_name: cx.new(|cx| InputState::new(window, cx)),
            update_status: None,
            focus_handle: cx.focus_handle(),
        }
    }

    fn save_profiles(&self) {
        let mut config = ProfileConfig::default();
        config.items = self.profiles.clone();
        config.current = self.current_profile.clone();
        if let Err(e) = config.save() {
            eprintln!("Failed to save profiles: {}", e);
        }
    }

    pub fn add_profile(&mut self, cx: &mut Context<Self>) {
        let url = self.new_profile_url.read(cx).text().to_string();
        let name = self.new_profile_name.read(cx).text().to_string();

        if url.is_empty() || name.is_empty() {
            self.update_status = Some("Please enter both URL and name".to_string());
            cx.notify();
            return;
        }

        let profile = ProfileItem::new_remote(name.clone(), url.clone(), Some(24));

        self.profiles.push(profile);
        self.save_profiles();

        self.new_profile_url.update(cx, |state, _| {});
        self.new_profile_name.update(cx, |state, _| {});

        self.update_status = Some(format!("Profile '{}' added", name));
        cx.notify();
    }

    pub fn remove_profile(&mut self, id: String, cx: &mut Context<Self>) {
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            let name = self.profiles[pos].name.clone();
            self.profiles.remove(pos);
            if self.current_profile.as_ref() == Some(&id) {
                self.current_profile = None;
            }
            self.save_profiles();
            self.update_status = Some(format!("Profile '{}' removed", name));
        }
        cx.notify();
    }

    pub fn select_profile(&mut self, id: String, cx: &mut Context<Self>) {
        self.current_profile = Some(id.clone());
        self.save_profiles();
        
        let core_manager = self.core_manager.clone();
        cx.spawn(async move |this, cx| {
            let result = {
                let manager = core_manager.read();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        manager.load_profile(&id).await
                    })
                })
            };
            
            if let Err(e) = result {
                eprintln!("Failed to load profile: {}", e);
                let _ = this.update(cx, |this, cx| {
                    this.update_status = Some(format!("Failed to load profile: {}", e));
                    cx.notify();
                });
            } else {
                let _ = this.update(cx, |this, cx| {
                    this.update_status = Some("Profile loaded successfully".to_string());
                    cx.notify();
                });
            }
        })
        .detach();
        
        cx.notify();
    }

    pub fn update_profile(&mut self, id: String, cx: &mut Context<Self>) {
        if let Some(profile) = self.profiles.iter().find(|p| p.id == id) {
            let name = profile.name.clone();
            self.update_status = Some(format!("Updating profile '{}'...", name));
            
            let url = profile.url.clone();
            
            cx.spawn(async move |this, cx| {
                if let Some(url) = url {
                    let client = match HttpClient::new_default() {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = this.update(cx, |this, cx| {
                                this.update_status = Some(format!("Failed to create HTTP client: {}", e));
                                cx.notify();
                            });
                            return;
                        }
                    };
                    
                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            client.get_text(&url).await
                        })
                    });
                    
                    match result {
                        Ok(_) => {
                            let _ = this.update(cx, |this, cx| {
                                this.update_status = Some(format!("Profile '{}' updated", name));
                                cx.notify();
                            });
                        }
                        Err(e) => {
                            let _ = this.update(cx, |this, cx| {
                                this.update_status = Some(format!("Failed to update: {}", e));
                                cx.notify();
                            });
                        }
                    }
                }
            })
            .detach();
        }
        cx.notify();
    }
}

impl Focusable for ProfilesPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ProfilesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let url_input = self.new_profile_url.clone();
        let name_input = self.new_profile_name.clone();
        let theme = cx.theme();

        v_flex()
            .size_full()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Profiles"),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_4()
                    .rounded_md()
                    .bg(theme.muted.opacity(0.1))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Add New Subscription")
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Input::new(&name_input))
                            .child(Input::new(&url_input))
                            .child(Button::new("add-profile").primary().child("Add").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.add_profile(cx);
                                }),
                            )),
                    ),
            )
            .when_some(self.update_status.as_ref(), |this, status| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(status.clone())
                )
            })
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(format!("Subscriptions ({})", self.profiles.len())),
            )
            .child(
                v_flex()
                    .gap_1()
                    .children(self.profiles.iter().enumerate().map(|(i, profile)| {
                        let is_selected = self.current_profile.as_ref() == Some(&profile.id);
                        let id = profile.id.clone();
                        let id_for_remove = profile.id.clone();
                        let id_for_update = profile.id.clone();
                        let profile_type = profile.profile_type.clone();

                        v_flex()
                            .gap_1()
                            .p_3()
                            .rounded(theme.radius)
                            .bg(theme.background)
                            .border_1()
                            .border_color(if is_selected { theme.primary } else { theme.border })
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
                                                    .child(profile.name.clone())
                                            )
                                            .child(
                                                div()
                                                    .px_1()
                                                    .rounded(theme.radius)
                                                    .bg(theme.muted)
                                                    .text_xs()
                                                    .child(profile_type.to_string())
                                            )
                                            .when(is_selected, |this| {
                                                this.child(
                                                    div()
                                                        .px_1()
                                                        .rounded(theme.radius)
                                                        .bg(theme.primary)
                                                        .text_color(theme.primary_foreground)
                                                        .text_xs()
                                                        .child("Active")
                                                )
                                            })
                                    )
                                    .child(
                                        h_flex()
                                            .gap_1()
                                            .child(
                                                Button::new(SharedString::from(format!("select-{}", i)))
                                                    .with_size(gpui_component::Size::XSmall)
                                                    .child("Select")
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        this.select_profile(id.clone(), cx);
                                                    }))
                                            )
                                            .when(profile.profile_type == ProfileType::Remote, |this| {
                                                this.child(
                                                    Button::new(SharedString::from(format!("update-{}", i)))
                                                        .with_size(gpui_component::Size::XSmall)
                                                        .child("Update")
                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                            this.update_profile(id_for_update.clone(), cx);
                                                        }))
                                                )
                                            })
                                            .child(
                                                Button::new(SharedString::from(format!("remove-{}", i)))
                                                    .with_size(gpui_component::Size::XSmall)
                                                    .child("Remove")
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        this.remove_profile(id_for_remove.clone(), cx);
                                                    }))
                                            ),
                                    )
                            )
                            .when_some(profile.url.as_ref(), |this, url| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(url.clone())
                                )
                            })
                    })),
            )
    }
}