use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    div, prelude::FluentBuilder, px, App, AppContext, Context, Entity, IntoElement, Model,
    ParentElement, Render, Styled, Window,
};
use gpui_component::{button::Button, h_flex, v_flex, ActiveTheme, Icon, IconName, Sizable};
use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl ToastMessage {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            id: rand::random(),
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Success)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Error)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Warning)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Info)
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
}

pub struct ToastManager {
    toasts: Vec<ToastMessage>,
    next_id: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
        }
    }

    pub fn push(&mut self, mut toast: ToastMessage) -> usize {
        let id = self.next_id;
        toast.id = id;
        self.next_id += 1;
        self.toasts.push(toast);
        id
    }

    pub fn remove(&mut self, id: usize) {
        self.toasts.retain(|t| t.id != id);
    }

    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    pub fn expire(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn toasts(&self) -> &[ToastMessage] {
        &self.toasts
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToastContainer {
    manager: Model<ToastManager>,
}

impl ToastContainer {
    pub fn new(cx: &mut App) -> Self {
        Self {
            manager: cx.new(|_| ToastManager::new()),
        }
    }

    pub fn show(&self, toast: ToastMessage, cx: &mut App) {
        self.manager.update(cx, |m, _| {
            m.push(toast);
        });
    }

    pub fn success(&self, message: impl Into<String>, cx: &mut App) {
        self.show(ToastMessage::success(message), cx);
    }

    pub fn error(&self, message: impl Into<String>, cx: &mut App) {
        self.show(ToastMessage::error(message), cx);
    }

    pub fn warning(&self, message: impl Into<String>, cx: &mut App) {
        self.show(ToastMessage::warning(message), cx);
    }

    pub fn info(&self, message: impl Into<String>, cx: &mut App) {
        self.show(ToastMessage::info(message), cx);
    }

    pub fn dismiss(&self, id: usize, cx: &mut App) {
        self.manager.update(cx, |m, _| {
            m.remove(id);
        });
    }

    pub fn manager(&self) -> Model<ToastManager> {
        self.manager.clone()
    }
}

impl Render for ToastContainer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let toasts = self.manager.read(cx).toasts().to_vec();

        v_flex()
            .fixed()
            .top_0()
            .right_0()
            .w(px(320.))
            .gap_2()
            .p_4()
            .z_index(1000)
            .children(toasts.into_iter().map(|toast| {
                let id = toast.id;
                let (icon, bg_color, fg_color) = match toast.toast_type {
                    ToastType::Success => (IconName::Check, theme.success, theme.background),
                    ToastType::Error => (IconName::X, theme.destructive, theme.background),
                    ToastType::Warning => {
                        (IconName::AlertTriangle, theme.warning, theme.background)
                    },
                    ToastType::Info => (IconName::Info, theme.primary, theme.background),
                };

                let manager = self.manager.clone();
                h_flex()
                    .gap_2()
                    .p_3()
                    .rounded(theme.radius)
                    .bg(bg_color)
                    .text_color(fg_color)
                    .shadow_lg()
                    .child(Icon::new(icon).size_4())
                    .child(div().flex_1().text_sm().child(toast.message))
                    .child(
                        Button::new(format!("dismiss-{}", id))
                            .xsmall()
                            .icon(Icon::new(IconName::X))
                            .text_color(fg_color)
                            .on_click(move |_, cx, _| {
                                manager.update(cx, |m, _| {
                                    m.remove(id);
                                });
                            }),
                    )
            }))
    }
}

pub fn toast(cx: &mut App) -> ToastContainer {
    ToastContainer::new(cx)
}
