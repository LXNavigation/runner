/*
This file is part of the Everdream Runner (https://gitlab.com/everdream/runner).
Copyright (c) 2021 Kyoko.

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
};

use async_std::task::{self, JoinHandle};
use futures::future::join_all;
use std::io::stdin;

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

    //ensure error folder exists
    std::fs::create_dir_all(&config.crash_path).expect("Could not create crash path, aborting...");

    // execute all commands, saving handles
    let mut handles = Vec::new();
    for app in config.apps {
        if let Some(handle) = execute_app(app, config.crash_path.clone()) {
            handles.push(handle);
        }
    }

    // wait for all commands to finish
    task::block_on(join_all(handles));

    // quit
    println!("All jobs finished, press enter to quit.");
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a valid string");
}

// executes app based on mode
fn execute_app(app: AppConfig, error_path: String) -> Option<JoinHandle<()>> {
    match app.mode {
        AppMode::RunOnce => return Some(run_once(app, error_path)),
        AppMode::RunOnceAndWait => run_once_and_wait(app, error_path),
        AppMode::RunUntilSuccess => return Some(run_until_success(app, error_path)),
        AppMode::RunUntilSuccessAndWait => run_until_success_and_wait(app, error_path),
        AppMode::KeepAlive => return Some(run_keep_alive(app, error_path)),
    };
    None
}

// run once
fn run_once(app: AppConfig, error_path: String) -> JoinHandle<()> {
    task::spawn(async move {
        let _ = crate::run_command::run_command(app, error_path).await;
    })
}

// run once and wait before moving to next command
fn run_once_and_wait(app: AppConfig, error_path: String) {
    task::block_on(async move {
        let _ = crate::run_command::run_command(app, error_path).await;
    });
}

// run until success (exit code 0)
fn run_until_success(app: AppConfig, error_path: String) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            if let Ok(()) = crate::run_command::run_command(app.clone(), error_path.clone()).await {
                return;
            }
        }
    })
}

// run until success (exit code 0) and wait before moving to next command
fn run_until_success_and_wait(app: AppConfig, error_path: String) {
    task::block_on(async move {
        loop {
            if let Ok(()) = crate::run_command::run_command(app.clone(), error_path.clone()).await {
                return;
            }
        }
    });
}

// keep alive, ignoring exit codes
fn run_keep_alive(app: AppConfig, error_path: String) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            let _ = crate::run_command::run_command(app.clone(), error_path.clone()).await;
        }
    })
}
