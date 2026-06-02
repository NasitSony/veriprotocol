use crate::network::Network;
use crate::node::Node;
use crate::trace::{trace, TraceEvent};
use crate::message::{Message, MessageType, VoteValue};

pub struct Simulation {
    pub network: Network,
    pub node1: Node,
    pub node2: Node,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            network: Network::new(),
            node1: Node::new(1),
            node2: Node::new(2),
        }
    }

    pub fn run(&mut self) {
        println!("Simulation starting");

        self.network.send(Message {
            from: self.node1.id,
            to: self.node2.id,
            round: 1,
            msg_type: MessageType::Vote,
            payload: String::from("vote"),
            value: VoteValue::Yes,
        });

        self.network.send(Message {
            from: self.node1.id,
            to: self.node2.id,
            round: 2,
            msg_type: MessageType::Vote,
            payload: String::from("vote"),
            value: VoteValue::Yes,
        });

        self.network.send(Message {
            from: self.node1.id,
            to: self.node2.id,
            round: 3,
            msg_type: MessageType::Vote,
            payload: String::from("vote"),
            value: VoteValue::Yes,
        });

        while let Some(msg) = self.network.deliver_next() {
            trace(
                TraceEvent::Deliver,
                &format!("{} -> {}", msg.from, msg.to),
            );

            if msg.to == self.node2.id {
                self.node2.receive(&msg);
            }
        }
    }
}