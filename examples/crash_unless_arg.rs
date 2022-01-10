use clap::{App, Arg};
use std::{thread::sleep, time::Duration};

fn main() {
    let not_crash = App::new("Runner")
        .version("0.1.0")
        .author("Jurij Robba <jurij.robba@lxnavigation.com>")
        .about("Runner and monitoring application")
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .help("makes app not crash"),
        )
        .get_matches()
        .occurrences_of("v")
        > 0;

    loop {
        sleep(Duration::from_secs(10));
        if not_crash {
            println!("working correctly with right argument");
        } else {
            panic!("This application crashed.");
        }
    }
}
