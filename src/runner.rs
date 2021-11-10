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
    task,
};
use futures::future::join_all;

use crate::{
    command_config::{CommandConfig, CommandMode},
    config::Config,
    runner_error::Result,
    tui_state::TuiEvent,
};

// main run called from main function
pub(crate) async fn run(config: String) -> Result<()> {
    // parse config file
    let config = Config::create(config)?;

    let (tx, rx) = channel::unbounded();
    let tui_handle = task::spawn(crate::tui::run(tx.clone(), rx));

    tx.try_send(TuiEvent::TabListChanged(
        config
            .commands
            .iter()
            .map(|command| command.name.clone())
            .collect(),
    ))?;

    //ensure error folder exists
    std::fs::create_dir_all(&config.crash_path)?;

    // execute all commands, saving handles
    execute_commands(config, tx).await?;

    tui_handle.await
}

// executes command based on mode
async fn execute_commands(config: Config, tx: Sender<TuiEvent>) -> Result<Vec<()>> {
    let mut futures = Vec::new();

    for (id, command) in config.commands.into_iter().enumerate() {
        match command.mode {
            CommandMode::RunOnce => futures.push(task::spawn(run_once(
                command,
                config.crash_path.clone(),
                tx.clone(),
                id,
            ))),
            CommandMode::RunOnceAndWait => {
                run_once(command, config.crash_path.clone(), tx.clone(), id).await?
            }
            CommandMode::RunUntilSuccess => futures.push(task::spawn(run_until_success(
                command,
                config.crash_path.clone(),
                tx.clone(),
                id,
            ))),
            CommandMode::RunUntilSuccessAndWait => {
                run_until_success(command, config.crash_path.clone(), tx.clone(), id).await?
            }
            CommandMode::KeepAlive => futures.push(task::spawn(run_keep_alive(
                command,
                config.crash_path.clone(),
                tx.clone(),
                id,
            ))),
        };
    }
    join_all(futures)
        .await
        .into_iter()
        .collect::<Result<Vec<()>>>()
}

// run once
async fn run_once(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<()> {
    crate::run_command::run_command(&command, error_path, tx, id).await
}

// run until success (exit code 0)
async fn run_until_success(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<()> {
    while crate::run_command::run_command(&command, error_path.clone(), tx.clone(), id)
        .await
        .is_err()
    {}
    Ok(())
}

// keep alive, ignoring exit codes
async fn run_keep_alive(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<()> {
    loop {
        let _ = crate::run_command::run_command(&command, error_path.clone(), tx.clone(), id).await;
    }
}
