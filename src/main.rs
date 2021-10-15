use clap::{crate_version, App, Arg};

mod app_config;
mod config;
mod monitor_stderr;
mod monitor_stdout;
mod run_command;
mod runner;

fn main() {
    let config = parse_args();
    runner::run(config);
}

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
