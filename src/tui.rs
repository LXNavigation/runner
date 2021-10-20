/*
This file is part of the Everdream Runner (https://gitlab.com/everdream/runner).
Copyright (c) 2021 Everdream.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

use async_std::{
    channel::{Receiver, Sender},
    future::timeout,
    sync::RwLock,
    task,
};
use std::{
    collections::VecDeque,
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};
use termion::{
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Tabs},
    Terminal,
};

// terminal type to be passed around
type TerminalT = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>;

// severity of messages for display purposes
#[derive(Clone, Debug)]
pub(crate) enum Severity {
    // information, messages from stdout
    Info,

    // error messages from stderr
    Error,

    // runner generated messages
    System,
}

// All possible events that should have tui react to
#[derive(Debug)]
pub(crate) enum TuiEvent {
    // tab list for interface (1 command / tab)
    TabListChanged(Vec<String>),

    // command at given id started
    CommandStarted(usize),

    // new stdout at given id
    NewStdoutMessage(usize, String),

    // new stderr at given id
    NewStderrMessage(usize, String),

    // command with the given id ended
    CommandEnded(usize),

    // user pressed a key
    Input(Key),

    // runner messages connected to command
    System(usize, String),
}

// Entire state of Tui
struct TuiState {
    // tabs, one for each command
    tabs: Vec<TabState>,

    // currently active tab
    index: usize,
}

impl TuiState {
    // create entire state from list of commands
    fn build(titles: Vec<String>) -> TuiState {
        TuiState {
            tabs: titles.into_iter().map(TabState::build).collect(),
            index: 0,
        }
    }

    // switch to next tab
    fn next(&mut self) {
        self.index = (self.index + 1) % self.tabs.len();
    }

    // switch to previous tab
    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }
}

// Info about single tab
struct TabState {
    // tab title
    pub title: String,

    // messages to display in tab
    pub content: VecDeque<(Severity, String)>,
}

impl TabState {
    // build tab from title
    fn build(title: String) -> TabState {
        TabState {
            title,
            content: VecDeque::new(),
        }
    }

    // adds message to the list, cleans old ones
    fn add_message(&mut self, severity: Severity, text: String) {
        self.content.push_back((severity, text));
        while self.content.len() > 100 {
            self.content.pop_front();
        }
    }
}

// tui thread. first to start, last to quit
pub(crate) async fn run(tx: Sender<TuiEvent>, rx: Receiver<TuiEvent>) {
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    let tui_state = Arc::new(RwLock::new(TuiState::build(Vec::new())));
    task::spawn(start_key_monitoring_loop(tx));
    start_display_loop(rx, tui_state, terminal);
    eprintln!("run ended");
}

// draw on display
fn draw_screen(tui_state: Arc<RwLock<TuiState>>, terminal: &mut TerminalT) {
    let app = task::block_on(tui_state.read());
    terminal
        .draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::Cyan));
            f.render_widget(block, size);
            let titles = app
                .tabs
                .iter()
                .map(|t| {
                    Spans::from(vec![Span::styled(
                        &t.title,
                        Style::default().fg(Color::Cyan),
                    )])
                })
                .collect();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Commands"))
                .select(app.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Magenta),
                );
            f.render_widget(tabs, chunks[0]);

            let messages: Vec<ListItem> = app.tabs[app.index]
                .content
                .iter()
                .rev()
                .map(|(severity, text)| match severity {
                    Severity::Info => ListItem::new(Spans::from(vec![Span::styled(
                        text,
                        Style::default().fg(Color::White),
                    )])),
                    Severity::Error => ListItem::new(Spans::from(vec![Span::styled(
                        text,
                        Style::default().fg(Color::Magenta),
                    )])),
                    Severity::System => ListItem::new(Spans::from(vec![Span::styled(
                        text,
                        Style::default().fg(Color::Cyan),
                    )])),
                })
                .collect();

            let outputs_list = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title("Output"))
                .start_corner(Corner::BottomLeft);
            f.render_widget(outputs_list, chunks[1]);
        })
        .unwrap();
}

// monitor key presses from users
async fn start_key_monitoring_loop(tx: Sender<TuiEvent>) {
    let stdin = io::stdin();
    for key in stdin.keys().flatten() {
        tx.try_send(TuiEvent::Input(key))
            .expect("unbound channels should never be full");
    }
}

// react to the events
fn start_display_loop(
    rx: Receiver<TuiEvent>,
    tui_state: Arc<RwLock<TuiState>>,
    mut terminal: TerminalT,
) {
    loop {
        match task::block_on(timeout(Duration::from_millis(250), rx.recv())) {
            Ok(Ok(event)) => {
                let mut app = task::block_on(tui_state.write());
                match event {
                    TuiEvent::TabListChanged(titles) => {
                        app.tabs = titles
                            .iter()
                            .map(|title| TabState::build(title.clone()))
                            .collect();
                    }
                    TuiEvent::CommandStarted(idx) => {
                        app.tabs[idx]
                            .add_message(Severity::System, String::from("Command Started"));
                    }
                    TuiEvent::NewStdoutMessage(idx, message) => {
                        app.tabs[idx].add_message(Severity::Info, message);
                    }
                    TuiEvent::NewStderrMessage(idx, message) => {
                        app.tabs[idx].add_message(Severity::Error, message);
                    }
                    TuiEvent::CommandEnded(idx) => {
                        app.tabs[idx].add_message(Severity::System, String::from("Command ended"));
                    }
                    TuiEvent::Input(key) => {
                        match key {
                            Key::Right => app.next(),
                            Key::Left => app.previous(),
                            _ => {}
                        };
                    }
                    TuiEvent::System(idx, message) => {
                        app.tabs[idx].add_message(Severity::System, message);
                    }
                }
            }
            Ok(Err(_)) => return,
            Err(_) => {}
        }
        draw_screen(tui_state.clone(), &mut terminal)
    }
}
