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

use async_std::{fs::OpenOptions, prelude::*};
use crate::{runner_error::RunnerError, tui_state::TuiEvent};
use async_std::{channel::Sender, fs::File, io::BufReader};
use chrono::Utc;

// runs another thread to monitor standard err. all outputs are stored in stderr.txt file in folder
pub(crate) async fn monitor_stderr(
    err_path: String,
    stderr: File,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<(), RunnerError> {
    let mut lines = BufReader::new(stderr).lines();
    while let Some(line) = lines.next().await {
        let line = line?;
        append_to_file(err_path.clone(), line.clone()).await?;
        tx.try_send(TuiEvent::NewStderrMessage(id, line))
            .expect("unbound channel should never be full");
    }
    Ok(())
}

// appends line to appropriate error file
async fn append_to_file(err_path: String, err_string: String) -> Result<(), RunnerError> {
    std::fs::create_dir_all(&err_path).expect("Could not create crash path, aborting...");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(err_path + "/stderr.txt").await
        .unwrap();

    file.write_all(format!("{} | {}\n", Utc::now().format("%H:%M:%S"), err_string).as_bytes()).await?;
    Ok(())
}
