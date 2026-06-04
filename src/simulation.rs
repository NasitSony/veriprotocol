use crate::network::Network;
use crate::node::Node;
use crate::trace::{trace, TraceEvent};
use crate::message::{Message, MessageType, VoteValue};

pub struct Simulation {
    pub network: Network,
    nodes: Vec<Node>,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            network: Network::new(),
            nodes: vec![
                Node::new(1),
                Node::new(2),
                Node::new(3),
                Node::new(4),
],
        }
    }

    pub fn run(&mut self) {
        println!("Simulation starting");

        self.broadcast_proposals();
        self.deliver_all_messages();
    }

    fn broadcast_proposals(&mut self) {
        for fromNode in  &self.nodes {
            for toNode in  &self.nodes {  
                self.network.send(Message {
                    from: fromNode.id,
                    to: toNode.id,
                    round: 0,
                    msg_type: MessageType::Proposal,
                    payload: String::from("proposal"),
                    value: VoteValue::Yes,
                });   
            }
        }
    }

    fn deliver_all_messages(&mut self) {
        while let Some(msg) = self.network.deliver_next() {
            trace(
                TraceEvent::Deliver,
                &format!("{} -> {}", msg.from, msg.to),
            );
    
            for node in &mut self.nodes {
                if node.id == msg.to {
                    node.receive(&msg);
                    break;
                }
            }
        }
    }
}