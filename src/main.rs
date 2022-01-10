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

mod command_config;
mod config;
mod config_error;
mod monitor_stderr;
mod monitor_stdout;
mod run_command;
mod runner;
mod runner_error;
mod tui;
mod tui_state;

use async_std::task;
use clap::{crate_version, App, Arg};

use runner_error::{Result, RunnerError};

// main function
fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    if let Err(err) = task::block_on(runner::run(config)) {
        eprintln!("{}", err);
    }
}

// parse arguments using clap
// runner takes one mandatory argument, path to a config file
fn parse_args() -> Result<String> {
    App::new("Runner")
        .version(crate_version!())
        .author("Jurij Robba <jurij.robba@lxnavigation.com>")
        .about("Runner and monitoring application")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .required(true),
        )
        .get_matches()
        .value_of("config")
        .ok_or(RunnerError::MissingConfiguration)
        .map(|val| val.to_owned())
}
