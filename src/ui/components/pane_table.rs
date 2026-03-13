use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::Row;

use crate::state::{ServerSort, SortDirection};

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
    pub fn header_row(&self, dynamic_widths: &[usize]) -> Row<'static> {
        self.header_row_with_sort(dynamic_widths, None, SortDirection::Asc)
    }

    /// Returns a Row for ratatui Table header with sort indicator
    pub fn header_row_with_sort(
        &self,
        dynamic_widths: &[usize],
        sort_by: Option<ServerSort>,
        sort_direction: SortDirection,
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

                // Add sort indicator for the sorted column
                let text_with_indicator = match (i, sort_by) {
                    (0, Some(ServerSort::Code)) => {
                        format!("{} {}", content, sort_indicator)
                    }
                    (1, Some(ServerSort::Country)) => {
                        format!("{} {}", content, sort_indicator)
                    }
                    _ => content.to_string(),
                };

                let text = match col.align {
                    ColumnAlign::Left => {
                        format!("{:<width$}", text_with_indicator, width = width)
                    }
                    ColumnAlign::Center => {
                        let len = text_with_indicator.len();
                        if len >= width {
                            text_with_indicator
                        } else {
                            let pad = (width - len) / 2;
                            format!(
                                "{}{:width$}",
                                " ".repeat(pad),
                                text_with_indicator,
                                width = width
                            )
                        }
                    }
                    ColumnAlign::Right => {
                        format!("{:>width$}", text_with_indicator, width = width)
                    }
                };
                ratatui::widgets::Cell::from(text).style(Style::new().bold())
            })
            .collect();
        Row::new(cells).height(1)
    }
}

pub struct CountriesTable;

impl CountriesTable {
    pub fn table() -> PaneTable {
        PaneTable::new("Countries").with_columns(vec![
            Column::left("Code", 6),
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
