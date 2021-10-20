use async_std::task::{self, sleep};
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};
use std::time::Duration;

fn main() {
    let choices = [true, false];
    let weights = [9, 1];
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = thread_rng();

    loop {
        task::block_on(sleep(Duration::from_secs(10)));
        if choices[dist.sample(&mut rng)] {
            println!("Working successfuly");
        } else {
            panic!("This application crashed.");
        }
    }
}
