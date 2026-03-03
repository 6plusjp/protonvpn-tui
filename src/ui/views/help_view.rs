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
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw("  - Connect to selected server"),
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
            Span::raw("  - Toggle direction (↑/↓)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::raw("  - Cycle field (ID/Country)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw("  - Navigate server list"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw("  - Switch view"),
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
