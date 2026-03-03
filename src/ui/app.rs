use crate::state::{AppState, AppView};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::panic;

pub struct TuiApp {
    state: AppState,
    notification_timer: u8,
    list_state: ListState,
    pending_g: bool, // for gg command
}

impl TuiApp {
    pub fn new() -> io::Result<Self> {
        panic::set_hook(Box::new(|_| {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = disable_raw_mode();
        }));

        let mut state = AppState::new();
        state.sync_connection_state();
        state.refresh_servers();

        Ok(Self {
            state,
            notification_timer: 30,
            list_state: ListState::default(),
            pending_g: false,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;

        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            // Sync connection state (checks for background connection completion)
            self.state.sync_connection_state();

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
            KeyCode::Char('c') | KeyCode::Enter => {
                self.state.connect();
                self.notification_timer = 30;
                None
            }
            // Ctrl+d = page down (must be before 'd' for disconnect)
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.state.current_view {
                    AppView::Connect => self.state.select_page_down(),
                    AppView::Settings => {
                        let count = crate::config::Settings::load_proton_settings()
                            .map(|ps| ps.settings_count())
                            .unwrap_or(0);
                        self.state.settings_select_page_down(count);
                    }
                    _ => {}
                }
                self.pending_g = false;
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
            KeyCode::Char('s') => {
                self.state.cycle_sort();
                self.notification_timer = 15;
                None
            }
            KeyCode::Char('f') => {
                self.state.cycle_sort_field();
                self.notification_timer = 15;
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.state.current_view {
                    AppView::Connect => self.state.select_next(),
                    AppView::Settings => {
                        let count = crate::config::Settings::load_proton_settings()
                            .map(|ps| ps.settings_count())
                            .unwrap_or(0);
                        self.state.settings_select_next(count);
                    }
                    _ => {}
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.state.current_view {
                    AppView::Connect => {
                        self.state.select_prev();
                        self.pending_g = false;
                    }
                    AppView::Settings => {
                        let count = crate::config::Settings::load_proton_settings()
                            .map(|ps| ps.settings_count())
                            .unwrap_or(0);
                        self.state.settings_select_prev(count);
                        self.pending_g = false;
                    }
                    _ => {}
                }
                None
            }
            // Vim: gg = go to top
            KeyCode::Char('g') => {
                if self.pending_g {
                    match self.state.current_view {
                        AppView::Connect => self.state.select_first(),
                        AppView::Settings => {
                            let count = crate::config::Settings::load_proton_settings()
                                .map(|ps| ps.settings_count())
                                .unwrap_or(0);
                            self.state.settings_select_first(count);
                        }
                        _ => {}
                    }
                    self.pending_g = false;
                } else {
                    self.pending_g = true;
                }
                None
            }
            // Vim: G = go to bottom
            KeyCode::Char('G') => {
                match self.state.current_view {
                    AppView::Connect => self.state.select_last(),
                    AppView::Settings => {
                        let count = crate::config::Settings::load_proton_settings()
                            .map(|ps| ps.settings_count())
                            .unwrap_or(0);
                        self.state.settings_select_last(count);
                    }
                    _ => {}
                }
                self.pending_g = false;
                None
            }
            // Ctrl+u = page up
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.state.current_view {
                    AppView::Connect => self.state.select_page_up(),
                    AppView::Settings => {
                        let count = crate::config::Settings::load_proton_settings()
                            .map(|ps| ps.settings_count())
                            .unwrap_or(0);
                        self.state.settings_select_page_up(count);
                    }
                    _ => {}
                }
                self.pending_g = false;
                None
            }
            KeyCode::Char('?') => {
                self.state.current_view = AppView::Help;
                None
            }
            KeyCode::Char('x') => {
                self.state.connect_random();
                self.notification_timer = 30;
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
        self.render_main(f, chunks[1]);
        self.render_footer(f, chunks[2]);

        // Render notification as popup last (on top)
        if self.state.notification.is_some() {
            self.render_notification_popup(f);
        }
    }
    fn render_notification_popup(&self, f: &mut Frame<'_>) {
        let Some(ref notification) = self.state.notification else {
            return;
        };

        let (fg_color, title) = match notification.notification_type {
            crate::state::NotificationType::Info => (Color::Cyan, None),
            crate::state::NotificationType::Success => (Color::Green, None),
            crate::state::NotificationType::Error => {
                // Extract title from error message or use default
                let title = if notification.message.contains("Disconnect") {
                    Some("Disconnect failed")
                } else if notification.message.contains("Connect")
                    || notification.message.contains("Connection")
                {
                    Some("Connection failed")
                } else {
                    Some("Error")
                };
                (Color::Red, title)
            }
        };

        // Format message - replace newlines with space, truncate with ...
        let raw_msg = &notification.message;
        let msg_single_line = raw_msg.replace('\n', " ");
        let max_len = 35;
        let message = if msg_single_line.len() > max_len {
            format!("{}", &msg_single_line[..max_len - 3])
        } else {
            msg_single_line
        };

        // Calculate popup size
        let popup_width = (message.len() + 4).max(30).min(54) as u16;
        let popup_height = if title.is_some() { 4 } else { 3 };

        // Position in top-right corner
        let terminal = f.size();
        let x = terminal.width.saturating_sub(popup_width + 1);
        let y = 1;
        let area = Rect::new(x, y, popup_width, popup_height);

        // Build content lines
        let mut lines = Vec::new();
        if let Some(title) = title {
            lines.push(Line::from(title).centered());
        }
        lines.push(Line::from(message.as_str()).centered());

        // Create the popup block with background
        let block = Block::bordered()
            .border_style(Style::default().fg(fg_color))
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);

        // Clear the area first, then render popup
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn render_header(&self, f: &mut Frame<'_>, area: Rect) {
        let title = " ProtonVPN TUI ";

        let (connection_status, server_info) = match &self.state.connection {
            crate::state::ConnectionState::Disconnected => ("Disconnected", String::new()),
            crate::state::ConnectionState::Connecting => ("Connecting...", String::new()),
            crate::state::ConnectionState::Connected { server, ip } => {
                let info = if ip.is_empty() {
                    server.clone()
                } else {
                    format!("{} ({})", server, ip)
                };
                ("Connected", info)
            }
            crate::state::ConnectionState::Disconnecting => ("Disconnecting...", String::new()),
            crate::state::ConnectionState::Error(e) => ("Error", e.clone()),
        };

        let status_text = if server_info.is_empty() {
            Line::from(vec![
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
            ])
        } else {
            Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    connection_status,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(server_info, Style::default().fg(Color::Cyan)),
                Span::raw(" | View: "),
                Span::styled(
                    format!("{:?}", self.state.current_view),
                    Style::default().fg(Color::Yellow),
                ),
            ])
        };

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

    fn render_connect_view(&mut self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default().title(" Servers ").borders(Borders::ALL);

        let servers = self.state.filtered_servers();

        // Update list state with selected server
        if let Some(idx) = self.state.selected_server {
            self.list_state.select(Some(idx));
        }

        let connected_server_id = match &self.state.connection {
            crate::state::ConnectionState::Connected { server, .. } => Some(server.clone()),
            _ => None,
        };

        let items: Vec<ListItem> = servers
            .iter()
            .enumerate()
            .map(|(idx, server)| {
                let is_selected = self.state.selected_server == Some(idx);
                let is_connected = connected_server_id
                    .as_ref()
                    .map(|cid| cid.starts_with(&server.id) || server.id.starts_with(cid))
                    .unwrap_or(false);

                let line = if is_selected {
                    Line::from(vec![
                        Span::raw("> "),
                        Span::styled(
                            server.name.as_str(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else if is_connected {
                    Line::from(vec![
                        Span::raw("* "),
                        Span::styled(
                            server.name.as_str(),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    Line::from(vec![Span::raw("  "), Span::raw(server.name.as_str())])
                };

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White));

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_stats_view(&self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default().title(" Statistics ").borders(Borders::ALL);

        let proton_settings = crate::config::Settings::load_proton_settings();

        let mut stats = Vec::new();

        if let Some(ps) = proton_settings {
            if let Some(protocol) = ps.protocol {
                stats.push(format!("Protocol: {}", protocol));
            }
            if let Some(features) = ps.features {
                if let Some(st) = features.split_tunneling {
                    let mode = st.mode.as_deref().unwrap_or("unknown");
                    let status = if st.enabled { "on" } else { "off" };
                    stats.push(format!("Split Tunneling: {} ({})", status, mode));
                }
            }
        }

        stats.extend([
            "Download Speed: 0.00 KB/s".to_string(),
            "Upload Speed: 0.00 KB/s".to_string(),
            "Total Received: 0 B".to_string(),
            "Total Sent: 0 B".to_string(),
            "Session Time: 00:00:00".to_string(),
        ]);

        if let crate::state::ConnectionState::Connected { ref server, ref ip } =
            self.state.connection
        {
            stats.push(format!("Server: {}", server));
            if !ip.is_empty() {
                stats.push(format!("IP: {}", ip));
            }
        }

        let items: Vec<ListItem> = stats.iter().map(|s| ListItem::new(s.as_str())).collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn render_settings_view(&mut self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default().title(" Settings ").borders(Borders::ALL);

        let proton_settings = crate::config::Settings::load_proton_settings();

        let settings = match proton_settings {
            Some(ps) => {
                let killswitch = match ps.killswitch {
                    Some(0) => "off",
                    Some(1) => "on",
                    _ => "unknown",
                };
                let ipv6 = if ps.ipv6 == Some(true) {
                    "enabled"
                } else {
                    "disabled"
                };
                let dns = if ps.custom_dns.enabled {
                    format!("custom ({})", ps.custom_dns.ip_list.join(", "))
                } else {
                    "default".to_string()
                };
                let netshield = match ps.features.as_ref().and_then(|f| f.netshield) {
                    Some(0) => "off",
                    Some(1) => "standard",
                    Some(2) => "plus",
                    _ => "unknown",
                };
                let moderate_nat = ps
                    .features
                    .as_ref()
                    .and_then(|f| f.moderate_nat)
                    .map(|v| if v { "on" } else { "off" })
                    .unwrap_or("off");
                let vpn_accelerator = ps
                    .features
                    .as_ref()
                    .and_then(|f| f.vpn_accelerator)
                    .map(|v| if v { "on" } else { "off" })
                    .unwrap_or("off");
                let port_forwarding = ps
                    .features
                    .as_ref()
                    .and_then(|f| f.port_forwarding)
                    .map(|v| if v { "on" } else { "off" })
                    .unwrap_or("off");

                vec![
                    format!("Kill Switch: {}", killswitch),
                    format!("IPv6: {}", ipv6),
                    format!("DNS: {}", dns),
                    format!("NetShield: {}", netshield),
                    format!("Moderate NAT: {}", moderate_nat),
                    format!("VPN Accelerator: {}", vpn_accelerator),
                    format!("Port Forwarding: {}", port_forwarding),
                ]
            }
            None => vec!["No Proton settings found".to_string()],
        };

        let items: Vec<ListItem> = settings
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let is_selected = self.state.settings_selected == Some(idx);
                let line = if is_selected {
                    Line::from(vec![
                        Span::raw("> "),
                        Span::styled(
                            s.as_str(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    Line::from(vec![Span::raw("  "), Span::raw(s.as_str())])
                };
                ListItem::new(line)
            })
            .collect();

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

    fn render_footer(&self, f: &mut Frame<'_>, area: Rect) {
        let sort_label = self.state.sort.label();
        let direction_label = self.state.sort_direction.label();
        let sort_display = format!("{} {}", sort_label, direction_label);

        let action_spans: Vec<Span<'_>> = match self.state.current_view {
            AppView::Connect => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw("] move | "),
                Span::raw("["),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw("] connect | "),
                Span::raw("["),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw("] disconnect | "),
                Span::raw("["),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw("] refresh | "),
                Span::raw("["),
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw("] sort ("),
                Span::raw(&sort_display),
                Span::raw(")"),
            ],
            AppView::Stats => vec![Span::raw("statistics")],
            AppView::Settings => vec![Span::raw("settings")],
            AppView::Help => vec![Span::raw("Press Tab or q to return")],
        };

        let text = Line::from(vec![
            Span::raw("["),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw("] help "),
            Span::raw("["),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw("] switch view "),
            Span::raw("["),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw("] quit"),
            Span::raw(" | "),
        ]);

        let mut text = Line::from(text);
        text.spans.extend(action_spans);

        f.render_widget(Paragraph::new(text), area);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppAction {
    Quit,
    SwitchView,
    None,
}
