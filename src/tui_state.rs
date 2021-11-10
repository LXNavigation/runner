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

use std::collections::VecDeque;

use termion::event::Key;

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
}

// Entire state of Tui
pub(crate) struct TuiState {
    // tabs, one for each command
    pub(crate) tabs: Vec<TabState>,

    // currently active tab
    pub(crate) index: usize,
}

impl TuiState {
    // create entire state from list of commands
    pub(crate) fn build(titles: Vec<String>) -> TuiState {
        TuiState {
            tabs: titles.into_iter().map(TabState::build).collect(),
            index: 0,
        }
    }

    // switch to next tab
    pub(crate) fn next(&mut self) {
        self.index = (self.index + 1) % self.tabs.len();
    }

    // switch to previous tab
    pub(crate) fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }
}

// Info about single tab
pub(crate) struct TabState {
    // tab title
    pub(crate) title: String,

    // messages to display in tab
    pub(crate) content: VecDeque<(Severity, String)>,
}

impl TabState {
    // build tab from title
    pub(crate) fn build(title: String) -> TabState {
        TabState {
            title,
            content: VecDeque::new(),
        }
    }

    // adds message to the list, cleans old ones
    pub(crate) fn add_message(&mut self, severity: Severity, text: String) {
        self.content.push_back((severity, text));
        while self.content.len() > 100 {
            self.content.pop_front();
        }
    }
}
