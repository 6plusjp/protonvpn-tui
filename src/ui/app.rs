use crate::config::UserConfig;
use crate::state::{AppState, ConnectionState, InputMode};
use crate::ui::input::{self as input_handler, InputState};
use crate::ui::render::{Renderable, ServersViewState, View};
use crate::ui::renderers::{footer, header, input as input_renderer, notification};
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io;
use std::panic;
use std::time::Instant;

/// Main TUI application
pub struct TuiApp {
    state: AppState,
    current_view: View,
    pending_g: bool,
    filter_mode: bool,
    filter_input: String,
    last_render_time: Instant,
}

impl TuiApp {
    pub fn new(config: UserConfig) -> io::Result<Self> {
        panic::set_hook(Box::new(|_| {
            let _ = execute!(
                io::stdout(),
                LeaveAlternateScreen,
                SetCursorStyle::SteadyBar
            );
            let _ = disable_raw_mode();
        }));

        let key_bindings = config.keybindings.clone().into();
        let ui_config = config.ui.clone();
        let mut state = AppState::from_config(&key_bindings, &ui_config, config);
        state.refresh_servers();

        Ok(Self {
            state,
            current_view: View::Servers(ServersViewState::default()),
            pending_g: false,
            filter_mode: false,
            filter_input: String::new(),
            last_render_time: Instant::now(),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            SetCursorStyle::SteadyBar
        )?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            let async_processed = self
                .state
                .wait_for_async_events(std::time::Duration::from_millis(10));

            let notifications_expired = self.state.notification_state.tick();
            self.state.notification_state.flush_if_dirty();

            // Always redraw on first iteration to show loading screen
            let is_first_render = self.state.is_initialized;
            let is_connected = matches!(
                self.state.connection_manager.connection,
                ConnectionState::Connected { .. }
            );
            let elapsed = self.last_render_time.elapsed();
            let should_update_session =
                is_connected && elapsed >= std::time::Duration::from_secs(5);

            if !is_first_render || async_processed || notifications_expired || should_update_session
            {
                terminal.draw(|f| self.render(f))?;
                self.last_render_time = Instant::now();
            }

            // Check for keyboard input (non-blocking)
            if event::poll(std::time::Duration::from_millis(0))? {
                if let Event::Key(key_event) = event::read()? {
                    // Handle Ctrl+C for graceful shutdown
                    if key_event.code == crossterm::event::KeyCode::Char('c')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    if key_event.kind == KeyEventKind::Press {
                        let mut input_ctx = InputState::new(
                            &mut self.state,
                            &mut self.current_view,
                            &mut self.pending_g,
                            &mut self.filter_mode,
                            &mut self.filter_input,
                        );
                        if let Some(action) = input_handler::handle_key(&mut input_ctx, key_event) {
                            match action {
                                input_handler::AppAction::Quit => break,
                                input_handler::AppAction::SwitchView => {
                                    self.state.switch_view();
                                    let mut input_ctx = InputState::new(
                                        &mut self.state,
                                        &mut self.current_view,
                                        &mut self.pending_g,
                                        &mut self.filter_mode,
                                        &mut self.filter_input,
                                    );
                                    input_handler::sync_view(&mut input_ctx);
                                }
                            }
                        }
                    }
                }

                terminal.draw(|f| self.render(f))?;
            }
        }

        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            SetCursorStyle::SteadyBar
        )?;
        disable_raw_mode()?;
        Ok(())
    }

    fn render(&mut self, f: &mut Frame<'_>) {
        // Apply theme background to entire terminal
        let theme = self.state.theme();
        let area = f.area();
        f.render_widget(
            Paragraph::new("").style(Style::default().bg(theme.background)),
            area,
        );

        let has_filter_active = !self.state.ui_state.search_query.query.is_empty();
        let show_filter = self.filter_mode || has_filter_active;
        let is_dns_input = self.state.ui_state.input_mode == InputMode::DnsInput;
        let show_footer = self.state.ui_state.show_footer && !show_filter && !is_dns_input;

        if is_dns_input {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(f.area());

            header::render_header(&self.state, f, chunks[0]);
            self.render_main(f, chunks[1]);
            input_renderer::render_dns_input(&self.state, f, chunks[2]);
        } else {
            let use_bottom_row = show_filter || show_footer;
            let chunks = if use_bottom_row {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(f.area())
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(f.area())
            };

            header::render_header(&self.state, f, chunks[0]);

            if show_filter {
                self.render_main(f, chunks[1]);
                input_renderer::render_filter_input(
                    &self.state,
                    f,
                    chunks[2],
                    self.filter_mode,
                    &self.filter_input,
                    has_filter_active,
                );
            } else if show_footer {
                self.render_main(f, chunks[1]);
                footer::render_footer(&self.state, f, chunks[2]);
            } else {
                self.render_main(f, chunks[1]);
            }
        }

        // Render notification as popup last (on top)
        if !self.state.notification_state.notifications.is_empty() {
            notification::render_notification_popup(&self.state, f);
        }
    }

    fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.current_view.render(&mut self.state, f, area);
    }
}
