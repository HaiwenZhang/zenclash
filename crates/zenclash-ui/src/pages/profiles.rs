use gpui::{
    div, prelude::FluentBuilder, App, AppContext, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex, ActiveTheme, Icon, IconName,
};

use zenclash_core::config::ProfileExtra;
use zenclash_core::prelude::{HttpClient, ProfileItem, ProfileType};

pub struct ProfilesPage {
    profiles: Vec<ProfileItem>,
    selected_profile: Option<String>,
    new_profile_url: Entity<InputState>,
    new_profile_name: Entity<InputState>,
    is_updating: bool,
    update_status: Option<String>,
    focus_handle: FocusHandle,
}

impl ProfilesPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            profiles: Vec::new(),
            selected_profile: None,
            new_profile_url: cx.new(|cx| InputState::new(window, cx)),
            new_profile_name: cx.new(|cx| InputState::new(window, cx)),
            is_updating: false,
            update_status: None,
            focus_handle: cx.focus_handle(),
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

        let profile = ProfileItem {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            url: Some(url),
            profile_type: ProfileType::Remote,
            path: None,
            interval: None,
            last_update: None,
            sub_info: None,
            auto_update: false,
            updated: None,
            extra: ProfileExtra::default(),
        };

        self.profiles.push(profile);

        self.new_profile_url.update(cx, |state, _| {});
        self.new_profile_name.update(cx, |state, _| {});

        self.update_status = Some("Profile added".to_string());
        cx.notify();
    }

    pub fn remove_profile(&mut self, id: String, cx: &mut Context<Self>) {
        self.profiles.retain(|p| p.id != id);
        cx.notify();
    }

    pub fn select_profile(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_profile = Some(id);
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
                    .bg(cx.theme().muted.opacity(0.1))
                    .child("Add New Subscription")
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Input::new(&name_input))
                            .child(Input::new(&url_input))
                            .child(Button::new("add-profile").primary().label("Add").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.add_profile(cx);
                                }),
                            )),
                    ),
            )
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
                        let is_selected = self.selected_profile.as_ref() == Some(&profile.id);
                        let id = profile.id.clone();

                        h_flex()
                            .id(SharedString::from(format!("profile-{}", i)))
                            .gap_2()
                            .p_3()
                            .cursor_pointer()
                            .when(is_selected, |this| this.bg(cx.theme().primary.opacity(0.1)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select_profile(id.clone(), cx);
                            }))
                            .child(div().flex_1().child(profile.name.clone()))
                            .when(is_selected, |this| this.child(div().child("Active")))
                    })),
            )
    }
}
