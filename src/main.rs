mod message;
mod node;
mod network;

use message::Message;
use network::Network;

fn main() {
    let mut network = Network::new();

    network.send(Message {
        from: 1,
        to: 2,
        round: 1,
    });

    let delivered = network.deliver_next();

    match delivered {
        Some(msg) => {
            //println!("Delivered message from {} to {}", msg.from, msg.to);
            println!(
                "Delivered message from {} to {} in round {}",
                msg.from, msg.to, msg.round
            );
        }
        None => {
            println!("No message to deliver");
        }
    }
}