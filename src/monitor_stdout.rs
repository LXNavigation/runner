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
    channel::Sender,
    fs::{File, OpenOptions},
    io::BufReader,
    prelude::*,
};
use chrono::{DateTime, Utc};
use circular_queue::CircularQueue;

use crate::{runner_error::Result, tui_state::TuiEvent};

// log type for stderr
pub(crate) type LogT = CircularQueue<(DateTime<Utc>, String)>;

// monitors std in parent thread. returns only when command exits
pub(crate) async fn monitor_stdout(
    buffer: &mut LogT,
    stdout: File,
    tx: Sender<TuiEvent>,
    id: usize,
) -> Result<()> {
    let mut lines = BufReader::new(stdout).lines();
    while let Some(line) = lines.next().await {
        let line = line?;
        buffer.push((Utc::now(), line.clone()));
        tx.try_send(TuiEvent::NewStdoutMessage(id, line))?;
    }
    Ok(())
}

// saves buffer to a stdout.txt should only be called on error
pub(crate) async fn save_to_file(buffer: LogT, err_path: String) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(&err_path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(err_path + "/stdout.txt")
        .await?;

    for line in buffer.iter() {
        file.write(format!("{} | {}\n", line.0.format("%H:%M:%S"), line.1).as_bytes())
            .await?;
    }

    Ok(())
}
