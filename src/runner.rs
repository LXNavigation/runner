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
    channel::{self, Sender},
    task::{self, JoinHandle},
};

use crate::{
    command_config::{CommandConfig, CommandMode},
    config::Config,
    tui_state::TuiEvent,
};

// main run called from main function
pub fn run(config: String) {
    // parse config file
    let config = match Config::create(config) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing config file: {:?}", err);
            return;
        }
    };

    let (tx, rx) = channel::unbounded();
    let tui_handle = task::spawn(crate::tui::run(tx.clone(), rx));

    tx.try_send(TuiEvent::TabListChanged(
        config
            .commands
            .iter()
            .map(|command| command.name.clone())
            .collect(),
    ))
    .expect("unbound channel should never be full");

    //ensure error folder exists
    std::fs::create_dir_all(&config.crash_path).expect("Could not create crash path, aborting...");

    // execute all commands, saving handles
    config
        .commands
        .into_iter()
        .enumerate()
        .map(|(id, command)| execute_command(command, config.crash_path.clone(), tx.clone(), id))
        .flatten()
        .for_each(task::block_on);

    task::block_on(tui_handle);
}

// executes command based on mode
fn execute_command(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Option<JoinHandle<()>> {
    match command.mode {
        CommandMode::RunOnce => return Some(run_once(command, error_path, tx, id)),
        CommandMode::RunOnceAndWait => run_once_and_wait(command, error_path, tx, id),
        CommandMode::RunUntilSuccess => {
            return Some(run_until_success(command, error_path, tx, id))
        }
        CommandMode::RunUntilSuccessAndWait => {
            run_until_success_and_wait(command, error_path, tx, id)
        }
        CommandMode::KeepAlive => return Some(run_keep_alive(command, error_path, tx, id)),
    };
    None
}

// run once
fn run_once(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> JoinHandle<()> {
    task::spawn(async move {
        let _ = crate::run_command::run_command(command, error_path, tx, id).await;
    })
}

// run once and wait before moving to next command
fn run_once_and_wait(command: CommandConfig, error_path: String, tx: Sender<TuiEvent>, id: usize) {
    task::block_on(async move {
        let _ = crate::run_command::run_command(command, error_path, tx, id).await;
    });
}

// run until success (exit code 0)
fn run_until_success(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> JoinHandle<()> {
    task::spawn(async move {
        while crate::run_command::run_command(command.clone(), error_path.clone(), tx.clone(), id)
            .await
            .is_err()
        {}
    })
}

// run until success (exit code 0) and wait before moving to next command
fn run_until_success_and_wait(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) {
    task::block_on(async move {
        while crate::run_command::run_command(command.clone(), error_path.clone(), tx.clone(), id)
            .await
            .is_err()
        {}
    });
}

// keep alive, ignoring exit codes
fn run_keep_alive(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            let _ = crate::run_command::run_command(
                command.clone(),
                error_path.clone(),
                tx.clone(),
                id,
            )
            .await;
        }
    })
}
