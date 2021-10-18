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

mod app_config;
mod config;
mod monitor_stderr;
mod monitor_stdout;
mod run_command;
mod runner;
mod tui;

use clap::{crate_version, App, Arg};

// main function
fn main() {
    let config = parse_args();
    runner::run(config);
}

// parse arguments using clap
//
// runner takes one mandatory argument, path to a config file
fn parse_args() -> String {
    let matches = App::new("Runner")
        .version(crate_version!())
        .author("Jurij R. <jurij.robba@vernocte.org>")
        .about("Runner and monitoring application")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    matches
        .value_of("config")
        .expect("no config file provided, quitting")
        .to_owned()
}
