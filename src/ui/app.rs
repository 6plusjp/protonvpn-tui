use crate::constants::ui::{NOTIFICATION_MSG_MAX_LEN, POPUP_WIDTH_MAX, POPUP_WIDTH_MIN};
use crate::state::{AppState, AppView, InputMode, Pane};
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
    countries_list_state: ListState,
    cities_list_state: ListState,
    pending_g: bool,
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
            countries_list_state: ListState::default(),
            cities_list_state: ListState::default(),
            pending_g: false,
            filter_mode: false,
            filter_input: String::new(),
        })
    }

    pub fn get_theme(&self) -> Theme {
        if self.state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            // Sync connection state (checks for background connection completion)
            let _notification_shown = self.state.sync_connection_state();

            self.state.tick_notifications();

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

        if let Some(action) = self.handle_common_keys(key_event) {
            return Some(action);
        }

        match self.state.get_current_view() {
            AppView::Servers => self.handle_servers_key(key_event),
            AppView::Settings => self.handle_settings_key(key_event),
            AppView::Logs => self.handle_logs_key(key_event),
            AppView::Help => self.handle_help_key(key_event),
        }
    }

    fn handle_common_keys(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        match key_event.code {
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Tab => Some(AppAction::SwitchView),
            KeyCode::Char('?') => {
                self.state.current_view = AppView::Help;
                None
            }
            KeyCode::Char('/') => {
                self.filter_mode = true;
                self.filter_input = self.state.search_query.clone();
                None
            }
            KeyCode::Esc => None,
            _ => None,
        }
    }

    fn handle_servers_key(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        match key_event.code {
            KeyCode::Char('c') => {
                self.handle_connect();
                None
            }
            KeyCode::Char('l') => {
                self.state.move_to_cities();

                None
            }
            KeyCode::Char('h') => {
                if self.state.get_pane_focus() == Pane::Cities {
                    self.state.move_to_countries();
                }
                None
            }
            KeyCode::Backspace => {
                if self.state.get_pane_focus() == Pane::Cities {
                    self.state.move_to_countries();
                }
                None
            }
            KeyCode::Enter => {
                if self.state.get_pane_focus() == Pane::Cities {
                    self.handle_connect();
                } else {
                    self.state.move_to_cities();
                }
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.handle_navigation_down();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.handle_navigation_up();
                None
            }
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_page_down();
                None
            }
            KeyCode::Char('d') => {
                self.handle_disconnect();
                None
            }
            KeyCode::Char('r') => {
                self.handle_refresh();
                None
            }
            KeyCode::Char('s') => {
                self.handle_cycle_sort();
                None
            }
            KeyCode::Char('f') => {
                self.handle_cycle_sort_field();
                None
            }
            KeyCode::Char('g') => {
                self.handle_go_to_first();
                None
            }
            KeyCode::Char('G') => {
                self.handle_go_to_last();
                None
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_page_up();
                None
            }
            KeyCode::Char('x') => {
                self.handle_connect_random();
                None
            }
            _ => None,
        }
    }

    fn handle_settings_key(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        match key_event.code {
            KeyCode::Char(' ') => {
                if let Some(idx) = self.state.settings_selected {
                    self.state.toggle_settings_off(idx);
                } else {
                    self.state.show_notification(
                        "No setting selected".to_string(),
                        crate::state::NotificationType::Info,
                    );
                }
                None
            }
            KeyCode::Enter => {
                if let Some(idx) = self.state.settings_selected {
                    self.state.toggle_settings(idx);
                } else {
                    self.state.show_notification(
                        "No setting selected".to_string(),
                        crate::state::NotificationType::Info,
                    );
                }
                None
            }
            KeyCode::Char('c') => {
                self.handle_connect();
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.handle_navigation_down();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.handle_navigation_up();
                None
            }
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_page_down();
                None
            }
            KeyCode::Char('g') => {
                self.handle_go_to_first();
                None
            }
            KeyCode::Char('G') => {
                self.handle_go_to_last();
                None
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_page_up();
                None
            }
            _ => None,
        }
    }

    fn handle_logs_key(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.handle_navigation_down();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.handle_navigation_up();
                None
            }
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_page_down();
                None
            }
            KeyCode::Char('g') => {
                self.handle_go_to_first();
                None
            }
            KeyCode::Char('G') => {
                self.handle_go_to_last();
                None
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_page_up();
                None
            }
            _ => None,
        }
    }

    fn handle_help_key(&mut self, key_event: crossterm::event::KeyEvent) -> Option<AppAction> {
        let _ = key_event;
        None
    }

    fn handle_navigation_down(&mut self) {
        match (self.state.get_current_view(), self.state.get_pane_focus()) {
            (AppView::Servers, Pane::Cities) => self.state.city_select_next(),
            (AppView::Servers, Pane::Countries) => self.state.select_next(),
            (AppView::Settings, _) => self.state.settings_select_next(),
            _ => {}
        }
    }

    fn handle_navigation_up(&mut self) {
        self.pending_g = false;
        match (self.state.get_current_view(), self.state.get_pane_focus()) {
            (AppView::Servers, Pane::Cities) => self.state.city_select_prev(),
            (AppView::Servers, Pane::Countries) => self.state.select_prev(),
            (AppView::Settings, _) => self.state.settings_select_prev(),
            _ => {}
        }
    }

    fn handle_page_down(&mut self) {
        self.pending_g = false;
        match (self.state.get_current_view(), self.state.get_pane_focus()) {
            (AppView::Servers, Pane::Cities) => self.state.city_select_page_down(),
            (AppView::Servers, Pane::Countries) => self.state.select_page_down(),
            (AppView::Settings, _) => self.state.settings_select_page_down(),
            _ => {}
        }
    }

    fn handle_page_up(&mut self) {
        self.pending_g = false;
        match (self.state.get_current_view(), self.state.get_pane_focus()) {
            (AppView::Servers, Pane::Cities) => self.state.city_select_page_up(),
            (AppView::Servers, Pane::Countries) => self.state.select_page_up(),
            (AppView::Settings, _) => self.state.settings_select_page_up(),
            _ => {}
        }
    }

    fn handle_go_to_first(&mut self) {
        if self.pending_g {
            match (self.state.get_current_view(), self.state.get_pane_focus()) {
                (AppView::Servers, Pane::Cities) => self.state.city_select_first(),
                (AppView::Servers, Pane::Countries) => self.state.select_first(),
                (AppView::Settings, _) => self.state.settings_select_first(),
                _ => {}
            }
            self.pending_g = false;
        } else {
            self.pending_g = true;
        }
    }

    fn handle_go_to_last(&mut self) {
        self.pending_g = false;
        match (self.state.get_current_view(), self.state.get_pane_focus()) {
            (AppView::Servers, Pane::Cities) => self.state.city_select_last(),
            (AppView::Servers, Pane::Countries) => self.state.select_last(),
            (AppView::Settings, _) => self.state.settings_select_last(),
            _ => {}
        }
    }

    fn handle_connect(&mut self) {
        if self.state.get_pane_focus() == Pane::Cities {
            if let Some(idx) = self.state.get_selected_city() {
                let city_name = self.state.current_cities.get(idx).map(|c| c.name.clone());
                if let Some(name) = city_name {
                    self.state.connect_city(&name);
                }
            }
        } else {
            self.state.connect();
        }
    }

    fn handle_disconnect(&mut self) {
        self.state.disconnect();
    }

    fn handle_connect_random(&mut self) {
        self.state.connect_random();
    }

    fn handle_refresh(&mut self) {
        self.state.refresh_servers();
    }

    fn handle_cycle_sort(&mut self) {
        self.state.cycle_sort();
    }

    fn handle_cycle_sort_field(&mut self) {
        self.state.cycle_sort_field();
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
                    }
                } else {
                    self.state.set_search_query(self.filter_input.clone());
                    self.filter_mode = false;
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
        let theme = self.get_theme();
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
        if !self.state.notifications.is_empty() {
            self.render_notification_popup(f);
        }
    }

    fn render_filter_input(&self, f: &mut Frame<'_>, area: Rect) {
        let theme = self.get_theme();
        let prompt = "filter: ";
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
        let theme = self.get_theme();
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
        let theme = self.get_theme();
        let terminal = f.size();

        let notifications: Vec<_> = self.state.notifications.iter().rev().collect();

        for (i, notification) in notifications.iter().enumerate() {
            let position_from_bottom = i;
            let popup_height = 3u16;

            let y = terminal
                .height
                .saturating_sub(3 + (position_from_bottom as u16 * popup_height));
            if y < 1 {
                break;
            }

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

            let x = terminal.width.saturating_sub(popup_width + 1);
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
    }

    fn render_header(&mut self, f: &mut Frame<'_>, area: Rect) {
        let theme = self.get_theme();

        let (status_text, status_color): (String, _) = match self.state.get_connection() {
            crate::state::ConnectionState::Disconnected => {
                ("Disconnected".to_string(), theme.foreground)
            }
            crate::state::ConnectionState::Connecting => {
                ("Connecting...".to_string(), theme.warning)
            }
            crate::state::ConnectionState::Connected { server, ip } => {
                let info = if ip.is_empty() {
                    server.clone()
                } else {
                    format!("{} ({})", server, ip)
                };
                (info, theme.success)
            }
            crate::state::ConnectionState::Disconnecting => {
                ("Disconnecting...".to_string(), theme.warning)
            }
            crate::state::ConnectionState::Error(e) => (e.clone(), theme.error),
        };

        let protocol = self.state.get_proton_protocol();

        let title = " ProtonVPN TUI ";

        let status_indicator = match self.state.get_connection() {
            crate::state::ConnectionState::Connected { .. } => "●",
            crate::state::ConnectionState::Connecting
            | crate::state::ConnectionState::Disconnecting => "◐",
            crate::state::ConnectionState::Disconnected => "○",
            crate::state::ConnectionState::Error(_) => "✕",
        };

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

        let mut status_spans = vec![
            Span::styled(
                status_indicator,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ];

        if let Some(proto) = protocol {
            status_spans.push(Span::raw("  |  "));
            status_spans.push(Span::styled(proto, Style::default().fg(theme.secondary)));
        }

        let status_line = Line::from(status_spans);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.block_border));

        f.render_widget(block, area);
        f.render_widget(Paragraph::new(title_line), chunks[0]);
        f.render_widget(
            Paragraph::new(status_line).alignment(ratatui::layout::Alignment::Center),
            chunks[1],
        );
    }

    fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
        match self.state.get_current_view() {
            AppView::Servers => views::servers_view::render_servers_view(
                &mut self.state,
                &mut self.countries_list_state,
                &mut self.cities_list_state,
                f,
                area,
            ),
            AppView::Settings => {
                views::settings_view::render_settings_view(&mut self.state, f, area)
            }
            AppView::Logs => views::logs_view::render_logs_view(
                &self.state,
                &mut self.countries_list_state,
                f,
                area,
            ),
            AppView::Help => views::help_view::render_help_view(&self.state, f, area),
        }
    }

    fn render_footer(&self, f: &mut Frame<'_>, area: Rect) {
        let theme = self.get_theme();
        let sort_label = self.state.sort.label();
        let direction_label = self.state.sort_direction.label();
        let _sort_display = format!("{} {}", sort_label, direction_label);

        let action_spans = self.get_footer_action_hints();

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
            Span::raw(" "),
        ]);

        text.spans.extend(action_spans);

        f.render_widget(Paragraph::new(text), area);
    }

    fn get_footer_action_hints(&self) -> Vec<Span<'_>> {
        let theme = self.get_theme();

        match (self.state.get_current_view(), self.state.get_pane_focus()) {
            (AppView::Servers, Pane::Countries) => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] navigate "),
                Span::raw("["),
                Span::styled("l/Enter", Style::default().fg(theme.key_hint)),
                Span::raw("] cities "),
                Span::raw("["),
                Span::styled("c", Style::default().fg(theme.key_hint)),
                Span::raw("] connect "),
                Span::raw("["),
                Span::styled("d", Style::default().fg(theme.key_hint)),
                Span::raw("] disconnect "),
                Span::raw("["),
                Span::styled("r", Style::default().fg(theme.key_hint)),
                Span::raw("] refresh "),
                Span::raw("["),
                Span::styled("s", Style::default().fg(theme.key_hint)),
                Span::raw("] sort "),
                Span::raw("["),
                Span::styled("f", Style::default().fg(theme.key_hint)),
                Span::raw("] field "),
                Span::raw("["),
                Span::styled("/", Style::default().fg(theme.key_hint)),
                Span::raw("] filter"),
            ],
            (AppView::Servers, Pane::Cities) => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] navigate "),
                Span::raw("["),
                Span::styled("c/Enter", Style::default().fg(theme.key_hint)),
                Span::raw("] connect "),
                Span::raw("["),
                Span::styled("h/Backspace", Style::default().fg(theme.key_hint)),
                Span::raw("] countries"),
            ],
            (AppView::Settings, _) => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] move "),
                Span::raw("["),
                Span::styled("Enter", Style::default().fg(theme.key_hint)),
                Span::raw("] toggle/input "),
                Span::raw("["),
                Span::styled("Space", Style::default().fg(theme.key_hint)),
                Span::raw("] off"),
            ],
            (AppView::Logs, _) => vec![
                Span::raw("["),
                Span::styled("j/k", Style::default().fg(theme.key_hint)),
                Span::raw("] scroll"),
            ],
            (AppView::Help, _) => vec![Span::raw("Press Tab or q to return")],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppAction {
    Quit,
    SwitchView,
    None,
}
