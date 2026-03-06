use crate::constants::ui::{
    NOTIFICATION_MSG_MAX_LEN, NOTIFICATION_TIMER_DEFAULT, NOTIFICATION_TIMER_SHORT,
    POPUP_WIDTH_MAX, POPUP_WIDTH_MIN,
};
use crate::state::{AppState, AppView, InputMode};
use crate::ui::styles::Theme;
use crate::ui::views;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::panic;

pub struct TuiApp {
    state: AppState,
    notification_timer: u8,
    list_state: ListState,
    pending_g: bool, // for gg command
    filter_mode: bool,
    filter_input: String,
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
            filter_mode: false,
            filter_input: String::new(),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
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
                    // Handle Ctrl+C for graceful shutdown
                    if key_event.code == KeyCode::Char('c')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
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
        if self.filter_mode || self.state.input_mode == InputMode::DnsInput {
            return self.handle_filter_input(key_event);
        }

        match key_event.code {
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Tab => Some(AppAction::SwitchView),
            KeyCode::Char('c') => {
                match self.state.current_view {
                    AppView::Cities => {
                        if let Some(idx) = self.state.selected_server {
                            let city_name =
                                self.state.current_cities.get(idx).map(|c| c.name.clone());
                            if let Some(name) = city_name {
                                self.state.connect_city(&name);
                                self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                            }
                        }
                    }
                    _ => {
                        self.state.connect();
                        self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                    }
                }
                None
            }
            KeyCode::Char(' ') => {
                if self.state.current_view == AppView::Settings {
                    if let Some(idx) = self.state.settings_selected {
                        self.state.toggle_settings_off(idx);
                        self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                    } else {
                        self.state.show_notification(
                            "No setting selected".to_string(),
                            crate::state::NotificationType::Info,
                        );
                    }
                }
                None
            }
            KeyCode::Enter => {
                match self.state.current_view {
                    AppView::Settings => {
                        if let Some(idx) = self.state.settings_selected {
                            self.state.toggle_settings(idx);
                            self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                        } else {
                            self.state.show_notification(
                                "No setting selected".to_string(),
                                crate::state::NotificationType::Info,
                            );
                        }
                    }
                    AppView::Servers => {
                        if let Some(idx) = self.state.selected_server {
                            let servers = self.state.filtered_servers();
                            if let Some(server) = servers.get(idx) {
                                self.state.fetch_cities(&server.id);
                                self.state.current_country_code = Some(server.id.clone());
                                self.state.current_view = AppView::Cities;
                                self.state.selected_server = Some(0);
                            }
                        }
                    }
                    AppView::Cities => {
                        if let Some(idx) = self.state.selected_server {
                            let city_name =
                                self.state.current_cities.get(idx).map(|c| c.name.clone());
                            if let Some(name) = city_name {
                                self.state.connect_city(&name);
                                self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                            }
                        }
                    }
                    _ => {}
                }
                None
            }
            // Ctrl+d = page down (must be before 'd' for disconnect)
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.state.current_view {
                    AppView::Servers => self.state.select_page_down(),
                    AppView::Cities => self.state.select_page_down(),
                    AppView::Settings => {
                        self.state.settings_select_page_down();
                    }
                    _ => {}
                }
                self.pending_g = false;
                None
            }
            KeyCode::Char('d') => {
                self.state.disconnect();
                self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                None
            }
            KeyCode::Char('r') => {
                self.state.refresh_servers();
                self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                None
            }
            KeyCode::Char('s') => {
                self.state.cycle_sort();
                self.notification_timer = NOTIFICATION_TIMER_SHORT;
                None
            }
            KeyCode::Char('f') => {
                self.state.cycle_sort_field();
                self.notification_timer = NOTIFICATION_TIMER_SHORT;
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.state.current_view {
                    AppView::Servers => self.state.select_next(),
                    AppView::Cities => self.state.select_next(),
                    AppView::Settings => {
                        self.state.settings_select_next();
                    }
                    _ => {}
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.state.current_view {
                    AppView::Servers => {
                        self.state.select_prev();
                        self.pending_g = false;
                    }
                    AppView::Cities => {
                        self.state.select_prev();
                        self.pending_g = false;
                    }
                    AppView::Settings => {
                        self.state.settings_select_prev();
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
                        AppView::Servers => self.state.select_first(),
                        AppView::Cities => self.state.select_first(),
                        AppView::Settings => {
                            self.state.settings_select_first();
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
                    AppView::Servers => self.state.select_last(),
                    AppView::Cities => self.state.select_last(),
                    AppView::Settings => {
                        self.state.settings_select_last();
                    }
                    _ => {}
                }
                self.pending_g = false;
                None
            }
            // Ctrl+u = page up
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.state.current_view {
                    AppView::Servers => self.state.select_page_up(),
                    AppView::Cities => self.state.select_page_up(),
                    AppView::Settings => {
                        self.state.settings_select_page_up();
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
                self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                None
            }
            KeyCode::Char('/') => {
                self.filter_mode = true;
                self.filter_input = self.state.search_query.clone();
                None
            }
            KeyCode::Esc => {
                if self.state.current_view == AppView::Cities {
                    self.state.current_view = AppView::Servers;
                    self.state.current_cities.clear();
                    self.state.current_country_code = None;
                }
                None
            }
            _ => None,
        }
    }

    fn handle_filter_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        let is_dns_input = self.state.input_mode == InputMode::DnsInput;

        match key_event.code {
            KeyCode::Esc => {
                self.filter_mode = false;
                self.filter_input.clear();
                self.state.set_search_query(String::new());
                if is_dns_input {
                    self.state.input_mode = InputMode::Normal;
                    self.state.dns_input.clear();
                }
                None
            }
            KeyCode::Enter => {
                if is_dns_input {
                    let dns_ips = self.state.dns_input.clone();
                    self.state.dns_input.clear();
                    self.state.input_mode = InputMode::Normal;
                    if !dns_ips.is_empty() {
                        self.state.apply_dns_setting(&dns_ips);
                        self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
                    }
                } else {
                    self.state.set_search_query(self.filter_input.clone());
                    self.filter_mode = false;
                    self.notification_timer = NOTIFICATION_TIMER_SHORT;
                }
                None
            }
            KeyCode::Backspace => {
                if is_dns_input {
                    self.state.dns_input.pop();
                } else {
                    self.filter_input.pop();
                    self.state.set_search_query(self.filter_input.clone());
                }
                None
            }
            KeyCode::Char(c) => {
                if is_dns_input {
                    self.state.dns_input.push(c);
                } else {
                    self.filter_input.push(c);
                    self.state.set_search_query(self.filter_input.clone());
                }
                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame<'_>) {
        // Apply theme background to entire terminal
        let theme = if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };
        let area = f.size();
        f.render_widget(
            Paragraph::new("").style(Style::default().bg(theme.background)),
            area,
        );

        if self.filter_mode {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            self.render_header(f, chunks[0]);
            self.render_main(f, chunks[1]);
            self.render_filter_input(f, chunks[2]);
        } else if self.state.input_mode == InputMode::DnsInput {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            self.render_header(f, chunks[0]);
            self.render_main(f, chunks[1]);
            self.render_dns_input(f, chunks[2]);
        } else {
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
        }

        // Render notification as popup last (on top)
        if self.state.notification.is_some() {
            self.render_notification_popup(f);
        }
    }

    fn render_filter_input(&self, f: &mut Frame<'_>, area: Rect) {
        let theme = if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };
        let prompt = "/ filter: ";
        let input_display = format!("{}{}", prompt, self.filter_input);
        let cursor = if self.filter_input.is_empty() {
            prompt.len()
        } else {
            prompt.len() + self.filter_input.len()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.key_hint));

        let text = Line::from(input_display.as_str());
        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(theme.foreground));

        f.render_widget(paragraph, area);

        if area.width > cursor as u16 + 2 {
            f.set_cursor(area.x + cursor as u16 + 1, area.y + 1);
        }
    }

    fn render_dns_input(&self, f: &mut Frame<'_>, area: Rect) {
        let theme = if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };
        let prompt = "DNS IPs (comma-separated): ";
        let input_display = format!("{}{}", prompt, self.state.dns_input);
        let cursor = if self.state.dns_input.is_empty() {
            prompt.len()
        } else {
            prompt.len() + self.state.dns_input.len()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.key_hint));

        let text = Line::from(input_display.as_str());
        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(theme.foreground));

        f.render_widget(paragraph, area);

        if area.width > cursor as u16 + 2 {
            f.set_cursor(area.x + cursor as u16 + 1, area.y + 1);
        }
    }

    fn render_notification_popup(&self, f: &mut Frame<'_>) {
        let Some(ref notification) = self.state.notification else {
            return;
        };

        let theme = if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };
        let (fg_color, title) = match notification.notification_type {
            crate::state::NotificationType::Info => (theme.primary, None),
            crate::state::NotificationType::Success => (theme.success, None),
            crate::state::NotificationType::Error => {
                let title = if notification.message.contains("Disconnect") {
                    Some("Disconnect failed")
                } else if notification.message.contains("Connect")
                    || notification.message.contains("Connection")
                {
                    Some("Connection failed")
                } else {
                    Some("Error")
                };
                (theme.error, title)
            }
        };

        let raw_msg = &notification.message;
        let msg_single_line = raw_msg.replace('\n', " ");
        let max_len = NOTIFICATION_MSG_MAX_LEN;
        let message = if msg_single_line.len() > max_len {
            msg_single_line[..max_len - 3].to_string()
        } else {
            msg_single_line
        };

        let popup_width = (message.len() + 4).clamp(POPUP_WIDTH_MIN, POPUP_WIDTH_MAX) as u16;
        let popup_height = if title.is_some() { 4 } else { 3 };

        let terminal = f.size();
        let x = terminal.width.saturating_sub(popup_width + 1);
        let y = 1;
        let area = Rect::new(x, y, popup_width, popup_height);

        let mut lines = Vec::new();
        if let Some(title) = title {
            lines.push(Line::from(title).centered());
        }
        lines.push(Line::from(message.as_str()).centered());

        let block = Block::bordered()
            .border_style(Style::default().fg(fg_color))
            .style(Style::default().fg(theme.foreground).bg(theme.background));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(theme.foreground))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn render_header(&self, f: &mut Frame<'_>, area: Rect) {
        let theme = if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };
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
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | View: "),
                Span::styled(
                    format!("{:?}", self.state.current_view),
                    Style::default().fg(theme.key_hint),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    connection_status,
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(server_info, Style::default().fg(theme.primary)),
                Span::raw(" | View: "),
                Span::styled(
                    format!("{:?}", self.state.current_view),
                    Style::default().fg(theme.key_hint),
                ),
            ])
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.primary));

        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(status_text).alignment(ratatui::layout::Alignment::Center),
            area,
        );
    }

    fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
        match self.state.current_view {
            AppView::Servers => views::servers_view::render_servers_view(
                &mut self.state,
                &mut self.list_state,
                f,
                area,
            ),
            AppView::Stats => views::stats_view::render_stats_view(&mut self.state, f, area),
            AppView::Settings => {
                views::settings_view::render_settings_view(&mut self.state, f, area)
            }
            AppView::Logs => {
                views::logs_view::render_logs_view(&self.state, &mut self.list_state, f, area)
            }
            AppView::Help => views::help_view::render_help_view(&self.state, f, area),
            AppView::Cities => views::cities_view::render_cities_view(
                &mut self.state,
                &mut self.list_state,
                f,
                area,
            ),
        }
    }

    fn render_footer(&self, f: &mut Frame<'_>, area: Rect) {
        let theme = if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };
        let sort_label = self.state.sort.label();
        let direction_label = self.state.sort_direction.label();
        let sort_display = format!("{} {}", sort_label, direction_label);

        let action_spans: Vec<Span<'_>> = match self.state.current_view {
            AppView::Servers => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] move | "),
                Span::raw("["),
                Span::styled("Enter", Style::default().fg(theme.key_hint)),
                Span::raw("] cities | "),
                Span::raw("["),
                Span::styled("c", Style::default().fg(theme.key_hint)),
                Span::raw("] connect | "),
                Span::raw("["),
                Span::styled("d", Style::default().fg(theme.key_hint)),
                Span::raw("] disconnect | "),
                Span::raw("["),
                Span::styled("r", Style::default().fg(theme.key_hint)),
                Span::raw("] refresh | "),
                Span::raw("["),
                Span::styled("s", Style::default().fg(theme.key_hint)),
                Span::raw("] sort ("),
                Span::raw(&sort_display),
                Span::raw(") | "),
                Span::raw("["),
                Span::styled("/", Style::default().fg(theme.key_hint)),
                Span::raw("] filter"),
            ],
            AppView::Cities => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] move | "),
                Span::raw("["),
                Span::styled("c/Enter", Style::default().fg(theme.key_hint)),
                Span::raw("] connect | "),
                Span::raw("["),
                Span::styled("Esc", Style::default().fg(theme.key_hint)),
                Span::raw("] back"),
            ],
            AppView::Stats => vec![Span::raw("statistics")],
            AppView::Settings => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] move | "),
                Span::raw("["),
                Span::styled("Enter", Style::default().fg(theme.key_hint)),
                Span::raw("] toggle/input | "),
                Span::raw("["),
                Span::styled("Space", Style::default().fg(theme.key_hint)),
                Span::raw("] off"),
            ],
            AppView::Logs => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] scroll"),
            ],
            AppView::Help => vec![Span::raw("Press Tab or q to return")],
        };

        let mut text = Line::from(vec![
            Span::raw("["),
            Span::styled("?", Style::default().fg(theme.key_hint)),
            Span::raw("] help "),
            Span::raw("["),
            Span::styled("Tab", Style::default().fg(theme.key_hint)),
            Span::raw("] switch view "),
            Span::raw("["),
            Span::styled("q", Style::default().fg(theme.key_hint)),
            Span::raw("] quit"),
            Span::raw(" | "),
        ]);

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
