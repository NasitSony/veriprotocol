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

    let time_model = args.get(10).map(String::as_str).unwrap_or("event-coupled");

    let observation_horizon = args
        .get(11)
        .map(String::as_str)
        .unwrap_or("scheduler-horizon");

    let mut results: Vec<u64> = Vec::new();
    let mut view_changes_results: Vec<u64> = Vec::new();
    let mut max_ballot_results: Vec<u64> = Vec::new();
    let mut stable_tick_results: Vec<Option<u64>> = Vec::new();

    for i in 0..runs {
        println!("\n=== Run {} ===", i + 1);

        let _run_seed = seed + i as u64;

        println!("Run seed: {}", _run_seed);

        let mut sim = Simulation::new(
            scheduler,
            _run_seed,
            protocol_name,
            timeout_threshold,
            max_delay as usize,
            node_count,
            delay_probability,
            network_model,
            time_model,
            observation_horizon,
        );
        sim.run();

        results.push(sim.metrics.messages_delivered_until_decision);

        view_changes_results.push(sim.metrics.view_changes);
        max_ballot_results.push(sim.metrics.max_ballot_seen);
        stable_tick_results.push(sim.metrics.mp_stable_recovery_tick);
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

        let unstable_runs = view_changes_results
            .iter()
            .filter(|&&views| views > 1)
            .count();

        let instability_rate = 100.0 * unstable_runs as f64 / runs as f64;

        let avg_views = view_changes_results.iter().sum::<u64>() as f64 / runs as f64;

        let max_views = *view_changes_results.iter().max().unwrap();

        let max_ballot = *max_ballot_results.iter().max().unwrap();

        let stable_ticks: Vec<u64> = stable_tick_results
            .iter()
            .filter_map(|&tick| tick)
            .collect();

        let stable_recovery_count = stable_ticks.len();

        let avg_stable_tick = if stable_ticks.is_empty() {
            None
        } else {
            Some(stable_ticks.iter().sum::<u64>() as f64 / stable_ticks.len() as f64)
        };

        println!("\n=== Multi-Paxos Instability Summary ===");
        println!("Runs: {}", runs);
        println!("Unstable Runs: {}", unstable_runs);
        println!("Instability Rate: {:.2}%", instability_rate);
        println!("Average View Changes: {:.2}", avg_views);
        println!("Max View Changes: {}", max_views);
        println!("Max Ballot Seen: {}", max_ballot);

        println!(
            "Stable Recovery Observed: {}/{}",
            stable_recovery_count, runs
        );

        match avg_stable_tick {
            Some(avg) => println!("Average Stable Recovery Tick: {:.2}", avg),
            None => println!("Average Stable Recovery Tick: N/A"),
        }
    }
}
