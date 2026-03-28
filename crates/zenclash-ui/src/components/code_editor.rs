use gpui::{
    div, prelude::FluentBuilder, px, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex, ActiveTheme, IconName, Sizable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeLanguage {
    #[default]
    Yaml,
    JavaScript,
    Json,
    Css,
    Plaintext,
}

impl CodeLanguage {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => CodeLanguage::Yaml,
            "js" | "javascript" => CodeLanguage::JavaScript,
            "json" => CodeLanguage::Json,
            "css" => CodeLanguage::Css,
            _ => CodeLanguage::Plaintext,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CodeLanguage::Yaml => "yaml",
            CodeLanguage::JavaScript => "javascript",
            CodeLanguage::Json => "json",
            CodeLanguage::Css => "css",
            CodeLanguage::Plaintext => "plaintext",
        }
    }
}

pub struct CodeEditor {
    pub content: String,
    pub language: CodeLanguage,
    pub filename: Option<String>,
    pub read_only: bool,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub minimap: bool,
    pub on_change: Option<Box<dyn Fn(&str) + 'static>>,
    pub on_save: Option<Box<dyn Fn() + 'static>>,
}

impl CodeEditor {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            language: CodeLanguage::default(),
            filename: None,
            read_only: false,
            line_numbers: true,
            word_wrap: false,
            minimap: false,
            on_change: None,
            on_save: None,
        }
    }

    pub fn language(mut self, language: CodeLanguage) -> Self {
        self.language = language;
        self
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    pub fn word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    pub fn minimap(mut self, show: bool) -> Self {
        self.minimap = show;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn on_save(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_save = Some(Box::new(handler));
        self
    }

    fn render_line_numbers(&self, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let lines = self.content.lines().count();
        let width = (lines.to_string().len() * 8 + 16) as f32;

        div()
            .w(px(width))
            .h_full()
            .pr_2()
            .text_right()
            .text_xs()
            .text_color(theme.muted_foreground)
            .children((1..=lines).map(|n| div().child(n.to_string())))
    }

    fn highlight_line(&self, line: &str, theme: &gpui_component::Theme) -> impl IntoElement {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        let mut colored = div().w_full().pl(px(indent as f32 * 8.0));

        match self.language {
            CodeLanguage::Yaml => {
                if trimmed.starts_with('#') {
                    colored = colored.text_color(theme.muted_foreground);
                } else if trimmed.contains(':') {
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        colored = colored.child(
                            h_flex()
                                .gap_1()
                                .child(div().text_color(theme.primary).child(parts[0].to_string()))
                                .child(div().child(":"))
                                .child(
                                    div()
                                        .text_color(theme.muted_foreground)
                                        .child(parts[1].to_string()),
                                ),
                        );
                    } else {
                        colored = colored.child(trimmed.to_string());
                    }
                } else if trimmed.starts_with('-') {
                    colored = colored.text_color(theme.success);
                } else {
                    colored = colored.child(trimmed.to_string());
                }
            }
            CodeLanguage::JavaScript => {
                if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    colored = colored.text_color(theme.muted_foreground);
                } else if [
                    "function", "const", "let", "var", "if", "else", "return", "async", "await",
                    "import", "export", "from", "class", "new",
                ]
                .iter()
                .any(|k| trimmed.starts_with(k))
                {
                    colored = colored.text_color(theme.primary);
                } else {
                    colored = colored.child(trimmed.to_string());
                }
            }
            CodeLanguage::Json => {
                if trimmed.starts_with('"') && trimmed.contains(':') {
                    colored = colored.text_color(theme.primary);
                } else if trimmed.starts_with('"') {
                    colored = colored.text_color(theme.success);
                } else {
                    colored = colored.child(trimmed.to_string());
                }
            }
            CodeLanguage::Css => {
                if trimmed.starts_with("/*") || trimmed.starts_with("*") {
                    colored = colored.text_color(theme.muted_foreground);
                } else if trimmed.ends_with('{') || trimmed.ends_with('}') {
                    colored = colored.text_color(theme.primary);
                } else if trimmed.contains(':') {
                    colored = colored.text_color(theme.warning);
                } else {
                    colored = colored.child(trimmed.to_string());
                }
            }
            CodeLanguage::Plaintext => {
                colored = colored.child(trimmed.to_string());
            }
        }

        colored
    }
}

impl RenderOnce for CodeEditor {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();

        v_flex()
            .w_full()
            .h_full()
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius)
            .overflow_hidden()
            .child(
                h_flex()
                    .w_full()
                    .h(px(32.))
                    .px_3()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.muted)
                    .when_some(self.filename.clone(), |this, name| {
                        this.child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(name),
                        )
                    })
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("wrap")
                                    .with_size(gpui_component::Size::XSmall)
                                    .icon(IconName::Minus)
                                    .when(self.word_wrap, |this| this.primary()),
                            )
                            .child(
                                Button::new("copy")
                                    .with_size(gpui_component::Size::XSmall)
                                    .icon(IconName::Copy),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .when(self.line_numbers, |this| {
                        this.child(self.render_line_numbers(cx))
                    })
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .p_2()
                            .overflow_scrollbar()
                            .font_family("monospace")
                            .text_sm()
                            .when(self.word_wrap, |this| this.whitespace_normal())
                            .when(!self.word_wrap, |this| this.whitespace_nowrap())
                            .children(lines.iter().map(|line| {
                                div().w_full().min_h(px(20.)).child(
                                    if self.language == CodeLanguage::Plaintext {
                                        div().child(line.clone()).into_any_element()
                                    } else {
                                        self.highlight_line(line, &theme).into_any_element()
                                    },
                                )
                            })),
                    ),
            )
    }
}
