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

use std::{
    io::{self, Stdout},
    time::Duration,
};

use async_std::{
    channel::{Receiver, Sender},
    future::timeout,
    task,
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

use crate::{
    runner_error::Result,
    tui_state::{Severity, TabState, TuiEvent, TuiState},
};

// terminal type to be passed around
type TerminalT = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>;

// tui thread. first to start, last to quit
pub(crate) async fn run(tx: Sender<TuiEvent>, rx: Receiver<TuiEvent>) -> Result<()> {
    task::spawn(start_key_monitoring_loop(tx));
    start_display_loop(rx)?;
    Ok(())
}

// draw on display
fn draw_screen(tui_state: &mut TuiState, terminal: &mut TerminalT) -> Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::Cyan));
        f.render_widget(block, f.size());

        let tabs = create_tabs(tui_state);
        f.render_widget(tabs, chunks[0]);

        let output = create_output(tui_state);
        f.render_widget(output, chunks[1]);
    })?;
    Ok(())
}

// monitor key presses from users
async fn start_key_monitoring_loop(tx: Sender<TuiEvent>) -> Result<()> {
    let stdin = io::stdin();
    for key in stdin.keys().flatten() {
        tx.try_send(TuiEvent::Input(key))?;
    }
    Ok(())
}

// react to the events
fn start_display_loop(rx: Receiver<TuiEvent>) -> Result<()> {
    let mut tui_state = TuiState::build(Vec::new());
    let mut terminal = Terminal::new(TermionBackend::new(AlternateScreen::from(
        MouseTerminal::from(io::stdout().into_raw_mode()?),
    )))?;
    loop {
        update_tui_state(&mut tui_state, &rx);
        draw_screen(&mut tui_state, &mut terminal)?;
    }
}

// updates tui data based on input or times out silently in 250 ms
fn update_tui_state(tui_state: &mut TuiState, rx: &Receiver<TuiEvent>) {
    if let Ok(Ok(event)) = task::block_on(timeout(Duration::from_millis(250), rx.recv())) {
        match event {
            TuiEvent::TabListChanged(titles) => {
                tui_state.tabs = titles
                    .iter()
                    .map(|title| TabState::build(title.clone()))
                    .collect()
            }
            TuiEvent::CommandStarted(idx) => {
                tui_state.tabs[idx].add_message(Severity::System, String::from("Command Started"))
            }
            TuiEvent::NewStdoutMessage(idx, message) => {
                tui_state.tabs[idx].add_message(Severity::Info, message)
            }
            TuiEvent::NewStderrMessage(idx, message) => {
                tui_state.tabs[idx].add_message(Severity::Error, message)
            }
            TuiEvent::CommandEnded(idx) => {
                tui_state.tabs[idx].add_message(Severity::System, String::from("Command ended"))
            }
            TuiEvent::Input(key) => match key {
                Key::Right => tui_state.next(),
                Key::Left => tui_state.previous(),
                _ => {}
            },
        }
    }
}

// creates tabs on top of screen
fn create_tabs(tui_state: &TuiState) -> Tabs {
    let titles = tui_state
        .tabs
        .iter()
        .map(|t| Spans::from(Span::styled(&t.title, Style::default().fg(Color::Cyan))))
        .collect();
    Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .select(tui_state.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Magenta),
        )
}

// draws output in the bottom of the screen
fn create_output(tui_state: &TuiState) -> List {
    let messages: Vec<ListItem> = tui_state.tabs[tui_state.index]
        .content
        .iter()
        .rev()
        .map(|(severity, text)| match severity {
            Severity::Info => ListItem::new(Span::styled(text, Style::default().fg(Color::White))),
            Severity::Error => {
                ListItem::new(Span::styled(text, Style::default().fg(Color::Magenta)))
            }
            Severity::System => ListItem::new(Span::styled(text, Style::default().fg(Color::Cyan))),
        })
        .collect();

    List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Output"))
        .start_corner(Corner::BottomLeft)
}
