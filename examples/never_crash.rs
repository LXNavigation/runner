use std::{thread::sleep, time::Duration};

fn main() {
    loop {
        sleep(Duration::from_secs(10));
        println!("Working successfuly");
    }
}
