mod message;
mod network;
mod node;
mod scheduler;
mod simulation;
mod state;
mod trace;
mod metrics;

use simulation::Simulation;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let scheduler = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("fifo");

    let runs: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    for i in 0..runs {
        println!("\n=== Run {} ===", i + 1);

        let mut sim = Simulation::new(scheduler);
        sim.run();
    }
}