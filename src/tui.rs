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

use std::io::Stdout;
use async_std::{channel::{Receiver, Sender}, sync::RwLock, task};
use std::{io, sync::Arc};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::{IntoRawMode, RawTerminal}, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Tabs},
    Terminal,
};

async fn start_key_monitoring_loop(tx: Sender<TuiEvent>) {
    let stdin = io::stdin();
    for key in stdin.keys().flatten() {
        tx.try_send(TuiEvent::Input(key))
            .expect("unbound channels should never be full");
    }
}

#[derive(Clone)]
pub(crate) enum Severity {
    Info,
    Error,
}

pub(crate) struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
    pub content: Vec<Vec<(Severity, String)>>,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState {
            titles,
            index: 0,
            content: Vec::new(),
        }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}


struct TuiState {
    tabs: TabsState,
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
    Input(Key)
}

type TerminalT = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>;

// tui thread. first to start, last to quit
pub(crate) async fn run(tx: Sender<TuiEvent>, rx: Receiver<TuiEvent>) {
    eprintln!("run started");
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    let tui_state = Arc::new(RwLock::new(TuiState {
        tabs: TabsState::new(Vec::new()),
    }));
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

                let block =
                    Block::default().style(Style::default().bg(Color::Black).fg(Color::Cyan));
                f.render_widget(block, size);
                let titles = app
                    .tabs
                    .titles
                    .iter()
                    .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Cyan))]))
                    .collect();
                let tabs = Tabs::new(titles)
                    .block(Block::default().borders(Borders::ALL).title("Commands"))
                    .select(app.tabs.index)
                    .style(Style::default().fg(Color::Cyan))
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Magenta),
                    );
                f.render_widget(tabs, chunks[0]);

                let messages: Vec<ListItem> = app.tabs.content[app.tabs.index]
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
                    })
                    .collect();

                let outputs_list = List::new(messages)
                    .block(Block::default().borders(Borders::ALL).title("Output"))
                    .start_corner(Corner::BottomLeft);
                f.render_widget(outputs_list, chunks[1]);
            })
            .unwrap();

}

// react to the events
fn start_display_loop(rx: Receiver<TuiEvent>, tui_state: Arc<RwLock<TuiState>>, mut terminal: TerminalT) {
    loop {
        match task::block_on(rx.recv()) {
            Ok(event) => {
                let mut app = task::block_on(tui_state.write());
                match event {
                    TuiEvent::TabListChanged(titles) => {
                        app.tabs.content.clear();
                        app.tabs.content.resize(titles.len(), Vec::new());
                        app.tabs.titles = titles;
                    }
                    TuiEvent::CommandStarted(_) => {}
                    TuiEvent::NewStdoutMessage(idx, message) => {
                        app.tabs.content[idx].push((Severity::Info, message));
                    }
                    TuiEvent::NewStderrMessage(idx, message) => {
                        app.tabs.content[idx].push((Severity::Error, message));
                    }
                    TuiEvent::CommandEnded(_) => {}
                    TuiEvent::Input(key) => {
                        match key {
                            Key::Right => app.tabs.next(),
                            Key::Left => app.tabs.previous(),
                            _ => {}
                        };
                    }
                }
            }
            Err(_) => return,
        }
        draw_screen(tui_state.clone(), &mut terminal)
    }
}
