mod message;
mod network;
mod node;
mod scheduler;
mod simulation;
mod state;
mod trace;

use simulation::Simulation;

fn main() {
    let mut sim = Simulation::new();
    sim.run();
}