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

use async_std::{channel::Receiver, task};

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
    task::spawn(start_event_loop());
    start_display_loop(rx).await;
}

// continuously update display
async fn start_event_loop() {}

// react to the events
async fn start_display_loop(rx: Receiver<TuiEvent>) {
    loop {
        match rx.recv().await {
            Ok(event) => println!("{:?}", event),
            Err(_) => return,
        }
    }
}
