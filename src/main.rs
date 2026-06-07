mod message;
mod network;
mod node;
mod scheduler;
mod simulation;
mod state;
mod trace;
mod metrics;
mod protocol;

use simulation::Simulation;
//use std::env;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let scheduler = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("fifo");

    let runs: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let seed: u64 = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);    

    let protocol_name = args.get(4)
        .map(String::as_str)
        .unwrap_or("simple");    

    let mut results: Vec<usize> = Vec::new();

    for i in 0..runs {
        println!("\n=== Run {} ===", i + 1);
    
        let run_seed = seed + i as u64;
    
        let mut sim = Simulation::new(scheduler, seed, protocol_name);
        sim.run();
    
        results.push(sim.metrics.decision_delivery_count);
    }

    if runs > 1 {
        let min = results.iter().min().unwrap();
        let max = results.iter().max().unwrap();

        let sum: usize = results.iter().sum();
        let avg = sum as f64 / results.len() as f64;

        println!("\n=== Summary ===");
        println!("Scheduler: {}", scheduler);
        println!("Runs: {}", runs);
        println!("Min Decision Delivery Count: {}", min);
        println!("Max Decision Delivery Count: {}", max);
        println!("Average Decision Delivery Count: {:.2}", avg);

        println!("scheduler,runs,min,max,avg");
        println!("{},{},{},{},{:.2}", scheduler, runs, min, max, avg);
    }
}