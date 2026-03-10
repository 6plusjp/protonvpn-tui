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
        self.columns
            .iter()
            .map(|col| {
                let content = col.name;
                match col.align {
                    ColumnAlign::Left => format!("{:<width$}", content, width = col.width as usize),
                    ColumnAlign::Center => {
                        let len = content.len();
                        if len >= col.width as usize {
                            content.to_string()
                        } else {
                            let pad = (col.width as usize - len) / 2;
                            format!(
                                "{}{:width$}",
                                " ".repeat(pad),
                                content,
                                width = col.width as usize
                            )
                        }
                    }
                    ColumnAlign::Right => {
                        format!("{:>width$}", content, width = col.width as usize)
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
}

pub struct CountriesTable;

impl CountriesTable {
    pub fn table() -> PaneTable {
        PaneTable::new("Countries").with_columns(vec![
            Column::left("ID", 4),
            Column::left("Country", 0),
            Column::left("Cities", 0),
        ])
    }
}

pub struct CitiesTable;

impl CitiesTable {
    pub fn table() -> PaneTable {
        PaneTable::new("Cities")
            .with_columns(vec![Column::left("City", 15), Column::left("Server", 0)])
    }
}
