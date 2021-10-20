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

use async_std::{channel::{Receiver, Sender}, future::timeout, sync::RwLock, task};
use std::{io, sync::Arc, time::Duration};
use termion::{
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
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

#[derive(Clone)]
pub(crate) enum Severity {
    Info,
    Error,
    System,
}

struct TabState {
    pub title: String,
    pub content: Vec<(Severity, String)>,
}

impl TabState {
    fn build(title: String) -> TabState {
        TabState {
            title,
            content: Vec::new()
        }
    }
}

struct TuiState {
    tabs: Vec<TabState>,
    index: usize,
}

impl TuiState {
    fn build(titles: Vec<String>) -> TuiState {
        TuiState {
            tabs: titles.into_iter().map(|title| { TabState::build(title) }).collect(),
            index: 0,
        }
    }
    fn next(&mut self) {
        self.index = (self.index + 1) % self.tabs.len();
    }

    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }
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
}

// tui thread. first to start, last to quit
pub(crate) async fn run(tx: Sender<TuiEvent>, rx: Receiver<TuiEvent>) {
    let tui_state = Arc::new(RwLock::new(TuiState::build(Vec::new())));
    task::spawn(start_key_monitoring_loop(tx));
    task::spawn(start_event_loop(tui_state.clone()));
    start_display_loop(rx, tui_state).await;
}

async fn start_key_monitoring_loop(tx: Sender<TuiEvent>) {
    let stdin = io::stdin();
    for key in stdin.keys().flatten() {
        tx.try_send(TuiEvent::Input(key))
            .expect("unbound channels should never be full");
    }
}

// continuously update display
async fn start_event_loop(tui_state: Arc<RwLock<TuiState>>) {
    // Terminal initialization
    let mut terminal = Terminal::new(TermionBackend::new(AlternateScreen::from(
        MouseTerminal::from(io::stdout().into_raw_mode().expect("could not get stdout in raw mode")),
    )))
    .expect("failed to initiate terminal from backend");

    // Main loop
    loop {
        let app = tui_state.read().await;
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(size);

                let block =
                    Block::default().style(Style::default().bg(Color::Black).fg(Color::Cyan));
                f.render_widget(block, size);
                let titles = app
                    .tabs.iter()
                    .map(|t| Spans::from(vec![Span::styled(&t.title, Style::default().fg(Color::Cyan))]))
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

                let messages: Vec<ListItem> = app.tabs[app.index].content
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
}

// react to the events
async fn start_display_loop(rx: Receiver<TuiEvent>, tui_state: Arc<RwLock<TuiState>>) {
    loop {
        match timeout(Duration::from_millis(250), rx.recv()).await {
            Ok(Ok(event)) => {
                let mut app = tui_state.write().await;
                match event {
                    TuiEvent::TabListChanged(titles) => {
                        app.tabs = titles.iter().map(|title| TabState::build(title.clone())).collect();
                    }
                    TuiEvent::CommandStarted(idx) => {
                        app.tabs[idx].content
                            .push((Severity::System, String::from("Command Started")));
                    }
                    TuiEvent::NewStdoutMessage(idx, message) => {
                        app.tabs[idx].content.push((Severity::Info, message));
                    }
                    TuiEvent::NewStderrMessage(idx, message) => {
                        app.tabs[idx].content.push((Severity::Error, message));
                    }
                    TuiEvent::CommandEnded(idx) => {
                        app.tabs[idx].content
                            .push((Severity::System, String::from("Command ended")));
                    }
                    TuiEvent::Input(key) => {
                        match key {
                            Key::Right => app.next(),
                            Key::Left => app.previous(),
                            _ => {}
                        };
                    }
                }
            }
            Ok(Err(_)) => return,
            Err(_) => {}
        }
    }
}
