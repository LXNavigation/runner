use async_std::task::{self, sleep};
use std::time::Duration;

fn main() {
    loop {
        task::block_on(sleep(Duration::from_secs(10)));
        println!("Working successfuly");
    }
}
