//! Ratatui-based interactive terminal UI.
//!
//! The UI is divided into three vertical panels:
//!
//! ```text
//! ┌─ Modules ───────────────┬─ Commands ──────────────┬─ Running ─────────┐
//! │                          │ [profile] command        │ #id profile/cmd   │
//! │                          │ ...                      │ status            │
//! │                          │                          ├───────────────────┤
//! │                          │                          │ output lines      │
//! └──────────────────────────┴──────────────────────────┴───────────────────┘
//! │ help bar                                                              │
//! └───────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key bindings
//!
//! | Key        | Action                                      |
//! |------------|---------------------------------------------|
//! | `Tab`      | Move focus to the next panel                |
//! | `Shift+Tab`| Move focus to the previous panel            |
//! | `↑` / `↓`  | Navigate the focused list                   |
//! | `Enter`    | Select module (Modules) / run command       |
//! | `k`        | Kill selected process (Running panel)       |
//! | `r`        | Re-scan modules                             |
//! | `q` / `^C` | Quit                                        |
use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, sync::Arc, time::Duration};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;

use crate::{
    client::MoldXClient,
    command::Command,
    errors::MoldXError2,
    executor::{self, Executor},
    module::Module,
};

/// Session holding the terminal and the shared executor.
///
/// Ensures that running processes are killed and the terminal is restored
/// when the session is dropped.
struct TuiSession {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    executor: Arc<Executor>,
}

impl TuiSession {
    fn new(terminal: Terminal<CrosstermBackend<io::Stdout>>, executor: Arc<Executor>) -> Self {
        Self {
            terminal: Some(terminal),
            executor,
        }
    }

    fn terminal_mut(&mut self) -> Option<&mut Terminal<CrosstermBackend<io::Stdout>>> {
        self.terminal.as_mut()
    }

    fn cleanup(&mut self) {
        self.executor.kill_all_running();
        if let Some(mut terminal) = self.terminal.take() {
            let _ = restore_terminal(&mut terminal);
        }
    }
}

impl Drop for TuiSession {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// The focusable panels of the UI.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Panel {
    Modules,
    Commands,
    Running,
}

impl Panel {
    fn next(self) -> Panel {
        match self {
            Panel::Modules => Panel::Commands,
            Panel::Commands => Panel::Running,
            Panel::Running => Panel::Modules,
        }
    }
    fn prev(self) -> Panel {
        match self {
            Panel::Modules => Panel::Running,
            Panel::Commands => Panel::Modules,
            Panel::Running => Panel::Commands,
        }
    }
}

/// State of the interactive UI.
///
/// Tracks the focused panel, the cursor position of each list, the commands
/// available for the selected module, and the asynchronous module refresh.
struct TuiApp {
    client: Arc<MoldXClient>,
    modules: Vec<Module>,
    active_panel: Panel,
    module_idx: usize,
    command_idx: usize,
    running_idx: usize,
    command_items: Vec<CommandItem>,
    is_refreshing: bool,
    refresh_rx: Option<oneshot::Receiver<Result<Vec<Module>>>>,
    output_scroll: usize,
    log: Vec<String>,
}

/// A command entry shown in the Commands panel.
#[derive(Debug, Clone)]
struct CommandItem {
    profile: String,
    command: Command,
}

impl TuiApp {
    fn new(client: Arc<MoldXClient>) -> Self {
        let mut app = TuiApp {
            modules: client.modules.clone(),
            active_panel: Panel::Modules,
            module_idx: 0,
            command_idx: 0,
            running_idx: 0,
            command_items: Vec::new(),
            is_refreshing: false,
            refresh_rx: None,
            output_scroll: 0,
            log: Vec::new(),
            client,
        };
        app.rebuild_command_items();
        app
    }

    fn rebuild_command_items(&mut self) {
        self.command_items.clear();
        self.command_idx = 0;
        if let Some(m) = self.modules.get(self.module_idx) {
            let mut resolved: Vec<(&str, &Command)> = Vec::new();
            for &profile_index in &m.profiles {
                if let Some(profile) = self.client.profile_children().get(profile_index) {
                    for command in &profile.commands {
                        resolved.push((&profile.name, command));
                    }
                }
            }
            resolved.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.name.cmp(&b.1.name)));
            for (profile_name, command) in resolved {
                self.command_items.push(CommandItem {
                    profile: profile_name.to_string(),
                    command: command.clone(),
                });
            }
        }
    }

    fn selected_module(&self) -> Option<&Module> {
        self.modules.get(self.module_idx)
    }

    fn trigger_refresh(&mut self) {
        if self.is_refreshing {
            return;
        }
        self.is_refreshing = true;
        let (tx, rx) = oneshot::channel();
        self.refresh_rx = Some(rx);
        let client = self.client.clone();
        tokio::spawn(async move {
            let result = client.resolve_modules();
            let _ = tx.send(result);
        });
    }

    fn tick(&mut self) {
        if let Some(rx) = self.refresh_rx.as_mut() {
            match rx.try_recv() {
                Ok(Ok(modules)) => {
                    self.modules = modules;
                    if self.module_idx >= self.modules.len() {
                        self.module_idx = self.modules.len().saturating_sub(1);
                    }
                    self.rebuild_command_items();
                    self.log.push("Modules refreshed".to_string());
                }
                Ok(Err(e)) => {
                    self.log.push(format!("Refresh error: {}", e));
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    return;
                }
                Err(_) => {}
            }
            self.refresh_rx = None;
            self.is_refreshing = false;
        }
    }

    fn run_selected_command(&mut self) {
        let module = match self.selected_module() {
            Some(m) => m.path.clone(),
            None => return,
        };
        let item = match self.command_items.get(self.command_idx) {
            Some(item) => item.clone(),
            None => return,
        };
        let profile = item.profile.clone();
        let command = item.command.name.clone();
        let script = item.command.path.clone();

        let id = self.client.executor.add_process(
            &module.to_string_lossy(),
            &profile,
            &command,
            None,
        );

        self.log.push(format!(
            "Spawned #{}: {}/{} on {}",
            id,
            profile,
            command,
            module.display()
        ));

        let executor = Arc::new(self.client.executor.clone());
        tokio::spawn(executor::run_and_track(executor, id, script, module));
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }
        if key.code == KeyCode::Char('q') && self.active_panel == Panel::Modules {
            return true;
        }

        match key.code {
            KeyCode::Tab => self.active_panel = self.active_panel.next(),
            KeyCode::BackTab => self.active_panel = self.active_panel.prev(),
            KeyCode::Char('r') => self.trigger_refresh(),
            KeyCode::Up => self.move_cursor(-1),
            KeyCode::Char('k') if self.active_panel != Panel::Running => {}
            KeyCode::Down => self.move_cursor(1),
            KeyCode::Enter => match self.active_panel {
                Panel::Modules => {
                    self.rebuild_command_items();
                    self.active_panel = Panel::Commands;
                }
                Panel::Commands => self.run_selected_command(),
                Panel::Running => {}
            },
            KeyCode::Char('k') => {
                let summaries = self.client.executor.get_summaries();
                if let Some(s) = summaries.get(self.running_idx) {
                    let id = s.id;
                    self.client.executor.kill_process(id);
                    self.log.push(format!("Killed process #{}", id));
                }
            }
            _ => {}
        }
        false
    }

    fn move_cursor(&mut self, delta: i32) {
        match self.active_panel {
            Panel::Modules => {
                let len = self.modules.len();
                if len == 0 {
                    return;
                }
                if delta < 0 {
                    if self.module_idx == 0 {
                        self.module_idx = len - 1;
                    } else {
                        self.module_idx -= 1;
                    }
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
                    if self.command_idx == 0 {
                        self.command_idx = len - 1;
                    } else {
                        self.command_idx -= 1;
                    }
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
                    if self.running_idx == 0 {
                        self.running_idx = len - 1;
                    } else {
                        self.running_idx -= 1;
                    }
                } else {
                    self.running_idx = (self.running_idx + 1) % len;
                }
            }
        }
    }
}

// ─── Terminal helpers ────────────────────────────────────────────────────────

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

// ─── Drawing ─────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let area = frame.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(42),
            Constraint::Percentage(30),
        ])
        .split(vertical[0]);

    draw_modules(frame, app, columns[0]);
    draw_commands(frame, app, columns[1]);
    draw_running(frame, app, columns[2]);
    draw_help(frame, vertical[1]);
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

fn draw_modules(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let active = app.active_panel == Panel::Modules;
    let title = if app.is_refreshing {
        "Modules (scanning\u{2026})"
    } else {
        "Modules"
    };
    let block = panel_block(title, active);

    let mut content_area = area;
    if area.height >= 6 {
        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        let purple = Color::Rgb(0x8b, 0x00, 0xff);
        let lime = Color::Rgb(0x74, 0xff, 0x00);

        let raw_lines = [
            "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}",
            "\u{2588}\u{2584}\u{2500}\u{2580}\u{2580}\u{2500}\u{2584}\u{2588}\u{2500}\u{2584}\u{2584}\u{2500}\u{2588}\u{2584}\u{2500}\u{2584}\u{2588}\u{2588}\u{2588}\u{2584}\u{2500}\u{2584}\u{2584}\u{2580}\u{2588}\u{2584}\u{2500}\u{2580}\u{2500}\u{2584}\u{2588}",
            "\u{2588}\u{2588}\u{2500}\u{2588}\u{2584}\u{2588}\u{2500}\u{2588}\u{2588}\u{2500}\u{2588}\u{2588}\u{2500}\u{2588}\u{2588}\u{2500}\u{2588}\u{2580}\u{2588}\u{2588}\u{2500}\u{2588}\u{2588}\u{2500}\u{2588}\u{2588}\u{2500}\u{2580}\u{2500}\u{2588}\u{2588}",
            "\u{2580}\u{2584}\u{2584}\u{2584}\u{2580}\u{2584}\u{2584}\u{2584}\u{2580}\u{2584}\u{2584}\u{2584}\u{2584}\u{2580}\u{2584}\u{2584}\u{2584}\u{2584}\u{2580}\u{2584}\u{2584}\u{2584}\u{2584}\u{2580}\u{2580}\u{2584}\u{2584}\u{2580}\u{2588}\u{2584}\u{2584}\u{2580}",
        ];

        let mut styled_lines: Vec<Line> = Vec::new();
        for (i, line) in raw_lines.iter().enumerate() {
            let color = if i % 2 == 0 { purple } else { lime };
            styled_lines.push(Line::from(Span::styled(
                *line,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
        }
        styled_lines.push(Line::from(Span::raw(" ")));

        let p = Paragraph::new(styled_lines)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(p, parts[0]);
        content_area = parts[1];
    }

    if app.modules.is_empty() {
        let msg = if app.is_refreshing {
            "Scanning\u{2026}"
        } else {
            "No modules found.\nPress r to scan."
        };
        let p = Paragraph::new(msg)
            .block(block)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, content_area);
        return;
    }

    let items: Vec<ListItem> = app
        .modules
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let name = m.path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let rel = m.path.to_string_lossy();
            let profile_list = {
                let mut names: Vec<&str> = m
                    .profiles
                    .iter()
                    .filter_map(|&idx| {
                        app.client
                            .profile_children()
                            .get(idx)
                            .map(|p| p.name.as_str())
                    })
                    .collect();
                names.sort();
                names.join(", ")
            };
            let style = if i == app.module_idx && active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.module_idx {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(vec![
                Line::from(Span::styled(name, style)),
                Line::from(Span::styled(
                    format!("  {}", rel),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    format!("  [{}]", profile_list),
                    Style::default().fg(Color::Blue),
                )),
            ])
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.module_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{25b6} ");

    frame.render_stateful_widget(list, content_area, &mut list_state);
}

fn draw_commands(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let active = app.active_panel == Panel::Commands;
    let module_name = app
        .selected_module()
        .and_then(|m| m.path.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("\u{2014}")
        .to_string();
    let commands_title = format!("Commands  {}", module_name);
    let block = panel_block(&commands_title, active);

    if app.command_items.is_empty() {
        let msg = if app.modules.is_empty() {
            "No modules detected"
        } else {
            "No commands for this module"
        };
        let p = Paragraph::new(msg)
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
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.command_idx {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", item.profile),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(item.command.name.as_str(), style),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.command_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("\u{25b6} ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_running(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let active = app.active_panel == Panel::Running;
    let summaries = app.client.executor.get_summaries();

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let running_title = format!("Running  ({})", summaries.len());
    let list_block = panel_block(&running_title, active);

    if summaries.is_empty() {
        let p = Paragraph::new("No processes yet.\nRun a command with Enter.")
            .block(list_block)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, inner[0]);
    } else {
        let items: Vec<ListItem> = summaries
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let status_color = if p.status.is_running() {
                    Color::Green
                } else {
                    Color::DarkGray
                };
                let sel = i == app.running_idx && active;
                let label_style = if sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!("#{} ", p.id), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}/{}", p.profile, p.command), label_style),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!("   {}", p.status.label()),
                            Style::default().fg(status_color),
                        ),
                        p.pid
                            .map(|pid| {
                                Span::styled(
                                    format!("  PID {}", pid),
                                    Style::default().fg(Color::DarkGray),
                                )
                            })
                            .unwrap_or_else(|| Span::raw("")),
                    ]),
                ])
            })
            .collect();

        let mut list_state = ListState::default();
        if !summaries.is_empty() {
            list_state.select(Some(app.running_idx.min(summaries.len() - 1)));
        }

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("\u{25b6} ");

        frame.render_stateful_widget(list, inner[0], &mut list_state);
    }

    let output_block = Block::default()
        .title(Span::styled(
            " Output ",
            Style::default().fg(Color::DarkGray),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let selected_output = summaries
        .get(app.running_idx)
        .map(|s| app.client.executor.get_output(s.id))
        .unwrap_or_default();

    let inner_height = inner[1].height.saturating_sub(2) as usize;
    let start = selected_output
        .len()
        .saturating_sub(inner_height + app.output_scroll);
    let output_lines: Vec<Line> = selected_output
        .iter()
        .skip(start)
        .take(inner_height)
        .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(Color::Gray))))
        .collect();

    let output_para = Paragraph::new(output_lines)
        .block(output_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(output_para, inner[1]);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(Color::Cyan)),
        Span::raw(":panel  "),
        Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Cyan)),
        Span::raw(":nav  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(":run/select  "),
        Span::styled("k", Style::default().fg(Color::Cyan)),
        Span::raw(":kill  "),
        Span::styled("r", Style::default().fg(Color::Cyan)),
        Span::raw(":refresh  "),
        Span::styled("q/^C", Style::default().fg(Color::Cyan)),
        Span::raw(":quit "),
    ]))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(help, area);
}

// ─── Entry point ─────────────────────────────────────────────────────────────

/// Runs the interactive terminal UI.
///
/// Installs a panic hook that restores the terminal, switches to the
/// alternate screen, and enters the event loop drawing the three panels.
/// The UI runs until the user quits or a shutdown signal is received.
///
/// # Arguments
///
/// * `client` - The client whose profiles and modules populate the UI.
///
/// # Returns
///
/// Ok when the UI exits normally.
///
/// # Errors
///
/// Returns an error if the terminal cannot be set up, a frame cannot be
/// drawn, or the terminal is unavailable during the run.
pub async fn run(client: &MoldXClient) -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let client = Arc::new(MoldXClient {
        profiles: client.profiles.clone(),
        modules: client.modules.clone(),
        config: client.config.clone(),
        executor: Executor::new(),
    });

    eprintln!("Scanning modules (this may take a moment)\u{2026}");
    eprintln!("Found {} module(s).", client.modules.len());

    let executor = Arc::new(client.executor.clone());
    let mut session = TuiSession::new(setup_terminal()?, executor);
    let mut app = TuiApp::new(client);

    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(500));

    loop {
        {
            let terminal = session
                .terminal_mut()
                .ok_or(MoldXError2::TerminalUnavailable)?;
            terminal.draw(|f| draw(f, &mut app))?;
        }

        tokio::select! {
            _ = tick.tick() => {
                app.tick();
            }
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn test_client() -> Arc<MoldXClient> {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        let profile_dir = moldx_dir.join("profiles/docker");
        fs::create_dir_all(profile_dir.join("template")).unwrap();
        fs::create_dir_all(profile_dir.join("bin")).unwrap();
        fs::write(profile_dir.join("template/Dockerfile"), "").unwrap();
        fs::write(
            profile_dir.join("bin/build.sh"),
            "#!/usr/bin/env bash\nexit 0\n",
        )
        .unwrap();

        let module_dir = dir.path().join("service");
        fs::create_dir_all(&module_dir).unwrap();
        fs::write(module_dir.join("Dockerfile"), "").unwrap();

        let config = crate::config::MoldXConfig {
            moldx_dir,
            profiles_dir: dir.path().join(".moldx/profiles"),
            profiles_dir_name: "profiles".into(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: dir.path().to_path_buf(),
            max_resolution_depth: 20,
        };
        Arc::new(MoldXClient::new(config).unwrap())
    }

    #[test]
    fn panel_navigation_wraps() {
        assert_eq!(Panel::Modules.next(), Panel::Commands);
        assert_eq!(Panel::Commands.next(), Panel::Running);
        assert_eq!(Panel::Running.next(), Panel::Modules);
        assert_eq!(Panel::Modules.prev(), Panel::Running);
        assert_eq!(Panel::Running.prev(), Panel::Commands);
    }

    #[test]
    fn app_rebuilds_commands_and_handles_navigation() {
        let client = test_client();
        let mut app = TuiApp::new(client);
        assert_eq!(app.modules.len(), 1);
        assert_eq!(app.command_items.len(), 1);
        assert_eq!(app.command_items[0].profile, "docker");

        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_panel, Panel::Commands);
        app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(app.active_panel, Panel::Modules);
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.module_idx, 0);
        assert!(!app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)));
        assert!(app.handle_key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )));
    }

    #[test]
    fn app_renders_with_test_backend() {
        let client = test_client();
        let mut app = TuiApp::new(client);
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        assert!(buffer.content().iter().any(|cell| cell.symbol() == "M"));
    }
}
