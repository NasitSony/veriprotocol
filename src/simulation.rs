use crate::basic_paxos::BasicPaxosProtocol;
use crate::message::{Message, MessageType, VoteValue};
use crate::metrics::Metrics;
use crate::network::Network;
use crate::node::{Node, NodeAction};
use crate::protocol::{Protocol, SimpleConsensusProtocol, TimeoutProtocol, TwoPhaseProtocol};
use crate::scheduler::SchedulerOutcome;
use crate::trace::{Config, TraceEvent, trace};
use crate::raft::RaftProtocol;
use crate::node::RaftRole;

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

    pub node_count: usize,
} 

impl Simulation {
    pub fn new(
        scheduler_name: &str,
        seed: u64,
        protocol_name: &str,
        timeout_threshold: u64,
        max_delay: usize,
        node_count: usize,
    ) -> Self {

        //let node_count = 4;

        let nodes: Vec<Node> = (1..=node_count as u64)
          .map(Node::new)
          .collect();

        let quorum_size = (node_count / 2) + 1;

        println!("Nodes: {}", node_count);
        println!("Quorum: {}", quorum_size);

        let protocol: Box<dyn Protocol> = match protocol_name {
            "two-phase" => Box::new(TwoPhaseProtocol::new()),
            "paxos" => Box::new(BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())),
            "paxos-adopt" => Box::new(BasicPaxosProtocol::new_with_proposer(1, 6, "v1".to_string()).with_quorum_size(quorum_size)),
            "paxos-retry" => Box::new(BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string()).with_quorum_size(quorum_size)),
            "paxos-dual" => Box::new( BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_ballot_value(2, "v2")
                .with_ballot_value(3, "v2")
                .with_quorum_size(quorum_size)
            ),
            "paxos-dual-adopt" => Box::new( BasicPaxosProtocol::new_with_proposer(2, 3, "v2".to_string())
                .with_ballot_value(1, "v1")
                .with_ballot_value(3, "v2")
                .with_quorum_size(quorum_size)
            ),

            "paxos-timeout-retry" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_quorum_size(quorum_size)
            ),

            "paxos-timeout-max" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_quorum_size(quorum_size)
            ),

            "paxos-partial-timeout" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_quorum_size(quorum_size)
            ),

            "paxos-missed-accept-recovery" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_quorum_size(quorum_size)
            ),

            "paxos-partition-heal" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_quorum_size(quorum_size)
            ),

            "paxos-crash-recovery" => Box::new(
                BasicPaxosProtocol::new_with_proposer(2, 2, "v2".to_string())
                .with_ballot_value(1, "v1")
                .with_ballot_value(2, "v2")
                .with_quorum_size(quorum_size)
            ),

            "paxos-race" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_ballot_value(1, "v1")
                .with_ballot_value(2, "v2")
                .with_ballot_value(3, "v2")
                .with_quorum_size(quorum_size)
            ),

            "paxos-leader-handoff" => Box::new(
                BasicPaxosProtocol::new_with_proposer(2, 2, "v2".to_string())
                .with_ballot_value(1, "v1")
                .with_ballot_value(2, "v2")
                .with_quorum_size(quorum_size)
            ),

            "paxos-partition-heal-adopt" => Box::new(
                BasicPaxosProtocol::new_with_proposer(2, 2, "v2".to_string())
                .with_ballot_value(1, "v1")
                .with_ballot_value(2, "v2")
                .with_quorum_size(quorum_size)
            ),

            "paxos-membership-change" => Box::new(
                BasicPaxosProtocol::new_with_proposer(1, 1, "v1".to_string())
                .with_quorum_size(quorum_size)
            ),

            "raft-election" => Box::new(RaftProtocol::new(quorum_size)),

            "raft-leader-crash" => Box::new(RaftProtocol::new(quorum_size)),

            "raft-partition-heal" => Box::new(RaftProtocol::new(quorum_size)),

            "raft-membership-change" => Box::new(RaftProtocol::new(quorum_size)),

            "timeout" => Box::new(TimeoutProtocol::new(4)),
            _ => Box::new(SimpleConsensusProtocol::new()),

            
        };

        Self {
            network: Network::new(scheduler_name, seed, max_delay),
            nodes,//vec![Node::new(1), Node::new(2), Node::new(3), Node::new(4)],
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
            node_count,
        }
    }

    fn quorum_size(&self) -> usize {
        (self.nodes.len() / 2) + 1
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
        } else if self.protocol_name == "paxos-dual" {
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
            // accepted-value adoption scenario
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
                       }else if self.protocol_name == "paxos-retry" {
            for node in &mut self.nodes {
                node.promised_ballot = 1;
            }
        
            self.broadcast(Message {
                from: 1,
                to: 0,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 1 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "paxos-dual-adopt" {
               for node in &mut self.nodes {
                  if node.id == 3 {
                     node.accepted_ballot = Some(1);
                     node.accepted_value = Some("v1".to_string());
                  }
                }

               self.broadcast(Message {
                  from: 2,
                  to: 0,
                  round: 0,
                  msg_type: MessageType::Prepare { ballot: 3 },
                  payload: String::from("prepare"),
                  value: VoteValue::Yes,
                  delay_count: 0,
                });
        } else if self.protocol_name == "paxos-timeout-retry" {
            self.metrics.timeouts_triggered += 1;
              let actions = self.protocol.on_timeout();

            for action in actions {
               // self.apply_action(&msg, action);
               match action {
                   NodeAction::BroadcastPrepare { ballot } => {
                     self.metrics.paxos_retries += 1;
                     self.metrics.max_ballot_seen =
                     self.metrics.max_ballot_seen.max(ballot);

                     self.broadcast(Message {
                    from: 1,
                    to: 0,
                    round: 1,
                    msg_type: MessageType::Prepare { ballot },
                    payload: String::from("prepare"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });
            }
            _ => {}
        }
            }
        } else if self.protocol_name == "paxos-timeout-max" {
            for _ in 0..4 {
                self.metrics.timeouts_triggered += 1;

                let actions = self.protocol.on_timeout();

                if actions.is_empty() {
                    self.metrics.paxos_retry_exhausted = true;
                }

                for action in actions {
                    match action {
                        NodeAction::BroadcastPrepare { ballot } => {
                            self.metrics.paxos_retries += 1;
                            self.metrics.max_ballot_seen =
                                self.metrics.max_ballot_seen.max(ballot);
                        }
                        _ => {}
                    }
                }
            }
        } else if self.protocol_name == "paxos-partial-timeout" {
            // Send Prepare(1) only to 2 nodes, not quorum.
            self.network.send(Message {
                from: 1,
                to: 2,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 1 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });

            self.network.send(Message {
                from: 1,
                to: 3,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 1 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });

            self.metrics.timeouts_triggered += 1;

            let actions = self.protocol.on_timeout();

            for action in actions {
                
                match action {
                    NodeAction::BroadcastPrepare { ballot } => {
                        self.metrics.paxos_retries += 1;
                        self.metrics.max_ballot_seen =
                            self.metrics.max_ballot_seen.max(ballot);

                        self.broadcast(Message {
                            from: 1,
                            to: 0,
                            round: 1,
                            msg_type: MessageType::Prepare { ballot },
                            payload: String::from("prepare"),
                            value: VoteValue::Yes,
                            delay_count: 0,
                        });
                    }
                    _ => {}
                }
            }
        } else if self.protocol_name == "paxos-missed-accept-recovery" {
            // Simulate partial Phase-2 progress:
            // two acceptors already accepted v1 at ballot 1,
            // but not enough Accepted messages reached proposer to decide.
            for node in &mut self.nodes {
                if node.id == 2 || node.id == 3 {
                    node.promised_ballot = 1;
                    node.accepted_ballot = Some(1);
                    node.accepted_value = Some("v1".to_string());
                }
            }

            self.metrics.timeouts_triggered += 1;

            let actions = self.protocol.on_timeout();

            for action in actions {
                
                match action {
                    NodeAction::BroadcastPrepare { ballot } => {
                        self.metrics.paxos_retries += 1;
                        self.metrics.max_ballot_seen =
                            self.metrics.max_ballot_seen.max(ballot);

                        self.broadcast(Message {
                            from: 1,
                            to: 0,
                            round: 1,
                            msg_type: MessageType::Prepare { ballot },
                            payload: String::from("prepare"),
                            value: VoteValue::Yes,
                            delay_count: 0,
                        });
                    }
                    _ => {}
                }
            }
        } else if self.protocol_name == "paxos-partition-heal" {
            // Partition-like phase:
            // proposer can reach only 2 acceptors, not enough for full progress.
            self.network.send(Message {
                from: 1,
                to: 2,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 1 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });

            self.network.send(Message {
                from: 1,
                to: 3,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 1 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });

            // Heal/retry phase.
            self.metrics.timeouts_triggered += 1;

            let actions = self.protocol.on_timeout();

            for action in actions {
               
                match action {
                    NodeAction::BroadcastPrepare { ballot } => {
                        self.metrics.paxos_retries += 1;
                        self.metrics.max_ballot_seen =
                            self.metrics.max_ballot_seen.max(ballot);

                        self.broadcast(Message {
                            from: 1,
                            to: 0,
                            round: 1,
                            msg_type: MessageType::Prepare { ballot },
                            payload: String::from("prepare"),
                            value: VoteValue::Yes,
                            delay_count: 0,
                        });
                    }
                    _ => {}
                }
            }
        } else if self.protocol_name == "paxos-crash-recovery" {
            // Simulate proposer 1 crashed after v1 was accepted by some acceptors.
            // New proposer 2 starts ballot 2 and must recover/adopt v1.
            for node in &mut self.nodes {
                if node.id == 2 || node.id == 3 {
                    node.promised_ballot = 1;
                    node.accepted_ballot = Some(1);
                    node.accepted_value = Some("v1".to_string());
                }
            }

            self.broadcast(Message {
                from: 2,
                to: 0,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 2 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "paxos-race" {
            // Two proposers race:
            // proposer 1 starts ballot 1 with v1
            // proposer 2 starts ballot 2 with v2
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
        } else if self.protocol_name == "paxos-leader-handoff" {
            // Old leader/proposer partially progressed with v1.
            for node in &mut self.nodes {
                if node.id == 2 {
                    node.promised_ballot = 1;
                    node.accepted_ballot = Some(1);
                    node.accepted_value = Some("v1".to_string());
                }
            }

            // New leader/proposer takes over with higher ballot.
            self.broadcast(Message {
                from: 2,
                to: 0,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 2 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "paxos-partition-heal-adopt" {
            // During partition, v1 was accepted by a subset.
            for node in &mut self.nodes {
                if node.id == 2 || node.id == 3 {
                    node.promised_ballot = 1;
                    node.accepted_ballot = Some(1);
                    node.accepted_value = Some("v1".to_string());
                }
            }

            // Partition heals. New proposer/leader tries v2 at higher ballot,
            // but must adopt v1 from acceptor promises.
            self.broadcast(Message {
                from: 2,
                to: 0,
                round: 0,
                msg_type: MessageType::Prepare { ballot: 2 },
                payload: String::from("prepare"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "paxos-membership-change" {
            self.broadcast(Message {
                from: 1,
                to: 0,
                round: 0,
                msg_type: MessageType::MembershipChange { new_node_count: self.node_count + 2 },
                payload: String::from("membership-change"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "raft-election" {
            self.broadcast(Message {
                from: 1,
                to: 0,
                round: 0,
                msg_type: MessageType::RequestVote {
                    term: 1,
                    candidate_id: 1,
                },
                payload: String::from("request-vote"),
                value: VoteValue::Yes,
                delay_count: 0,
            });
        } else if self.protocol_name == "raft-leader-crash" {
                // First election: node 1 becomes leader in term 1.
                self.broadcast(Message {
                    from: 1,
                    to: 0,
                    round: 0,
                    msg_type: MessageType::RequestVote {
                        term: 1,
                        candidate_id: 1,
                    },
                    payload: String::from("request-vote"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });

                // Simulate crash/re-election trigger:
                // node 2 starts a new election in term 2.
                self.broadcast(Message {
                    from: 2,
                    to: 0,
                    round: 10,
                    msg_type: MessageType::RequestVote {
                        term: 2,
                        candidate_id: 2,
                    },
                    payload: String::from("request-vote"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });
        } else if self.protocol_name == "raft-partition-heal" {
                // First leader election: node 1 becomes leader in term 1.
                self.broadcast(Message {
                    from: 1,
                    to: 0,
                    round: 0,
                    msg_type: MessageType::RequestVote {
                        term: 1,
                        candidate_id: 1,
                    },
                    payload: String::from("request-vote"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });

                // Partition-like transition:
                // majority side elects node 2 in higher term.
                self.broadcast(Message {
                    from: 2,
                    to: 0,
                    round: 10,
                    msg_type: MessageType::RequestVote {
                        term: 2,
                        candidate_id: 2,
                    },
                    payload: String::from("request-vote"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });

                // Heal phase:
                // new leader's heartbeat reaches everyone, including old leader.
                self.broadcast(Message {
                    from: 2,
                    to: 0,
                    round: 20,
                    msg_type: MessageType::AppendEntries {
                        term: 2,
                        leader_id: 2,
                    },
                    payload: String::from("append-entries"),
                    value: VoteValue::Yes,
                    delay_count: 0,
                });
            } else if self.protocol_name == "raft-membership-change" {
                self.broadcast(Message {
                    from: 1,
                    to: 0,
                    round: 0,
                    msg_type: MessageType::RaftConfigChange {
                        term: 1,
                        leader_id: 1,
                        new_node_count: self.node_count + 2,
                    },
                    payload: String::from("raft-config-change"),
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

                let max_steps: u64 = 1000;
                self.metrics.max_steps = max_steps;
                
                while self.metrics.scheduler_steps < max_steps {
                    match self.network.deliver_next() {
                        SchedulerOutcome::Deliver(msg) => {

                            println!(
                                "[step={}] DELIVER from={} to={} type={:?} queue_len={}",
                                self.metrics.scheduler_steps,
                                msg.from,
                                msg.to,
                                msg.msg_type,
                                self.network.queue.len()
                            );

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

                            self.count_message_metrics(&msg);

                            for node in &mut self.nodes {
                                if node.id == msg.to {
                                    

                                      
                                    let actions = self.protocol.handle_message(node, &msg);

                                    for action in actions {
                                        self.apply_action(&msg, action);
                                        

                                        
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

                    if self.metrics.scheduler_steps >= max_steps {
                        self.metrics.reached_step_cap = true;

                        println!(
                            "[SIM] Reached step cap: {}. Treating as non-terminating within measurement bound.",
                            max_steps
                        );
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

            fn apply_action(&mut self, msg: &Message, action: NodeAction){
                        
                match action {


                    NodeAction::BroadcastPrepare { ballot } => {

                        self.metrics.paxos_retries += 1;
                        self.metrics.max_ballot_seen = self.metrics.max_ballot_seen.max(ballot);

                        self.broadcast(Message {
                            from: msg.to,
                            to: 0,
                            round: msg.round + 1,
                            msg_type: MessageType::Prepare { ballot },
                            payload: String::from("prepare"),
                            value: VoteValue::Yes,
                            delay_count: msg.delay_count,
                        });
                    }

                    NodeAction::BroadcastProposal => {
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

                   NodeAction::SendNack {
                        to,
                        ballot,
                        promised_ballot,
                    } => {
                        println!(
                            "[NACK-SEND] from={} to={} ballot={} promised_ballot={}",
                            msg.to,
                            to,
                            ballot,
                            promised_ballot
                        );

                        self.send_to(Message {
                            from: msg.to,
                            to,
                            round: msg.round + 1,
                            msg_type: MessageType::Nack {
                                ballot,
                                promised_ballot,
                            },
                            payload: String::from("nack"),
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

                    NodeAction::RecordChosen { value } => {
                        self.metrics.chosen_values.insert(value);
                    
                        if self.metrics.chosen_values.len() > 1 {
                            self.metrics.safety_violation = true;
                        }
                    }

                    NodeAction::SendMembershipAck { to, new_node_count } => {
                        self.metrics.messages_sent += 1;
                        self.network.send(Message {
                            from: msg.to,
                            to,
                            round: msg.round + 1,
                            msg_type: MessageType::MembershipAck { new_node_count },
                            payload: String::from("membership-ack"),
                            value: VoteValue::Yes,
                            delay_count: msg.delay_count,
                            });
                    }

                    NodeAction::BroadcastMembershipChange { new_node_count } => {
                        self.broadcast(Message {
                            from: msg.to,
                            to: 0,
                            round: msg.round + 1,
                            msg_type: MessageType::MembershipChange { new_node_count },
                            payload: String::from("membership-change"),
                            value: VoteValue::Yes,
                            delay_count: msg.delay_count,
                            });
                    }

                    NodeAction::BroadcastRequestVote { term, candidate_id } => {
                            self.broadcast(Message {
                                from: candidate_id,
                                to: 0,
                                round: 0,
                                msg_type: MessageType::RequestVote {
                                    term,
                                    candidate_id,
                                },
                                payload: String::from("request-vote"),
                                value: VoteValue::Yes,
                                delay_count: 0,
                            });
                    }

                    NodeAction::SendVoteResponse {
                        to,
                        term,
                        vote_granted,
                    } => {
                        self.metrics.messages_sent += 1;
                        self.network.send(Message {
                            from: msg.to,
                            to,
                            round: msg.round + 1,
                            msg_type: MessageType::VoteResponse {
                                term,
                                vote_granted,
                            },
                            payload: String::from("vote-response"),
                            value: VoteValue::Yes,
                            delay_count: msg.delay_count,
                        });
                    }


                    NodeAction::BecomeRaftLeader { leader_id, term } => {
                        self.metrics.raft_leader_elected = true;
                        self.metrics.raft_leader_id = Some(leader_id);
                        self.metrics.raft_election_count += 1;

                        for node in &mut self.nodes {
                            if node.id == leader_id {
                                node.raft_role = RaftRole::Leader;
                                node.raft_current_term = term;
                            } else {
                                node.raft_role = RaftRole::Follower;
                                node.raft_current_term = term;
                            }
                        }

                        self.broadcast(Message {
                            from: leader_id,
                            to: 0,
                            round: 0,
                            msg_type: MessageType::AppendEntries {
                                term,
                                leader_id,
                            },
                            payload: String::from("append-entries"),
                            value: VoteValue::Yes,
                            delay_count: 0,
                        });
                    }

                    NodeAction::BroadcastAppendEntries { term, leader_id } => {
                        self.broadcast(Message {
                            from: leader_id,
                            to: 0,
                            round: 0,
                            msg_type: MessageType::AppendEntries {
                                term,
                                leader_id,
                            },
                            payload: String::from("append-entries"),
                            value: VoteValue::Yes,
                            delay_count: 0,
                        });
                    }

                    NodeAction::SendAppendResponse { to, term, success } => {
                        self.metrics.messages_sent += 1;

                        self.network.send(Message {
                            from: msg.to,
                            to,
                            round: msg.round + 1,
                            msg_type: MessageType::AppendResponse {
                                term,
                                success,
                            },
                            payload: String::from("append-response"),
                            value: VoteValue::Yes,
                            delay_count: msg.delay_count,
                        });
                    }

                    NodeAction::SendRaftConfigAck {
                            to,
                            term,
                            success,
                            new_node_count,
                        } => {
                            self.metrics.messages_sent += 1;

                            self.network.send(Message {
                                from: msg.to,
                                to,
                                round: msg.round + 1,
                                msg_type: MessageType::RaftConfigAck {
                                    term,
                                    success,
                                    new_node_count,
                                },
                                payload: String::from("raft-config-ack"),
                                value: VoteValue::Yes,
                                delay_count: msg.delay_count,
                            });
                        }

                    NodeAction::ActivateRaftConfig { new_node_count: _ } => {
                        self.metrics.raft_config_activated = true;
                    }    
               }
        
            }

    fn count_message_metrics(&mut self, msg: &Message){
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

                    MessageType::Nack { promised_ballot, .. } => {
                        self.metrics.nack_messages += 1;
                        self.metrics.max_ballot_seen =
                            self.metrics.max_ballot_seen.max(*promised_ballot);
                    }

                    MessageType::MembershipChange { .. } => {
                        self.metrics.membership_changes += 1;
                    }

                    MessageType::MembershipAck { .. } =>  {
                    self.metrics.membership_acks += 1;
                    }

                    _ => {}
                }
                match &msg.msg_type {
                    MessageType::RequestVote { .. } => {
                        self.metrics.request_vote_messages += 1;
                    }

                    MessageType::VoteResponse { vote_granted, .. } => {
                        self.metrics.vote_response_messages += 1;

                        if *vote_granted {
                            self.metrics.votes_granted += 1;
                        } else {
                            self.metrics.votes_rejected += 1;
                        }
                    }
                    _ => {}
                }

                match &msg.msg_type {
                    MessageType::AppendEntries { .. } => {
                        self.metrics.append_entries_messages += 1;
                    }

                    MessageType::AppendResponse { success, .. } => {
                        self.metrics.append_response_messages += 1;
                        if *success {
                            self.metrics.heartbeat_successes += 1;
                        } else {
                            self.metrics.heartbeat_rejections += 1;
                        }
                    }

                    MessageType::RaftConfigChange { .. } => {
                        self.metrics.raft_config_changes += 1;
                    }

                    MessageType::RaftConfigAck { .. } => {
                        self.metrics.raft_config_acks += 1;
                    }

                    _ => {}
                }
    }        
}
