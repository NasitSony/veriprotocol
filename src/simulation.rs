use crate::basic_paxos::BasicPaxosProtocol;
use crate::message::{Message, MessageType, VoteValue};
use crate::metrics::Metrics;
use crate::network::Network;
use crate::node::{Node, NodeAction};
use crate::protocol::{Protocol, SimpleConsensusProtocol, TimeoutProtocol, TwoPhaseProtocol};
use crate::scheduler::SchedulerOutcome;
use crate::trace::{Config, TraceEvent, trace};

pub struct Simulation {
    pub network: Network,
    nodes: Vec<Node>,
    pub metrics: Metrics,
    pub config: Config,
    pub timeout_injected: bool,
    pub timeout_threshold: u64,
    protocol_name: String,

    // pub protocol: SimpleConsensusProtocol,
    pub protocol: Box<dyn Protocol>,
}

impl Simulation {
    pub fn new(
        scheduler_name: &str,
        seed: u64,
        protocol_name: &str,
        timeout_threshold: u64,
        max_delay: usize,
    ) -> Self {
        let protocol: Box<dyn Protocol> = match protocol_name {
            "two-phase" => Box::new(TwoPhaseProtocol::new()),
            "paxos" => Box::new(BasicPaxosProtocol::new()),
            "paxos-adopt" => Box::new(BasicPaxosProtocol::new()),
            "timeout" => Box::new(TimeoutProtocol::new(4)),
            _ => Box::new(SimpleConsensusProtocol::new()),
        };

        Self {
            network: Network::new(scheduler_name, seed, max_delay),
            nodes: vec![Node::new(1), Node::new(2), Node::new(3), Node::new(4)],
            metrics: Metrics::new(),
            config: Config {
                print_trace: false,
                print_state_changes: true,
                print_quorums: true,
                print_decisions: true,
            },
            protocol,
            timeout_injected: false,
            timeout_threshold: timeout_threshold,
            protocol_name: protocol_name.to_string(),
        }
    }

    pub fn run(&mut self) {
        println!("Simulation starting");



        if self.protocol_name == "paxos" {
            self.broadcast(Message {
                from: 1,
                to: 0,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 1 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });

            self.broadcast(Message {
                from: 2,
                to: 0,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 2 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "paxos-adopt" {
            for node in &mut self.nodes {
            if node.id == 3 {
               node.accepted_ballot = Some(5);
               node.accepted_value = Some("old".to_string());
            }
        }

        self.broadcast(Message {
           from: 1,
           to: 0,
           round: 0,
           msg_type: MessageType::Prepare { ballot: 6 },
           payload: String::from("prepare"),
           value: VoteValue::Yes,
           delay_count: 0,
        });
        }else {
            let node_ids: Vec<u64> = self.nodes.iter().map(|node| node.id).collect();

            for &from_node in &node_ids {
                if !self
                    .protocol
                    .should_send_initial_proposal(from_node as usize)
                {
                    continue;
                }

                self.broadcast(Message {
                    from: from_node,
                    to: 0,
                    round: 0,
                    msg_type: MessageType::Proposal,
                    payload: String::from("proposal"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });
            }
        }

        self.deliver_all_messages();

        self.metrics.decisions = self
            .nodes
            .iter()
            .filter(|node| node.decided.is_some())
            .count() as u64;

        self.metrics.print();
    }

    fn deliver_all_messages(&mut self) {
        loop {
            match self.network.deliver_next() {
                SchedulerOutcome::Deliver(msg) => {
                    self.metrics.scheduler_steps += 1;
                    self.metrics.messages_delivered += 1;

                    if self.protocol.uses_timeout()
                        && !self.timeout_injected
                        && self.metrics.scheduler_steps >= self.timeout_threshold
                    {
                        self.inject_timeouts();
                        self.timeout_injected = true;
                    }

                    if msg.msg_type == MessageType::Timeout {
                        self.metrics.timeouts_triggered += 1;
                        self.metrics.view_changes += 1;
                    }

                    trace(
                        &self.config,
                        TraceEvent::Deliver,
                        &format!("{} -> {}", msg.from, msg.to),
                    );

                    for node in &mut self.nodes {
                        if node.id == msg.to {
                            match &msg.msg_type {
                                MessageType::Prepare { .. } => {
                                    self.metrics.prepare_messages += 1;
                                }
                                MessageType::Promise { .. } => {
                                    self.metrics.promise_messages += 1;
                                }
                                MessageType::AcceptRequest { .. } => {
                                    self.metrics.accept_requests += 1;
                                }
                                MessageType::Accepted { .. } => {
                                    self.metrics.accepted_messages += 1;
                                }
                                _ => {}
                            }
                            let actions = self.protocol.handle_message(node, &msg);

                            for action in actions {
                                match action {
                                    NodeAction::BroadcastProposal => {
                                        if self.protocol_name == "paxos" {
                                            self.broadcast(Message {
                                                from: msg.to,
                                                to: 0,
                                                round: 0,
                                                msg_type: MessageType::Prepare { ballot: 1 },
                                                payload: String::from("prepare"),
                                                value: VoteValue::Yes,
                                                delay_count: 0,
                                            });
                                        } else {
                                            self.broadcast(Message {
                                                from: msg.to,
                                                to: 0,
                                                round: msg.round + 1,
                                                msg_type: MessageType::Proposal,
                                                payload: String::from("proposal"),
                                                value: VoteValue::Yes,
                                                delay_count: msg.delay_count,
                                            });
                                        }
                                    }

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

                                    NodeAction::BroadcastTimeout => {
                                        // TODO: create timeout messages
                                    }

                                    NodeAction::StaleMessageIgnored => {
                                        self.metrics.stale_messages_ignored += 1;
                                    }

                                    NodeAction::SendPromise {
                                        to,
                                        ballot,
                                        accepted_ballot,
                                        accepted_value,
                                    } => {
                                        self.send_to(Message {
                                            from: msg.to,
                                            to,
                                            round: msg.round + 1,
                                            msg_type: MessageType::Promise {
                                                ballot,
                                                accepted_ballot,
                                                accepted_value,
                                            },
                                            payload: String::from("promise"),
                                            value: msg.value.clone(),
                                            delay_count: msg.delay_count,
                                        });
                                    }

                                    NodeAction::BroadcastAcceptRequest { ballot, value } => {
                                        self.broadcast(Message {
                                            from: msg.to,
                                            to: 0,
                                            round: msg.round + 1,
                                            msg_type: MessageType::AcceptRequest {
                                                ballot,
                                                value: value.clone(),
                                            },
                                            payload: String::from("accept_request"),
                                            value: msg.value.clone(),
                                            delay_count: msg.delay_count,
                                        });
                                    }

                                    NodeAction::SendAccepted { to, ballot, value } => {
                                        self.send_to(Message {
                                            from: msg.to,
                                            to,
                                            round: msg.round + 1,
                                            msg_type: MessageType::Accepted {
                                                ballot,
                                                value: value.clone(),
                                            },
                                            payload: String::from("accepted"),
                                            value: msg.value.clone(),
                                            delay_count: msg.delay_count,
                                        });
                                    }
                                }
                            }

                            break;
                        }
                    }

                    if self.nodes.iter().all(|node| node.decided.is_some()) {
                        self.metrics.messages_delivered_until_decision =
                            self.metrics.messages_delivered;
                        self.metrics.messages_sent_until_decision = self.metrics.messages_sent;
                        break;
                    }
                }

                SchedulerOutcome::Delay => {
                    self.metrics.scheduler_steps += 1;

                    if self.protocol.uses_timeout()
                        && !self.timeout_injected
                        && self.metrics.scheduler_steps >= self.timeout_threshold
                    {
                        self.inject_timeouts();
                        self.timeout_injected = true;
                    }
                }

                SchedulerOutcome::Empty => {
                    break;
                }
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

    fn send_to(&mut self, msg: Message) {
        self.metrics.messages_sent += 1;
        self.network.send(msg);
    }

    fn inject_timeouts(&mut self) {
        let node_ids: Vec<u64> = self.nodes.iter().map(|node| node.id).collect();

        for node_id in node_ids {
            let timeout = Message {
                from: 0,
                to: node_id,
                round: 0,
                msg_type: MessageType::Timeout,
                payload: String::from("timeout"),
                value: VoteValue::Yes,
                delay_count: 0,
            };

            self.metrics.messages_sent += 1;
            self.network.send(timeout);
        }
    }
}
