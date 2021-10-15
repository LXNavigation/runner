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

use async_std::task;
use chrono::{DateTime, Utc};
use std::path::Path;
use subprocess::{ExitStatus, Popen, PopenConfig, Redirection};

use crate::{app_config::AppConfig, monitor_stdout::LogT};

// runs command, starting stdout and stderr monitoring
pub(crate) async fn run_command(config: AppConfig, error_path: String) -> Result<(), ExitStatus> {
    let name = get_name(&config.command);
    let (mut process, start) = run(config.command, config.args);
    let process_folder = error_path + "/" + &name + "-" + &start.to_rfc3339();

    let stderr = process.stderr.take().unwrap();
    task::spawn(crate::monitor_stderr::monitor_stderr(
        process_folder.clone(),
        stderr,
    ));

    let mut buffer = LogT::with_capacity(config.hist);
    let stdout = process.stdout.take().unwrap();
    crate::monitor_stdout::monitor_stdout(&mut buffer, stdout);

    let exit_status = process
        .wait()
        .expect("Process owned by runner killed from outside");
    if exit_status != ExitStatus::Exited(0u32) {
        crate::monitor_stdout::save_to_file(buffer, process_folder);
        return Err(exit_status);
    }
    Ok(())
}

// run detached with stdout and stderr piped
fn run(command: String, args: Vec<String>) -> (Popen, DateTime<Utc>) {
    (
        Popen::create(
            &create_command(command, args),
            PopenConfig {
                stdout: Redirection::Pipe,
                stderr: Redirection::Pipe,
                detached: true,
                ..Default::default()
            },
        )
        .unwrap(),
        Utc::now(),
    )
}

// get name from command. should extract file name from executable path
fn get_name(command: &str) -> String {
    Path::new(command)
        .file_stem()
        .expect("This is not a valid command!")
        .to_str()
        .unwrap()
        .to_owned()
}

// created full command array from command and arguments
fn create_command(command: String, mut args: Vec<String>) -> Vec<String> {
    args.insert(0, command);
    args
}
