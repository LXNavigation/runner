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

use async_std::{channel::Receiver, sync::RwLock, task};
use std::{io, sync::Arc};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Tabs},
    Terminal,
};

use crate::tui_helper::{Event, Events, Severity, TabsState};

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
}

// tui thread. first to start, last to quit
pub(crate) async fn run(rx: Receiver<TuiEvent>) {
    let tui_state = Arc::new(RwLock::new(TuiState {
        tabs: TabsState::new(Vec::new()),
    }));
    task::spawn(start_event_loop(tui_state.clone()));
    start_display_loop(rx, tui_state).await;
}

// continuously update display
async fn start_event_loop(tui_state: Arc<RwLock<TuiState>>) {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let events = Events::new();

    // Main loop
    loop {
        let mut app = tui_state.write().await;
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

        if let Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.previous(),
                _ => {}
            };
        }
    }
}

// react to the events
async fn start_display_loop(rx: Receiver<TuiEvent>, tui_state: Arc<RwLock<TuiState>>) {
    loop {
        match rx.recv().await {
            Ok(event) => {
                let mut app = tui_state.write().await;
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
                }
            }
            Err(_) => return,
        }
    }
}
