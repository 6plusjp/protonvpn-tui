use crate::state::{AppState, AppView};
use crate::ui::views;
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
            AppView::Connect => views::connect_view::render_connect_view(
                &mut self.state,
                &mut self.list_state,
                f,
                area,
            ),
            AppView::Stats => views::stats_view::render_stats_view(&self.state, f, area),
            AppView::Settings => {
                views::settings_view::render_settings_view(&mut self.state, f, area)
            }
            AppView::Help => views::help_view::render_help_view(f, area),
        }
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
