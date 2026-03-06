use crate::ui::components::{centered_block, key_span, primary_style};
use crate::ui::styles::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_help_view(f: &mut Frame<'_>, area: Rect) {
    let theme = Theme::default();
    let block = centered_block("Help", &theme);

    let help_text = vec![
        Line::from(vec![
            key_span("Key", &theme),
            Span::raw(": "),
            Span::styled("Action", primary_style(&theme)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            key_span("c/Enter", &theme),
            Span::raw(" - Connect to selected server"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("x", &theme),
            Span::raw("  - Random connect"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("d", &theme),
            Span::raw("  - Disconnect from VPN"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("r", &theme),
            Span::raw("  - Refresh server list"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("s", &theme),
            Span::raw("  - Toggle direction (asc/desc)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("f", &theme),
            Span::raw("  - Cycle field (ID/Country)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("j/k/↑/↓", &theme),
            Span::raw(" - Navigate server list"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("g", &theme),
            Span::raw("  - Go to top (press twice: gg)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("G", &theme),
            Span::raw("  - Go to bottom"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("Ctrl+d", &theme),
            Span::raw(" - Page down"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("Ctrl+u", &theme),
            Span::raw(" - Page up"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("Tab", &theme),
            Span::raw("  - Switch view"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("?", &theme),
            Span::raw("  - Show this help"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            key_span("q", &theme),
            Span::raw("  - Quit"),
        ]),
    ];

    f.render_widget(block, area);
    f.render_widget(Paragraph::new(help_text), area);
}
