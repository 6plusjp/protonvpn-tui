use crate::ui::styles::Theme;
use ratatui::{
    style::{Modifier, Style},
    text::Span,
};

pub fn key_hint_style(theme: &Theme) -> Style {
    Style::default().fg(theme.warning)
}

pub fn primary_style(theme: &Theme) -> Style {
    Style::default().fg(theme.primary)
}

pub fn success_style(theme: &Theme) -> Style {
    Style::default().fg(theme.success)
}

pub fn error_style(theme: &Theme) -> Style {
    Style::default().fg(theme.error)
}

pub fn warning_style(theme: &Theme) -> Style {
    Style::default().fg(theme.warning)
}

pub fn info_style(theme: &Theme) -> Style {
    Style::default().fg(theme.primary)
}

pub fn selected_text(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.selection)
        .add_modifier(Modifier::BOLD)
}

pub fn connected_text(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.success)
        .add_modifier(Modifier::BOLD)
}

pub fn header_style(theme: &Theme) -> Style {
    Style::default().fg(theme.primary)
}

pub fn footer_style(theme: &Theme) -> Style {
    Style::default().fg(theme.warning)
}

pub fn muted_style(theme: &Theme) -> Style {
    Style::default().fg(theme.muted)
}

pub fn key_span<'a>(key: &'a str, theme: &'a Theme) -> Span<'a> {
    Span::styled(key, key_hint_style(theme))
}
