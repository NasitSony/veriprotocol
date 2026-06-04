use crate::network::Network;
use crate::node::{Node,NodeAction};
use crate::trace::{trace, TraceEvent};
use crate::message::{Message, MessageType, VoteValue};

use std::collections::HashSet;



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
                Node::new(4),],
        }
    }

    pub fn run(&mut self) {
        println!("Simulation starting");

        let node_ids: Vec<u64> = self.nodes.iter().map(|node| node.id).collect();

        for &fromNode in  &node_ids {
            let proposal = Message {
                from: fromNode,
                to: 0, // ignored during broadcast
                round: 0,
                msg_type: MessageType::Proposal,
                payload: String::from("proposal"),
                value: VoteValue::Yes,
            };
            self.broadcast(proposal);    
        }

       

        
        //self.broadcast_proposals();
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
                    let actions = node.receive(&msg);

                    for action in actions {
                      match action {
                         NodeAction::BroadcastVote(value) => {
                         self.broadcast(Message {
                            from: msg.to,
                            to: 0,
                            round: msg.round + 1,
                            msg_type: MessageType::Vote,
                            payload: String::from("vote"),
                            value,
                           });
                        }

                        NodeAction::BroadcastCommit(value) => {
                            self.broadcast(Message {
                               from: msg.to,
                               to: 0,
                               round: msg.round + 1,
                               msg_type: MessageType::Commit,
                               payload: String::from("commit"),
                               value,
                              });
                           }


                      }
                    }
                    break;
                }
            }
        }
    }

    fn broadcast(&mut self, template: Message) {
        for node in &self.nodes {
            let mut msg = template.clone();
            msg.to = node.id;
            self.network.send(msg);
        }
    }
}