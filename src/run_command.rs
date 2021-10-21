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

use async_std::{channel::Sender, task};
use chrono::{DateTime, Utc};
use subprocess::{ExitStatus, Popen, PopenConfig, Redirection};

use crate::{command_config::CommandConfig, monitor_stdout::LogT, tui_state::TuiEvent};

// runs command, starting stdout and stderr monitoring
pub(crate) async fn run_command(
    config: CommandConfig,
    error_path: String,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<(), ExitStatus> {
    tx.try_send(TuiEvent::CommandStarted(id))
        .expect("unbound channel should never be full");
    let (mut process, start) = run(config.command, config.args);
    let process_folder = format!(
        "{}/{}-{}",
        error_path,
        config.name,
        start.format("%Y-%m-%d_%H:%M:%S")
    );

    task::spawn(crate::monitor_stderr::monitor_stderr(
        process_folder.clone(),
        process.stderr.take().unwrap(),
        tx.clone(),
        id,
    ));

    let mut buffer = LogT::with_capacity(config.stdout_history);
    crate::monitor_stdout::monitor_stdout(
        &mut buffer,
        process.stdout.take().unwrap(),
        tx.clone(),
        id,
    );

    let exit_status = process
        .wait()
        .expect("Process owned by runner killed from outside");
    tx.try_send(TuiEvent::CommandEnded(id))
        .expect("unbound channel should never be full");
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

// created full command array from command and arguments
fn create_command(command: String, mut args: Vec<String>) -> Vec<String> {
    args.insert(0, command);
    args
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create_command() {
        let command = String::from("test");
        let args = Vec::new();
        assert_eq!(create_command(command, args), [String::from("test")]);
    }
}
