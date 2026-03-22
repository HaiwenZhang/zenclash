use gpui::{
    div, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window,
};
use gpui_component::{
    button::Button,
    input::{Input, InputState},
    v_flex, h_flex,
    ActiveTheme,
};

use zenclash_core::{
    Rule, RuleType,
};

pub struct RulesPage {
    rules: Vec<Rule>,
    search_query: Entity<InputState>,
    focus_handle: FocusHandle,
}

impl RulesPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_query = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search rules...")
        });
        
        Self {
            rules: Vec::new(),
            search_query,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn update_rules(&mut self, rules: Vec<Rule>, cx: &mut Context<Self>) {
        self.rules = rules;
        cx.notify();
    }

    fn get_rule_type_color(&self, rule_type: &RuleType) -> gpui::Hsla {
        match rule_type {
            RuleType::Domain => gpui::Hsla::green(),
            RuleType::DomainSuffix => gpui::Hsla::blue(),
            RuleType::DomainKeyword => gpui::Hsla::yellow(),
            RuleType::GeoIP => gpui::Hsla::red(),
            RuleType::Geosite => gpui::Hsla::purple(),
            RuleType::IP-CIDR => gpui::Hsla::orange(),
            RuleType::SrcIP-CIDR => gpui::Hsla::pink(),
            RuleType::DstPort => gpui::Hsla::cyan(),
            RuleType::SrcPort => gpui::Hsla::teal(),
            RuleType::ProcessName => gpui::Hsla::lime(),
            RuleType::ProcessPath => gpui::Hsla::indigo(),
            RuleType::Match => gpui::Hsla::gray(),
            _ => gpui::Hsla::gray(),
        }
    }

    fn render_rule(&self, rule: &Rule, cx: &Context<Self>) -> impl IntoElement {
        let type_color = self.get_rule_type_color(&rule.rule_type);
        
        h_flex()
            .gap_2()
            .p_2()
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(type_color.opacity(0.2))
                    .text_color(type_color)
                    .text_sm()
                    .child(format!("{:?}", rule.rule_type))
            )
            .child(
                div()
                    .flex_1()
                    .child(rule.payload.clone())
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(cx.theme().primary.opacity(0.1))
                    .child(rule.proxy.clone())
            )
    }
}

impl Focusable for RulesPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RulesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filtered_rules: Vec<_> = {
            let query = self.search_query.read(cx).text().to_string().to_lowercase();
            if query.is_empty() {
                self.rules.clone()
            } else {
                self.rules
                    .iter()
                    .filter(|r| {
                        r.payload.to_lowercase().contains(&query) ||
                        r.proxy.to_lowercase().contains(&query) ||
                        format!("{:?}", r.rule_type).to_lowercase().contains(&query)
                    })
                    .cloned()
                    .collect()
            }
        };
        
        v_flex()
            .size_full()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Rules")
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} rules", self.rules.len()))
                    )
            )
            .child(
                Input::new(&self.search_query, window, cx)
                    .placeholder("Search rules...")
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .gap_1()
                            .children(filtered_rules.iter().map(|rule| {
                                self.render_rule(rule, cx)
                            }))
                    )
            )
    }
}
