use ratatui::layout::Constraint;
use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::widgets::Row;

use crate::state::{ServerSort, SortDirection};
use crate::ui::styles::Theme;

#[derive(Clone, Copy)]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

pub struct Column {
    pub name: &'static str,
    pub width: u16,
    pub align: ColumnAlign,
}

impl Column {
    pub fn left(name: &'static str, width: u16) -> Self {
        Self {
            name,
            width,
            align: ColumnAlign::Left,
        }
    }

    pub fn dynamic(name: &'static str) -> Self {
        Self {
            name,
            width: 0,
            align: ColumnAlign::Left,
        }
    }
}

pub struct PaneTable {
    pub title: String,
    pub columns: Vec<Column>,
    pub focus_prefix: &'static str,
}

impl PaneTable {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            columns: Vec::new(),
            focus_prefix: "> ",
        }
    }

    pub fn with_columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    pub fn header(&self) -> String {
        self.header_with_widths(&[])
    }

    pub fn header_with_widths(&self, dynamic_widths: &[usize]) -> String {
        self.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let content = col.name;
                let width = if col.width == 0 {
                    *dynamic_widths.get(i).unwrap_or(&content.len())
                } else {
                    col.width as usize
                };
                match col.align {
                    ColumnAlign::Left => format!("{:<width$}", content, width = width),
                    ColumnAlign::Center => {
                        let len = content.len();
                        if len >= width {
                            content.to_string()
                        } else {
                            let pad = (width - len) / 2;
                            format!("{}{:width$}", " ".repeat(pad), content, width = width)
                        }
                    }
                    ColumnAlign::Right => {
                        format!("{:>width$}", content, width = width)
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn title_with_indicator(&self, focused: bool) -> String {
        if focused {
            format!("{}{}", self.focus_prefix, self.title)
        } else {
            format!("  {}", self.title)
        }
    }

    pub fn format_row_with_widths(&self, values: &[&str], dynamic_widths: &[usize]) -> String {
        assert!(
            values.len() <= self.columns.len(),
            "Too many values for columns"
        );
        values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let col = &self.columns[i];
                let width = if col.width == 0 {
                    *dynamic_widths.get(i).unwrap_or(&value.len())
                } else {
                    col.width as usize
                };
                match col.align {
                    ColumnAlign::Left => format!("{:<width$}", value, width = width),
                    ColumnAlign::Right => format!("{:>width$}", value, width = width),
                    ColumnAlign::Center => {
                        let len = value.len();
                        if len >= width {
                            value.to_string()
                        } else {
                            let pad = (width - len) / 2;
                            format!("{}{:width$}", " ".repeat(pad), value, width = width)
                        }
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn format_row(&self, values: &[&str]) -> String {
        let dynamic_widths: Vec<usize> = values.iter().map(|v| v.len()).collect();
        self.format_row_with_widths(values, &dynamic_widths)
    }

    /// Returns Constraints for ratatui Table column widths
    pub fn column_widths(&self, dynamic_widths: &[usize]) -> Vec<Constraint> {
        self.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if col.width == 0 {
                    let w = dynamic_widths.get(i).copied().unwrap_or(10);
                    if i == self.columns.len() - 1 {
                        Constraint::Fill(1)
                    } else {
                        Constraint::Length(w as u16)
                    }
                } else {
                    Constraint::Length(col.width)
                }
            })
            .collect()
    }

    /// Returns a Row for ratatui Table header
    pub fn header_row(&self, dynamic_widths: &[usize], theme: Option<&Theme>) -> Row<'static> {
        self.header_row_with_sort(dynamic_widths, None, SortDirection::Asc, theme)
    }

    /// Returns a Row for ratatui Table header with sort indicator
    /// If theme is provided, key numbers (1, 2) will be colored with key_hint
    pub fn header_row_with_sort(
        &self,
        dynamic_widths: &[usize],
        sort_by: Option<ServerSort>,
        sort_direction: SortDirection,
        theme: Option<&Theme>,
    ) -> Row<'static> {
        let sort_indicator = sort_direction.label();

        let cells: Vec<ratatui::widgets::Cell> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let content = col.name;
                let width = if col.width == 0 {
                    *dynamic_widths.get(i).unwrap_or(&content.len())
                } else {
                    col.width as usize
                };

                // Add key number prefix only for sortable columns that are currently sorted
                let key_num: Option<&str> = match (i, sort_by) {
                    (0, Some(ServerSort::Country)) => Some("¹"),
                    (1, Some(ServerSort::Code)) => Some("²"),
                    _ => None,
                };

                // Add sort indicator for the sorted column (highlight arrow only)
                let (content_text, indicator_text) = match (i, sort_by) {
                    (0, Some(ServerSort::Code)) => {
                        (content.to_string(), format!(" {}", sort_indicator))
                    }
                    (1, Some(ServerSort::Country)) => {
                        (content.to_string(), format!(" {}", sort_indicator))
                    }
                    _ => (content.to_string(), String::new()),
                };

                let header_primary = theme.map(|t| t.primary).unwrap_or(Theme::default().primary);
                let header_accent = theme.map(|t| t.warning).unwrap_or(Theme::default().warning);

                // Calculate total content length (key + text + indicator)
                let key_len = key_num.map(|k| k.len()).unwrap_or(0);
                let total_len = key_len + content_text.len() + indicator_text.len();
                let pad_len = width.saturating_sub(total_len);

                let key_style = theme
                    .map(|t| Style::default().fg(t.warning))
                    .unwrap_or_else(|| Style::default().fg(Theme::default().warning));

                // Build spans: [key?] + [text] + [arrow?] + [padding]
                let mut spans = Vec::new();

                if let Some(key) = key_num {
                    spans.push(Span::styled(key, key_style));
                }
                // Content with primary
                spans.push(Span::styled(
                    content_text,
                    Style::default()
                        .fg(header_primary)
                        .add_modifier(Modifier::BOLD),
                ));
                // Arrow with accent (only if sorted column)
                if !indicator_text.is_empty() {
                    spans.push(Span::styled(
                        indicator_text,
                        Style::default()
                            .fg(header_accent)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                // Add padding at the end for Left align
                if pad_len > 0 {
                    spans.push(Span::raw(" ".repeat(pad_len)));
                }

                ratatui::widgets::Cell::from(Line::from(spans))
            })
            .collect();
        Row::new(cells).height(1)
    }
}

pub struct CountriesTable;

impl CountriesTable {
    pub fn table() -> PaneTable {
        PaneTable::new("Countries").with_columns(vec![
            Column::left("Code", 0),
            Column::left("Country", 0),
            Column::left("Cities", 0),
        ])
    }
}

pub struct CitiesTable;

impl CitiesTable {
    pub fn table() -> PaneTable {
        PaneTable::new("Cities")
            .with_columns(vec![Column::left("City", 19), Column::left("Features", 0)])
    }
}
