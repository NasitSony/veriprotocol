use crate::network::Network;
use crate::node::{Node, NodeAction};
use crate::trace::{trace, TraceEvent, Config};
use crate::message::{Message, MessageType, VoteValue};
use crate::metrics::Metrics;
use crate::protocol::{Protocol, SimpleConsensusProtocol, TwoPhaseProtocol};




pub struct Simulation {
    pub network: Network,
    nodes: Vec<Node>,
    pub metrics: Metrics,
    pub config: Config,
    // pub protocol: SimpleConsensusProtocol,
    pub protocol: Box<dyn Protocol>
}

impl Simulation {
    pub fn new(
        scheduler_name: &str,
        seed: u64,
        protocol_name: &str,
    ) -> Self {
        let protocol: Box<dyn Protocol> = match protocol_name {
            "two-phase" => Box::new(TwoPhaseProtocol::new()),
            _ => Box::new(SimpleConsensusProtocol::new()),
        };

        Self {
            network: Network::new(scheduler_name, seed),
            nodes: vec![
                Node::new(1),
                Node::new(2),
                Node::new(3),
                Node::new(4),
            ],
            metrics: Metrics::new(),
            config: Config {
                print_trace: false,
                print_state_changes: true,
                print_quorums: true,
                print_decisions: true,
            },
            protocol,
        }
    }


    pub fn run(&mut self) {
        println!("Simulation starting");

        let node_ids: Vec<u64> = self.nodes.iter().map(|node| node.id).collect();

        for &from_node in  &node_ids {
            let proposal = Message {
                from: from_node,
                to: 0, // ignored during broadcast
                round: 0,
                msg_type: MessageType::Proposal,
                payload: String::from("proposal"),
                value: VoteValue::Yes,
                delay_count: 0,
            };
            self.broadcast(proposal);    
        }

       

        
        //self.broadcast_proposals();
        self.deliver_all_messages();
        self.metrics.decisions = self.nodes
            .iter()
            .filter(|node| node.decided.is_some())
            .count();

            println!("\n=== Metrics ===");
            println!("Messages Sent: {}", self.metrics.messages_sent);
            println!("Messages Delivered: {}", self.metrics.messages_delivered);
            println!(
                "Decision Delivery Count: {}",
                self.metrics.decision_delivery_count
            );
            println!(
                "Messages Sent Until Decision: {}",
                self.metrics.messages_sent_until_decision
            );
            println!(
                "Undelivered Messages At Decision: {}",
                self.metrics.messages_sent_until_decision
                    - self.metrics.decision_delivery_count
            );
            println!("Decisions: {}", self.metrics.decisions);
    }

    fn deliver_all_messages(&mut self) {
        while let Some(msg) = self.network.deliver_next() {
            self.metrics.messages_delivered += 1;
    
            trace(
                &self.config,
                TraceEvent::Deliver,
                &format!("{} -> {}", msg.from, msg.to),
            );
    
            for node in &mut self.nodes {
                if node.id == msg.to {
                    let actions = self.protocol.handle_message(node, &msg);
    
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
                                    delay_count: msg.delay_count,
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
                                    delay_count: msg.delay_count,
                                });
                            }
                        }
                    }
    
                    break;
                }
            }
    
            // Stop once all nodes have decided.
            if self.nodes.iter().all(|node| node.decided.is_some()) {
                self.metrics.decision_delivery_count = self.metrics.messages_delivered;
                self.metrics.messages_sent_until_decision = self.metrics.messages_sent;
                break;
            }
        }
    }

    fn broadcast(&mut self, template: Message) {
        for node in &self.nodes {
            let mut msg = template.clone();
            msg.to = node.id;
            self.metrics.messages_sent += 1;
            self.network.send(msg);
        }
    }
}