mod message;
mod node;
mod network;
mod state;
mod trace;
mod scheduler;
mod simulation;

use message::Message;
use network::Network;
use node::Node;
use message::MessageType;
use trace::TraceEvent;
use trace::trace;
use simulation::Simulation;


fn main() {

    let mut sim = Simulation::new();
    sim.run();
}