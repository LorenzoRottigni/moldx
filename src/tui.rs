use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, sync::Arc, time::Duration};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

use crate::{
    client::MoldXClient,
    command::Command,
    executor::{self, Executor},
    module::Module,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Panel {
    Modules,
    Commands,
    Running,
}

impl Panel {
    fn next(self) -> Self {
        match self {
            Panel::Modules => Panel::Commands,
            Panel::Commands => Panel::Running,
            Panel::Running => Panel::Modules,
        }
    }

    fn prev(self) -> Self {
        match self {
            Panel::Modules => Panel::Running,
            Panel::Commands => Panel::Modules,
            Panel::Running => Panel::Commands,
        }
    }
}

#[derive(Debug, Clone)]
struct CommandItem {
    strategy: String,
    command: Command,
}

struct TuiApp {
    client: Arc<MoldXClient>,
    modules: Vec<Module>,
    active_panel: Panel,
    module_idx: usize,
    command_idx: usize,
    running_idx: usize,
    command_items: Vec<CommandItem>,
}

impl TuiApp {
    fn new(client: Arc<MoldXClient>) -> Self {
        let mut app = Self {
            client,
            modules: vec![],
            active_panel: Panel::Modules,
            module_idx: 0,
            command_idx: 0,
            running_idx: 0,
            command_items: vec![],
        };
        app.modules = app.client.modules.clone();
        app.rebuild_command_items();
        app
    }

    fn rebuild_command_items(&mut self) {
        self.command_items.clear();
        self.command_idx = 0;
        if let Some(module) = self.modules.get(self.module_idx) {
            for strategy_index in &module.strategies {
                let strategy = &self.client.strategies[*strategy_index];
                for command in &strategy.commands {
                    self.command_items.push(CommandItem {
                        strategy: strategy.name.clone(),
                        command: command.clone(),
                    });
                }
            }
        }
    }

    fn selected_module(&self) -> Option<&Module> {
        self.modules.get(self.module_idx)
    }

    fn move_cursor(&mut self, delta: i32) {
        match self.active_panel {
            Panel::Modules => {
                let len = self.modules.len();
                if len == 0 {
                    return;
                }
                if delta < 0 {
                    self.module_idx = if self.module_idx == 0 { len - 1 } else { self.module_idx - 1 };
                } else {
                    self.module_idx = (self.module_idx + 1) % len;
                }
                self.rebuild_command_items();
            }
            Panel::Commands => {
                let len = self.command_items.len();
                if len == 0 {
                    return;
                }
                if delta < 0 {
                    self.command_idx = if self.command_idx == 0 { len - 1 } else { self.command_idx - 1 };
                } else {
                    self.command_idx = (self.command_idx + 1) % len;
                }
            }
            Panel::Running => {
                let len = self.client.executor.get_summaries().len();
                if len == 0 {
                    return;
                }
                if delta < 0 {
                    self.running_idx = if self.running_idx == 0 { len - 1 } else { self.running_idx - 1 };
                } else {
                    self.running_idx = (self.running_idx + 1) % len;
                }
            }
        }
    }

    fn run_selected_command(&mut self) {
        let module = match self.selected_module() {
            Some(module) => module,
            None => return,
        };
        let item = match self.command_items.get(self.command_idx) {
            Some(item) => item,
            None => return,
        };

        let script = item.command.dir.clone();
        let id = self.client.executor.add_process(
            &module.dir.to_string_lossy(),
            &item.strategy,
            &item.command.name,
            None,
        );

        let executor = Arc::new(self.client.executor.clone());
        tokio::spawn(executor::run_and_track(
            executor,
            id,
            script,
            module.dir.clone(),
        ));
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }
        if key.code == KeyCode::Char('q') {
            return true;
        }

        match key.code {
            KeyCode::Tab => self.active_panel = self.active_panel.next(),
            KeyCode::BackTab => self.active_panel = self.active_panel.prev(),
            KeyCode::Up => self.move_cursor(-1),
            KeyCode::Down => self.move_cursor(1),
            KeyCode::Enter => match self.active_panel {
                Panel::Modules => {
                    self.active_panel = Panel::Commands;
                }
                Panel::Commands => self.run_selected_command(),
                Panel::Running => {}
            },
            KeyCode::Char('k') => {
                let summaries = self.client.executor.get_summaries();
                if let Some(summary) = summaries.get(self.running_idx) {
                    self.client.executor.kill_process(summary.id);
                }
            }
            _ => {}
        }

        false
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn panel_block(title: &str, active: bool) -> Block<'_> {
    let border_style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(if active { Color::Cyan } else { Color::Gray })
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style)
}

fn draw_modules(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let active = app.active_panel == Panel::Modules;
    let block = panel_block("Modules", active);

    if app.modules.is_empty() {
        let p = Paragraph::new("No modules found")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app
        .modules
        .iter()
        .enumerate()
        .map(|(i, module)| {
            let name = module
                .dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            let style = if i == app.module_idx && active {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if i == app.module_idx {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(name, style),
                Span::raw(format!("  {}", module.dir.display())),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.module_idx));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_commands(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let active = app.active_panel == Panel::Commands;
    let block = panel_block("Commands", active);

    if app.command_items.is_empty() {
        let p = Paragraph::new("No commands available")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app
        .command_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.command_idx && active {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if i == app.command_idx {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", item.strategy), Style::default().fg(Color::Magenta)),
                Span::styled(item.command.name.as_str(), style),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.command_idx));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_running(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let active = app.active_panel == Panel::Running;
    let summaries = app.client.executor.get_summaries();
    let block = panel_block("Running", active);

    if summaries.is_empty() {
        let p = Paragraph::new("No processes yet")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = summaries
        .iter()
        .enumerate()
        .map(|(i, summary)| {
            let style = if i == app.running_idx && active {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("#{} ", summary.id), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}/{}", summary.strategy, summary.command), style),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.running_idx.min(summaries.len().saturating_sub(1))));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(Color::Cyan)),
        Span::raw(":panel  "),
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(":nav  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(":run  "),
        Span::styled("k", Style::default().fg(Color::Cyan)),
        Span::raw(":kill  "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(":quit"),
    ]))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(help, area);
}

fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(35), Constraint::Percentage(30)])
        .split(outer[0]);

    draw_modules(frame, app, columns[0]);
    draw_commands(frame, app, columns[1]);
    draw_running(frame, app, columns[2]);
    draw_help(frame, outer[1]);
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        tokio::select! {
            _ = &mut ctrl_c => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

pub async fn run(client: &MoldXClient) -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let client = Arc::new(MoldXClient {
        strategies: client.strategies.clone(),
        modules: client.modules.clone(),
        config: client.config.clone(),
        executor: Executor::new(),
    });
    let mut session = TuiSession::new(setup_terminal()?);
    let mut app = TuiApp::new(client);

    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(250));

    loop {
        {
            let terminal = session
                .terminal_mut()
                .expect("terminal should remain available during TUI run");
            terminal.draw(|f| draw(f, &mut app))?;
        }

        tokio::select! {
            _ = tick.tick() => {}
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) if app.handle_key(key) => break,
                    None => break,
                    _ => {}
                }
            }
            _ = wait_for_shutdown_signal() => break,
        }
    }

    session.cleanup(&app.client.executor);
    Ok(())
}

struct TuiSession {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
}

impl TuiSession {
    fn new(terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Self {
        Self {
            terminal: Some(terminal),
        }
    }

    fn terminal_mut(&mut self) -> Option<&mut Terminal<CrosstermBackend<io::Stdout>>> {
        self.terminal.as_mut()
    }

    fn cleanup(&mut self, executor: &Executor) {
        executor.kill_all_running();
        if let Some(mut terminal) = self.terminal.take() {
            let _ = restore_terminal(&mut terminal);
        }
    }
}

impl Drop for TuiSession {
    fn drop(&mut self) {
        if let Some(mut terminal) = self.terminal.take() {
            let _ = restore_terminal(&mut terminal);
        }
    }
}
