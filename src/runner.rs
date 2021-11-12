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

use chrono::Utc;
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
    let mut crashes = Vec::new();
    while let Err(_) = crate::run_command::run_command(&command, error_path.clone(), tx.clone(), id).await {
        crashes.push(Utc::now());
        if let Some(strategy) = &command.backup_strategy {
            let mut crash_count = 0u64;
            for timestamp in &crashes {
                if timestamp > &(Utc::now() - strategy.period) {
                    crash_count += 1u64;
                }
                if crash_count > strategy.times {
                    if strategy.script.is_none() && strategy.safe_mode.is_none() {
                        // we have no handling strategy so we just give up
                        tx.try_send(TuiEvent::NewStderrMessage(id, String::from("Crash limit reached with no handling strategy, giving up!")))?;
                        return Ok(())
                    }
                    if let Some(script) = &strategy.script {
                        let script_config = CommandConfig {
                            command: script.to_owned(),
                            args: Vec::new(),
                            stdout_history: command.stdout_history,
                            mode: CommandMode::RunOnceAndWait,
                            name: script.to_owned(),
                            backup_strategy: None,
                        };
                        run_once(script_config, error_path.clone(), tx.clone(), id).await?
                    }
                    if let Some(args) = &strategy.safe_mode {
                        let script_config = CommandConfig {
                            command: command.command.to_owned(),
                            args: args.clone(),
                            stdout_history: command.stdout_history,
                            mode: CommandMode::RunOnceAndWait,
                            name: command.name.to_owned(),
                            backup_strategy: None,
                        };
                        run_once(script_config, error_path.clone(), tx.clone(), id).await?
                    }
                }
            }
        }
    }
    Ok(())
}

// keep alive, ignoring exit codes
async fn run_keep_alive(
    command: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<()> {
    let mut crashes = Vec::new();
    loop {
        if let Err(_) = crate::run_command::run_command(&command, error_path.clone(), tx.clone(), id).await {
            crashes.push(Utc::now());
            if let Some(strategy) = &command.backup_strategy {
                let mut crash_count = 0u64;
                for timestamp in &crashes {
                    if timestamp > &(Utc::now() - strategy.period) {
                        crash_count += 1u64;
                    }
                    if crash_count > strategy.times {
                        if strategy.script.is_none() && strategy.safe_mode.is_none() {
                            // we have no handling strategy so we just give up
                            tx.try_send(TuiEvent::NewStderrMessage(id, String::from("Crash limit reached with no handling strategy, giving up!")))?;
                            return Ok(())
                        }
                        if let Some(script) = &strategy.script {
                            let script_config = CommandConfig {
                                command: script.to_owned(),
                                args: Vec::new(),
                                stdout_history: command.stdout_history,
                                mode: CommandMode::RunOnceAndWait,
                                name: script.to_owned(),
                                backup_strategy: None,
                            };
                            run_once(script_config, error_path.clone(), tx.clone(), id).await?
                        }
                        if let Some(args) = &strategy.safe_mode {
                            let script_config = CommandConfig {
                                command: command.command.to_owned(),
                                args: args.clone(),
                                stdout_history: command.stdout_history,
                                mode: CommandMode::RunOnceAndWait,
                                name: command.name.to_owned(),
                                backup_strategy: None,
                            };
                            run_once(script_config, error_path.clone(), tx.clone(), id).await?
                        }
                    }
                }
            }
        }
    }
}
