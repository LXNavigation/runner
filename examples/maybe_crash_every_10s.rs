use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};
use std::{thread::sleep, time::Duration};

fn main() {
    let choices = [true, false];
    let weights = [9, 1];
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = thread_rng();

    loop {
        sleep(Duration::from_secs(10));
        if choices[dist.sample(&mut rng)] {
            println!("Working successfuly");
        } else {
            panic!("This application crashed.");
        }
    }
}
