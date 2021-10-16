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

use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use chrono::Utc;

// runs another thread to monitor standard err. all outputs are stored in stderr.txt file in folder
pub(crate) async fn monitor_stderr(err_path: String, stderr: File) {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        match line {
            Ok(line) => append_to_file(err_path.clone(), line),
            Err(err) => {
                eprintln!("quitting sterr monitoring because of {}", err);
                return;
            }
        }
    }
}

// appends line to appropriate error file
fn append_to_file(err_path: String, err_string: String) {
    std::fs::create_dir_all(&err_path).expect("Could not create crash path, aborting...");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(err_path + "/stderr.txt")
        .unwrap();

        writeln!(file, "{} | {}", Utc::now().to_rfc3339(), err_string)
        .expect("could not write to stderr file");
}
