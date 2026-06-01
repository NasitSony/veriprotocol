mod message;
mod node;
mod network;
mod state;

use message::Message;
use network::Network;
use node::Node;
use message::MessageType;
use state::NodeState;


fn main() {

    let node1 = Node::new(1);
    let mut node2 = Node::new(2);

    println!("Node {} created", node1.id);
    println!("Node {} created", node2.id);

    let mut network = Network::new();

    //let payload = String::from("proposal");

    network.send(Message {
        from: 1,
        to: 2,
        round: 1,
        msg_type: MessageType::Proposal,
        payload: String::from("proposal"),
    });

    if let Some(msg) = network.deliver_next() {
        println!(
            "Delivered message from {} to {}",
            msg.from,
            msg.to
        );
        node2.receive(&msg);
    }

    network.send(Message {
        from: 1,
        to: 2,
        round: 2,
        msg_type: MessageType::Vote,
        payload: String::from("vote"),
    });

    if let Some(msg) = network.deliver_next() {
        println!(
            "Delivered message from {} to {}",
            msg.from,
            msg.to
        );
        node2.receive(&msg);
    }

    network.send(Message {
        from: 1,
        to: 2,
        round: 1,
        msg_type: MessageType::Commit,
        payload: String::from("commit"),
    });
    //println!("{}", payload);

    //let delivered = network.deliver_next();

    if let Some(msg) = network.deliver_next() {
        println!(
            "Delivered message from {} to {}",
            msg.from,
            msg.to
        );
        node2.receive(&msg);
    }

   /* match delivered {
        Some(msg) => {
            //println!("Delivered message from {} to {}", msg.from, msg.to);
            println!(
                "Delivered message from {} to {} in round {} with payload {}",
                msg.from,
                msg.to,
                msg.round,
                msg.payload
            );
        }
        None => {
            println!("No message to deliver");
        }
    }*/
}