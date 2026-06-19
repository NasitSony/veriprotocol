// src/basic_paxos.rs
use std::collections::HashSet;

use crate::message::{Message, MessageType, VoteValue};
use crate::node::{Node, NodeAction};
use crate::protocol::Protocol;

pub struct BasicPaxosProtocol {
    pub ballot: u64,
    pub value: String,
    pub promises: HashSet<u64>,
    pub accepted: HashSet<u64>,
    pub accept_request_sent: bool,
}

impl BasicPaxosProtocol {
    pub fn new() -> Self {
        Self {
            ballot: 1,
            value: "v1".to_string(),
            promises: HashSet::new(),
            accepted: HashSet::new(),
            accept_request_sent: false,
        }
    }
}

impl Protocol for BasicPaxosProtocol {
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        match &msg.msg_type {
            MessageType::Prepare { ballot } => {
                println!(
                    "Node {} received Prepare({}) from {}",
                    node.id, ballot, msg.from
                );
                if *ballot > node.promised_ballot {
                    node.promised_ballot = *ballot;

                    vec![NodeAction::SendPromise {
                        to: msg.from,
                        ballot: *ballot,
                        accepted_ballot: node.accepted_ballot,
                        accepted_value: node.accepted_value.clone(),
                    }]
                } else {
                    vec![]
                }
            }

            MessageType::AcceptRequest { ballot, value } => {
                println!(
                    "Node {} received Promise({}) from {}",
                    node.id, ballot, msg.from
                );
                if *ballot >= node.promised_ballot {
                    node.promised_ballot = *ballot;
                    node.accepted_ballot = Some(*ballot);
                    node.accepted_value = Some(value.clone());

                    vec![NodeAction::SendAccepted {
                        to: msg.from,
                        ballot: *ballot,
                        value: value.clone(),
                    }]
                } else {
                    vec![]
                }
            }

            MessageType::Promise { ballot, .. } => {
                println!(
                    "Node {} received AcceptRequest({}) from {}",
                    node.id, ballot, msg.from
                );
                if node.id != 1 {
                    return vec![];
                }

                if *ballot == self.ballot {
                    self.promises.insert(msg.from);

                    if self.promises.len() >= 3 && !self.accept_request_sent {
                        self.accept_request_sent = true;

                        return vec![NodeAction::BroadcastAcceptRequest {
                            ballot: self.ballot,
                            value: self.value.clone(),
                        }];
                    }
                }

                vec![]
            }

            MessageType::Accepted { ballot, value } => {
                println!(
                    "Node {} received Accepted({}, {}) from {}",
                    node.id, ballot, value, msg.from
                );
                if node.id != 1 {
                    return vec![];
                }

                if *ballot == self.ballot {
                    self.accepted.insert(msg.from);

                    if self.accepted.len() >= 3 && node.decided.is_none() {
                        node.decided = Some(VoteValue::Yes);
                        println!("Paxos chosen value: {}", value);
                    }
                }
                vec![]
            }

            _ => vec![],
        }
    }
}
