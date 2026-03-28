use gpui::{
    div, prelude::FluentBuilder, px, App, IntoElement, ParentElement,
    RenderOnce, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    v_flex, ActiveTheme, Icon, IconName,
};

pub struct ConfirmModal {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub confirm_variant: ConfirmVariant,
    pub on_confirm: Option<Box<dyn Fn() + 'static>>,
    pub on_cancel: Option<Box<dyn Fn() + 'static>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfirmVariant {
    #[default]
    Primary,
    Danger,
    Warning,
}

impl ConfirmModal {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "Confirm".into(),
            cancel_text: "Cancel".into(),
            confirm_variant: ConfirmVariant::default(),
            on_confirm: None,
            on_cancel: None,
        }
    }

    pub fn confirm_text(mut self, text: impl Into<String>) -> Self {
        self.confirm_text = text.into();
        self
    }

    pub fn cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    pub fn danger(mut self) -> Self {
        self.confirm_variant = ConfirmVariant::Danger;
        self
    }

    pub fn warning(mut self) -> Self {
        self.confirm_variant = ConfirmVariant::Warning;
        self
    }

    pub fn on_confirm(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_confirm = Some(Box::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ConfirmModal {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let on_confirm = self.on_confirm.take();
        let on_cancel = self.on_cancel.take();

        div()
            .relative()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(400.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(self.title),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.message),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("cancel").child(self.cancel_text).on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_cancel {
                                        handler();
                                    }
                                },
                            ))
                            .child(
                                Button::new("confirm")
                                    .child(self.confirm_text)
                                    .when(self.confirm_variant == ConfirmVariant::Primary, |this| {
                                        this.primary()
                                    })
                                    .when(self.confirm_variant == ConfirmVariant::Danger, |this| {
                                        this.danger()
                                    })
                                    .when(self.confirm_variant == ConfirmVariant::Warning, |this| {
                                        this.warning()
                                    })
                                    .on_click(move |_, _, _| {
                                        if let Some(handler) = &on_confirm {
                                            handler();
                                        }
                                    }),
                            ),
                    ),
            )
    }
}

pub struct EditModal {
    pub title: String,
    pub fields: Vec<EditField>,
    pub save_text: String,
    pub cancel_text: String,
    pub on_save: Option<Box<dyn Fn() + 'static>>,
    pub on_cancel: Option<Box<dyn Fn() + 'static>>,
}

pub struct EditField {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub field_type: EditFieldType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditFieldType {
    #[default]
    Text,
    Number,
    Password,
    Url,
}

impl EditModal {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fields: Vec::new(),
            save_text: "Save".into(),
            cancel_text: "Cancel".into(),
            on_save: None,
            on_cancel: None,
        }
    }

    pub fn field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(EditField {
            label: label.into(),
            value: value.into(),
            placeholder: String::new(),
            field_type: EditFieldType::default(),
        });
        self
    }

    pub fn password(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(EditField {
            label: label.into(),
            value: value.into(),
            placeholder: String::new(),
            field_type: EditFieldType::Password,
        });
        self
    }

    pub fn on_save(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_save = Some(Box::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for EditModal {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let on_save = self.on_save.take();
        let on_cancel = self.on_cancel.take();

        div()
            .relative()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(480.))
                    .max_h(px(600.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .overflow_y_hidden()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(self.title),
                    )
                    .child(v_flex().gap_3().children(self.fields.iter().map(|field| {
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(field.label.clone()),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .bg(theme.muted)
                                    .text_sm()
                                    .child(if field.field_type == EditFieldType::Password {
                                        "••••••••".into()
                                    } else {
                                        field.value.clone()
                                    }),
                            )
                    })))
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("cancel").child(self.cancel_text).on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_cancel {
                                        handler();
                                    }
                                },
                            ))
                            .child(
                                Button::new("save")
                                    .primary()
                                    .child(self.save_text)
                                    .on_click(move |_, _, _| {
                                        if let Some(handler) = &on_save {
                                            handler();
                                        }
                                    }),
                            ),
                    ),
            )
    }
}

pub struct InfoModal {
    pub title: String,
    pub content: String,
    pub icon: Option<IconName>,
    pub on_close: Option<Box<dyn Fn() + 'static>>,
}

impl InfoModal {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            icon: None,
            on_close: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for InfoModal {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_close = self.on_close.take();

        div()
            .relative()
            .inset_0()
            .bg(theme.background.opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(400.))
                    .p_4()
                    .gap_4()
                    .shadow_xl()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .when_some(self.icon, |this, icon| this.child(Icon::new(icon).size_5()))
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(self.title),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.content),
                    )
                    .child(
                        h_flex().justify_end().child(
                            Button::new("close").primary().child("Close").on_click(
                                move |_, _, _| {
                                    if let Some(handler) = &on_close {
                                        handler();
                                    }
                                },
                            ),
                        ),
                    ),
            )
    }
}
