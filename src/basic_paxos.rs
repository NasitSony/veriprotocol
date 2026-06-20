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
    pub highest_accepted_ballot: Option<u64>,
    pub chosen_proposal_value: String,
}

impl BasicPaxosProtocol {
    pub fn new() -> Self {
        Self {
            ballot: 1,
            value: "v1".to_string(),
            promises: HashSet::new(),
            accepted: HashSet::new(),
            accept_request_sent: false,
            highest_accepted_ballot: None,
            chosen_proposal_value: "v1".to_string(),
        }
    }
}

impl Protocol for BasicPaxosProtocol {
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        match &msg.msg_type {
            MessageType::Prepare { ballot } => {
               
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
               
                
                if *ballot >= node.promised_ballot {
                    node.promised_ballot = *ballot;
                    node.accepted_ballot = Some(*ballot);
                    node.accepted_value = Some(value.clone());

                    vec![NodeAction::SendAccepted {
                        to: msg.from,
                        ballot: *ballot,
                        value: self.chosen_proposal_value.clone(),
                    }]
                } else {
                    vec![]
                }
            }

            MessageType::Promise {
                ballot,
                accepted_ballot,
                accepted_value,
            } => {
                if node.id != 1 {
                    return vec![];
                }
            
                if *ballot == self.ballot {
                    self.promises.insert(msg.from);
            
                    if let (Some(ab), Some(av)) = (accepted_ballot, accepted_value) {
                        if self.highest_accepted_ballot.is_none()
                            || *ab > self.highest_accepted_ballot.unwrap()
                        {
                            self.highest_accepted_ballot = Some(*ab);
                            self.chosen_proposal_value = av.clone();
                        }
                    }
            
                    if self.promises.len() >= 3 && !self.accept_request_sent {
                        self.accept_request_sent = true;
            
                        return vec![NodeAction::BroadcastAcceptRequest {
                            ballot: self.ballot,
                            value: self.chosen_proposal_value.clone(),
                        }];
                    }
                }
            
                vec![]
            }

            MessageType::Accepted { ballot, value } => {
                
                if node.id != 1 {
                    return vec![];
                }

                if *ballot == self.ballot {
                    self.accepted.insert(msg.from);

                    if self.accepted.len() >= 3 && node.decided.is_none() {
                        node.decided = Some(VoteValue::Yes);
                    }
                }
                vec![]
            }

            _ => vec![],
        }
    }
}
