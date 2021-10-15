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

use chrono::{DateTime, Utc};
use circular_queue::CircularQueue;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Write;
use std::{fs::File, io::BufReader};

pub(crate) type LogT = CircularQueue<(DateTime<Utc>, String)>;

pub(crate) fn monitor_stdout(buffer: &mut LogT, stdout: File) {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        match line {
            Ok(line) => buffer.push((Utc::now(), line)),
            Err(err) => {
                eprintln!("quitting sterr monitoring because of {}", err);
                return;
            }
        };
    }
}

pub(crate) fn save_to_file(buffer: LogT, err_path: String) {
    if buffer.is_empty() {
        return
    }
    std::fs::create_dir_all(&err_path).expect("Could not create crash path, aborting...");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(err_path + "/stdout.txt")
        .unwrap();

    for line in buffer.iter() {
        writeln!(file, "{} | {}", line.0.to_rfc3339(), line.1)
            .expect("could not write to stdout file");
    }
}
