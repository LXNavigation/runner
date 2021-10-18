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

use crate::{
    app_config::{AppConfig, AppMode},
    config::Config,
    tui::TuiEvent,
};

use async_std::{
    channel::{self, Sender},
    task::{self, JoinHandle},
};
use futures::future::join_all;

// main run called from main function
pub fn run(config: String) {
    let (tx, rx) = channel::unbounded();
    let tui_handle = task::spawn(crate::tui::run(rx));

    // parse config file
    let config = match Config::create(config) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing config file: {:?}", err);
            return;
        }
    };

    tx.try_send(TuiEvent::TabListChanged(
        config.apps.iter().map(|app| app.get_name()).collect(),
    ))
    .expect("unbound channel should never be full");

    //ensure error folder exists
    std::fs::create_dir_all(&config.crash_path).expect("Could not create crash path, aborting...");

    // execute all commands, saving handles
    let mut handles = Vec::new();
    for (id, app) in config.apps.into_iter().enumerate() {
        if let Some(handle) = execute_app(app, config.crash_path.clone(), tx.clone(), id) {
            handles.push(handle);
        }
    }

    // wait for all commands to finish
    task::block_on(join_all(handles));
    task::block_on(tui_handle);
}

// executes app based on mode
fn execute_app(
    app: AppConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Option<JoinHandle<()>> {
    match app.mode {
        AppMode::RunOnce => return Some(run_once(app, error_path, tx, id)),
        AppMode::RunOnceAndWait => run_once_and_wait(app, error_path, tx, id),
        AppMode::RunUntilSuccess => return Some(run_until_success(app, error_path, tx, id)),
        AppMode::RunUntilSuccessAndWait => run_until_success_and_wait(app, error_path, tx, id),
        AppMode::KeepAlive => return Some(run_keep_alive(app, error_path, tx, id)),
    };
    None
}

// run once
fn run_once(app: AppConfig, error_path: String, tx: Sender<TuiEvent>, id: usize) -> JoinHandle<()> {
    task::spawn(async move {
        let _ = crate::run_command::run_command(app, error_path, tx, id).await;
    })
}

// run once and wait before moving to next command
fn run_once_and_wait(app: AppConfig, error_path: String, tx: Sender<TuiEvent>, id: usize) {
    task::block_on(async move {
        let _ = crate::run_command::run_command(app, error_path, tx, id).await;
    });
}

// run until success (exit code 0)
fn run_until_success(
    app: AppConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            if let Ok(()) =
                crate::run_command::run_command(app.clone(), error_path.clone(), tx.clone(), id)
                    .await
            {
                return;
            }
        }
    })
}

// run until success (exit code 0) and wait before moving to next command
fn run_until_success_and_wait(app: AppConfig, error_path: String, tx: Sender<TuiEvent>, id: usize) {
    task::block_on(async move {
        loop {
            if let Ok(()) =
                crate::run_command::run_command(app.clone(), error_path.clone(), tx.clone(), id)
                    .await
            {
                return;
            }
        }
    });
}

// keep alive, ignoring exit codes
fn run_keep_alive(
    app: AppConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            let _ =
                crate::run_command::run_command(app.clone(), error_path.clone(), tx.clone(), id)
                    .await;
        }
    })
}
