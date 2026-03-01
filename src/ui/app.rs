//! Main TUI application using ratatui

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::panic;
use crate::state::{AppState, AppView, ConnectionState};

pub struct TuiApp {
    state: AppState,
    notification_timer: u8,
}

impl TuiApp {
    pub fn new() -> io::Result<Self> {
        panic::set_hook(Box::new(|_| {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = disable_raw_mode();
        }));

        let mut state = AppState::new();
        // Sync connection state from system (check proton0)
        state.sync_connection_state();
        state.refresh_servers();
        // Show startup notification with connection status
        let conn_status = if state.connection.is_connected() {
            if let ConnectionState::Connected { ref server, ref ip } = state.connection {
                format!("Connected to {} ({})", server, ip)
            } else {
                "Connected".to_string()
            }
        } else {
            "Disconnected".to_string()
        };
        state.show_notification(
            format!("Loaded {} servers | {}", state.servers.len(), conn_status),
            if state.connection.is_connected() {
                crate::state::NotificationType::Success
            } else {
                crate::state::NotificationType::Info
            },
        );

        Ok(Self { state, notification_timer: 30 })
    }

    pub fn run(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            if self.notification_timer > 0 {
                self.notification_timer -= 1;
                if self.notification_timer == 0 {
                    self.state.clear_notification();
                }
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        if let Some(action) = self.handle_key(key_event) {
                            match action {
                                AppAction::Quit => break,
                                AppAction::SwitchView => {
                                    self.state.switch_view();
                                }
                                AppAction::None => {}
                            }
                        }
                    }
                }
            }
        }

        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn handle_key(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        match key_event.code {
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Tab => Some(AppAction::SwitchView),
            KeyCode::Char('c') => {
                self.state.connect();
                self.notification_timer = 30;
                None
            }
            KeyCode::Char('d') => {
                self.state.disconnect();
                self.notification_timer = 30;
                None
            }
            KeyCode::Char('r') => {
                self.state.refresh_servers();
                self.notification_timer = 30;
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.select_next();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.select_prev();
                None
            }
            KeyCode::Char('?') => {
                self.state.current_view = AppView::Help;
                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(f.size());

        self.render_header(f, chunks[0]);
        self.render_notification(f, chunks[0]);
        self.render_main(f, chunks[1]);
        self.render_footer(f, chunks[2]);
    }

    fn render_notification(&self, f: &mut Frame<'_>, header_area: Rect) {
        if let Some(ref notification) = self.state.notification {
            let (fg_color, level_text) = match notification.notification_type {
                crate::state::NotificationType::Info => (Color::Cyan, "info"),
                crate::state::NotificationType::Success => (Color::Green, "success"),
                crate::state::NotificationType::Error => (Color::Red, "error"),
            };

            // Single line: [level] message
            let text = Line::from(vec![
                Span::styled(
                    format!("[{}]", level_text),
                    Style::default().fg(fg_color),
                ),
                Span::raw(" "),
                Span::styled(
                    notification.message.as_str(),
                    Style::default().fg(fg_color),
                ),
            ]);

            let area = Rect::new(
                header_area.x + 2,
                header_area.y + 1,
                header_area.x + header_area.width - 2,
                header_area.y + 2,
            );

            f.render_widget(Paragraph::new(text), area);
        }
    }

    fn render_header(&self, f: &mut Frame<'_>, area: Rect) {
        let title = " ProtonVPN TUI ";

        let connection_status = match &self.state.connection {
            crate::state::ConnectionState::Disconnected => "Disconnected",
            crate::state::ConnectionState::Connecting => "Connecting...",
            crate::state::ConnectionState::Connected { .. } => "Connected",
            crate::state::ConnectionState::Disconnecting => "Disconnecting...",
            crate::state::ConnectionState::Error(_) => "Error",
        };

        let status_text = Line::from(vec![
            Span::raw("Connection: "),
            Span::styled(
                connection_status,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | View: "),
            Span::styled(
                format!("{:?}", self.state.current_view),
                Style::default().fg(Color::Yellow),
            ),
        ]);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(status_text).alignment(ratatui::layout::Alignment::Center),
            area,
        );
    }

    fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
        match self.state.current_view {
            AppView::Connect => self.render_connect_view(f, area),
            AppView::Stats => self.render_stats_view(f, area),
            AppView::Settings => self.render_settings_view(f, area),
            AppView::Help => self.render_help_view(f, area),
        }
    }

    fn render_connect_view(&self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default().title(" Servers ").borders(Borders::ALL);

        let servers = self.state.filtered_servers();

        let items: Vec<ListItem> = servers
            .iter()
            .enumerate()
            .map(|(idx, server)| {
                let is_selected = self.state.selected_server == Some(idx);
                let load_str = format!("{}%", server.load);
                let ping_str = server
                    .ping
                    .map(|p| format!("{}ms", p))
                    .unwrap_or_else(|| "-".to_string());

                let line = if is_selected {
                    Line::from(vec![
                        Span::raw("> "),
                        Span::styled(
                            server.name.as_str(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" | "),
                        Span::raw(server.city.as_str()),
                        Span::raw(" | Load: "),
                        Span::styled(
                            load_str.clone(),
                            Style::default().fg(get_load_color(server.load)),
                        ),
                        Span::raw(" | Ping: "),
                        Span::raw(ping_str.clone()),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("  "),
                        Span::raw(server.name.as_str()),
                        Span::raw(" | "),
                        Span::raw(server.city.as_str()),
                        Span::raw(" | Load: "),
                        Span::styled(
                            load_str.clone(),
                            Style::default().fg(get_load_color(server.load)),
                        ),
                        Span::raw(" | Ping: "),
                        Span::raw(ping_str.clone()),
                    ])
                };

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn render_stats_view(&self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default().title(" Statistics ").borders(Borders::ALL);

        let stats = [
            "Download Speed: 0.00 KB/s",
            "Upload Speed: 0.00 KB/s",
            "Total Received: 0 B",
            "Total Sent: 0 B",
            "Session Time: 00:00:00",
            "Server IP: Not connected",
            "Protocol: N/A",
        ];

        let items: Vec<ListItem> = stats.iter().map(|s| ListItem::new(*s)).collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn render_settings_view(&self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default().title(" Settings ").borders(Borders::ALL);

        let settings = [
            "Theme: dark",
            "Kill Switch: off",
            "Secure Core: off",
            "IPv6: protected",
            "DNS: default",
            "Auto-connect: off",
            "Auto-start: off",
        ];

        let items: Vec<ListItem> = settings.iter().map(|s| ListItem::new(*s)).collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn render_help_view(&self, f: &mut Frame<'_>, area: Rect) {
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

    fn render_footer(&self, f: &mut Frame<'_>, area: Rect) {
        let action = match self.state.current_view {
            AppView::Connect => "j/k: move | c: connect | d: disconnect | r: refresh",
            AppView::Stats => "Statistics view",
            AppView::Settings => "Settings view",
            AppView::Help => "Press Tab or q to return",
        };

        let text = Line::from(vec![
            Span::raw("["),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw("] Help "),
            Span::raw("["),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw("] Switch View "),
            Span::raw("["),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw("] Quit"),
            Span::raw(" | "),
            Span::styled(action, Style::default().fg(Color::Cyan)),
        ]);

        f.render_widget(Paragraph::new(text), area);
    }
}

fn get_load_color(load: u8) -> Color {
    if load < 50 {
        Color::Green
    } else if load < 80 {
        Color::Yellow
    } else {
        Color::Red
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppAction {
    Quit,
    SwitchView,
    None,
}
