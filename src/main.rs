mod basic_paxos;
mod message;
mod metrics;
mod multi_paxos;
mod network;
mod node;
mod protocol;
mod raft;
mod scheduler;
mod simulation;
mod stable_multi_paxos;
mod state;
mod trace;

use simulation::Simulation;
//use std::env;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let scheduler = args.get(1).map(|s| s.as_str()).unwrap_or("fifo");

    let runs: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(42);

    let protocol_name = args.get(4).map(String::as_str).unwrap_or("simple");

    let timeout_threshold: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(5);

    let max_delay: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(6);

    let node_count: usize = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(4);

    let delay_probability: f64 = args.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.5);

    let network_model = args.get(9).map(String::as_str).unwrap_or("global");

    let mut results: Vec<u64> = Vec::new();

    for i in 0..runs {
        println!("\n=== Run {} ===", i + 1);

        let _run_seed = seed + i as u64;

        let mut sim = Simulation::new(
            scheduler,
            seed,
            protocol_name,
            timeout_threshold,
            max_delay as usize,
            node_count,
            delay_probability,
            network_model,
        );
        sim.run();

        results.push(sim.metrics.messages_delivered_until_decision);
    }

    if runs > 1 {
        let min = results.iter().min().unwrap();
        let max = results.iter().max().unwrap();

        let sum: u64 = results.iter().sum();
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
