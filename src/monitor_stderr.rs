use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

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

fn append_to_file(err_path: String, err_string: String) {
    std::fs::create_dir_all(&err_path).expect("Could not create crash path, aborting...");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(err_path + "/stderr.txt")
        .unwrap();

    writeln!(file, "{}", err_string).expect("could not write to err file");
}
