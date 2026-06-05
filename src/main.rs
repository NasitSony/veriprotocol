mod message;
mod network;
mod node;
mod scheduler;
mod simulation;
mod state;
mod trace;
mod metrics;

use simulation::Simulation;

fn main() {
    let mut sim1 = Simulation::new();
    sim1.run();
    let mut sim2 = Simulation::new();
    sim2.run();
}