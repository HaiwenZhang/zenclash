use gpui::{InteractiveElement, ParentElement, div, prelude::FluentBuilder, px, App, IntoElement, RenderOnce, Styled, Window};
use gpui_component::{h_flex, v_flex, ActiveTheme};

use super::ConnectionInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Time,
    Process,
    Destination,
    Upload,
    Download,
    Speed,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

#[derive(Debug, Clone)]
pub struct ColumnConfig {
    pub key: SortColumn,
    pub label: &'static str,
    pub width: f32,
    pub visible: bool,
}

pub struct ConnectionTable {
    pub connections: Vec<ConnectionInfo>,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub columns: Vec<ColumnConfig>,
    pub on_sort: Option<Box<dyn Fn(SortColumn) + 'static>>,
    pub on_close: Option<Box<dyn Fn(&str) + 'static>>,
    pub on_detail: Option<Box<dyn Fn(&ConnectionInfo) + 'static>>,
}

impl ConnectionTable {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            sort_column: SortColumn::default(),
            sort_order: SortOrder::default(),
            columns: vec![
                ColumnConfig {
                    key: SortColumn::Time,
                    label: "Time",
                    width: 80.,
                    visible: true,
                },
                ColumnConfig {
                    key: SortColumn::Process,
                    label: "Process",
                    width: 120.,
                    visible: true,
                },
                ColumnConfig {
                    key: SortColumn::Destination,
                    label: "Destination",
                    width: 200.,
                    visible: true,
                },
                ColumnConfig {
                    key: SortColumn::Chain,
                    label: "Chain",
                    width: 100.,
                    visible: true,
                },
                ColumnConfig {
                    key: SortColumn::Upload,
                    label: "Upload",
                    width: 80.,
                    visible: true,
                },
                ColumnConfig {
                    key: SortColumn::Download,
                    label: "Download",
                    width: 80.,
                    visible: true,
                },
                ColumnConfig {
                    key: SortColumn::Speed,
                    label: "Speed",
                    width: 100.,
                    visible: true,
                },
            ],
            on_sort: None,
            on_close: None,
            on_detail: None,
        }
    }

    pub fn connections(mut self, connections: Vec<ConnectionInfo>) -> Self {
        self.connections = connections;
        self
    }

    pub fn sort(mut self, column: SortColumn, order: SortOrder) -> Self {
        self.sort_column = column;
        self.sort_order = order;
        self
    }

    pub fn on_sort(mut self, handler: impl Fn(SortColumn) + 'static) -> Self {
        self.on_sort = Some(Box::new(handler));
        self
    }

    fn render_header(&self, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .gap_0()
            .w_full()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.muted)
            .children(self.columns.iter().filter(|c| c.visible).map(|col| {
                let is_sorted = self.sort_column == col.key;
                let label = col.label;

                div()
                    .w(px(col.width))
                    .p_2()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(if is_sorted {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    })
                    .child(label)
                    .when(is_sorted, |this| {
                        this.child(if self.sort_order == SortOrder::Asc {
                            " ↑"
                        } else {
                            " ↓"
                        })
                    })
            }))
    }

    fn render_row(&self, conn: &ConnectionInfo, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let upload = zenclash_core::prelude::format_traffic(conn.upload);
        let download = zenclash_core::prelude::format_traffic(conn.download);
        let upload_speed = conn
            .upload_speed
            .map(zenclash_core::prelude::format_speed)
            .unwrap_or_default();
        let download_speed = conn
            .download_speed
            .map(zenclash_core::prelude::format_speed)
            .unwrap_or_default();

        let time = chrono::DateTime::from_timestamp(conn.start, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_default();

        let process = conn
            .metadata
            .process
            .clone()
            .unwrap_or_else(|| conn.metadata.source_ip.clone());
        let destination = conn
            .metadata
            .host
            .clone()
            .or_else(|| conn.metadata.sniff_host.clone())
            .or_else(|| conn.metadata.destination_ip.clone())
            .unwrap_or_default();
        let chain = conn.chains.first().cloned().unwrap_or_default();

        h_flex()
            .gap_0()
            .w_full()
            .border_b_1()
            .border_color(theme.border)
            .hover(|this| this.bg(theme.muted.opacity(0.3)))
            .children(self.columns.iter().filter(|c| c.visible).map(|col| {
                let content = match col.key {
                    SortColumn::Time => time.clone(),
                    SortColumn::Process => process.clone(),
                    SortColumn::Destination => destination.clone(),
                    SortColumn::Chain => chain.clone(),
                    SortColumn::Upload => upload.clone(),
                    SortColumn::Download => download.clone(),
                    SortColumn::Speed => format!("↑{} ↓{}", upload_speed, download_speed),
                };

                div()
                    .w(px(col.width))
                    .p_2()
                    .text_xs()
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(content)
            }))
    }
}

impl Default for ConnectionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ConnectionTable {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .w_full()
            .h_full()
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius)
            .overflow_hidden()
            .child(self.render_header(cx))
            .child(
                v_flex().flex_1().overflow_y_hidden().children(
                    self.connections
                        .iter()
                        .map(|conn| self.render_row(conn, cx)),
                ),
            )
    }
}
