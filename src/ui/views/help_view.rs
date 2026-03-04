use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_help_view(f: &mut Frame<'_>, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let help_text = vec![
        Line::from(vec![
            Span::styled("Key", Style::default().fg(Color::Yellow)),
            Span::raw(": "),
            Span::styled("Action", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("c/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" - Connect to selected server"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("x", Style::default().fg(Color::Yellow)),
            Span::raw("  - Random connect"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw("  - Disconnect from VPN"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::raw("  - Refresh server list"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("s", Style::default().fg(Color::Yellow)),
            Span::raw("  - Toggle direction (asc/desc)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::raw("  - Cycle field (ID/Country)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("j/k/↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" - Navigate server list"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("g", Style::default().fg(Color::Yellow)),
            Span::raw("  - Go to top (press twice: gg)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("G", Style::default().fg(Color::Yellow)),
            Span::raw("  - Go to bottom"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+d", Style::default().fg(Color::Yellow)),
            Span::raw(" - Page down"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+u", Style::default().fg(Color::Yellow)),
            Span::raw(" - Page up"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw("  - Switch view"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw("  - Show this help"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw("  - Quit"),
        ]),
    ];

    f.render_widget(block, area);
    f.render_widget(Paragraph::new(help_text), area);
}
