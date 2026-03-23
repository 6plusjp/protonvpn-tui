use crate::state::{AppState, ConnectionState};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_header(state: &AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.theme();

    let status_indicator = match state.connection_manager.connection {
        ConnectionState::Connected { .. } => "●",
        ConnectionState::Connecting | ConnectionState::Disconnecting => "◐",
        ConnectionState::Disconnected => "○",
        ConnectionState::Error(_) => "✕",
    };

    let (status_indicator_color, mut status_spans): (_, Vec<Span<'_>>) =
        match &state.connection_manager.connection {
            ConnectionState::Disconnected => {
                let spans = vec![Span::styled(
                    "Disconnected",
                    Style::default().fg(theme.foreground),
                )];
                (theme.foreground, spans)
            }
            ConnectionState::Connecting => {
                let spans = vec![Span::styled(
                    "Connecting...",
                    Style::default().fg(theme.warning),
                )];
                (theme.warning, spans)
            }
            ConnectionState::Connected {
                server,
                ip,
                city,
                country,
                via,
            } => {
                let mut spans = vec![];

                spans.push(Span::styled(
                    server.clone(),
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ));

                if !ip.is_empty() {
                    spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
                    spans.push(Span::styled("ip:", Style::default().fg(theme.dim)));
                    spans.push(Span::styled(
                        ip.as_str(),
                        Style::default().fg(theme.secondary),
                    ));
                }

                let loc = match (&city, &country, &via) {
                    (Some(c), Some(ct), Some(v)) => format!("{},{} via {}", c, ct, v),
                    (Some(c), Some(ct), None) => format!("{},{}", c, ct),
                    (Some(c), None, Some(v)) => format!("{} via {}", c, v),
                    (Some(c), None, None) => c.clone(),
                    (None, Some(ct), Some(v)) => format!("{} via {}", ct, v),
                    (None, Some(ct), None) => ct.clone(),
                    (None, None, Some(v)) => format!("via {}", v),
                    (None, None, None) => String::new(),
                };
                if !loc.is_empty() {
                    spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
                    spans.push(Span::styled("loc:", Style::default().fg(theme.dim)));
                    spans.push(Span::styled(loc, Style::default().fg(theme.secondary)));
                }

                (theme.success, spans)
            }
            ConnectionState::Disconnecting => {
                let spans = vec![Span::styled(
                    "Disconnecting...",
                    Style::default().fg(theme.warning),
                )];
                (theme.warning, spans)
            }
            ConnectionState::Error(e) => {
                let spans = vec![Span::styled(e.clone(), Style::default().fg(theme.error))];
                (theme.error, spans)
            }
        };

    let protocol = match state.connection_manager.connection {
        ConnectionState::Connected { .. } | ConnectionState::Disconnected => {
            state.vpn_state.get_connection_protocol()
        }
        ConnectionState::Connecting
        | ConnectionState::Disconnecting
        | ConnectionState::Error(_) => None,
    };

    let title = " ProtonVPN TUI ";

    if let Some(ref proto) = protocol {
        status_spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
        status_spans.push(Span::styled("protocol:", Style::default().fg(theme.dim)));
        status_spans.push(Span::styled(proto, Style::default().fg(theme.secondary)));
    }

    if let ConnectionState::Connected { .. } = state.connection_manager.connection {
        if let Some(connected_at) = state.vpn_state.get_connected_at() {
            let elapsed = Utc::now().signed_duration_since(connected_at);
            let mins = elapsed.num_minutes();
            let session_str = if mins < 60 {
                format!("{}m", mins.max(1))
            } else {
                format!("{}h {}m", mins / 60, mins % 60)
            };
            status_spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
            status_spans.push(Span::styled("session:", Style::default().fg(theme.dim)));
            status_spans.push(Span::styled(
                session_str,
                Style::default().fg(theme.secondary),
            ));
        }

        if let Some((bytes_received, bytes_sent)) = state.vpn_state.get_connection_stats() {
            fn format_bytes(bytes: u64) -> String {
                const KB: u64 = 1024;
                const MB: u64 = KB * 1024;
                const GB: u64 = MB * 1024;
                if bytes >= GB {
                    format!("{:.1}GB", bytes as f64 / GB as f64)
                } else if bytes >= MB {
                    format!("{:.0}MB", bytes as f64 / MB as f64)
                } else if bytes >= KB {
                    format!("{:.0}KB", bytes as f64 / KB as f64)
                } else {
                    format!("{}B", bytes)
                }
            }
            status_spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
            status_spans.push(Span::styled("↓", Style::default().fg(theme.dim)));
            status_spans.push(Span::styled(
                format_bytes(bytes_received),
                Style::default().fg(theme.secondary),
            ));
            status_spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
            status_spans.push(Span::styled("↑", Style::default().fg(theme.dim)));
            status_spans.push(Span::styled(
                format_bytes(bytes_sent),
                Style::default().fg(theme.secondary),
            ));
        }
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let title_line = Line::from(vec![Span::styled(
        title,
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    )]);

    let mut status_line_spans = vec![
        Span::styled(
            status_indicator,
            Style::default()
                .fg(status_indicator_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    status_line_spans.extend(status_spans);

    let status_line = Line::from(status_line_spans);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.primary));

    f.render_widget(block, area);
    f.render_widget(Paragraph::new(title_line), chunks[0]);
    f.render_widget(
        Paragraph::new(status_line).alignment(ratatui::layout::Alignment::Center),
        chunks[1],
    );
}
