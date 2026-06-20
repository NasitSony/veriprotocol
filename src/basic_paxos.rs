use crate::message::{Message, MessageType, VoteValue};
use crate::node::{Node, NodeAction};
use crate::protocol::Protocol;
use std::collections::{HashMap, HashSet};

pub struct BasicPaxosProtocol {
    pub ballot: u64,
    pub value: String,
    pub promises_by_ballot: HashMap<u64, HashSet<u64>>,
    pub accepted_by_ballot: HashMap<u64, HashSet<u64>>,
    pub accept_request_sent: bool,
    pub highest_accepted_ballot: Option<u64>,
    pub chosen_proposal_value: String,
    pub accept_request_sent_by_ballot: HashSet<u64>,
}

impl BasicPaxosProtocol {
    pub fn new() -> Self {
        Self {
            ballot: 1,
            value: "v1".to_string(),
            promises_by_ballot: HashMap::new(),
            accepted_by_ballot: HashMap::new(),
            accept_request_sent: false,
            highest_accepted_ballot: None,
            chosen_proposal_value: "v1".to_string(),
            accept_request_sent_by_ballot: HashSet::new(),
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
                let proposer_id = *ballot;
            
                if node.id != proposer_id {
                    return vec![];
                }
            
                self.promises_by_ballot
                    .entry(*ballot)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);
            
                if let (Some(ab), Some(av)) = (accepted_ballot, accepted_value) {
                    if self.highest_accepted_ballot.is_none()
                        || *ab > self.highest_accepted_ballot.unwrap()
                    {
                        self.highest_accepted_ballot = Some(*ab);
                        self.chosen_proposal_value = av.clone();
                    }
                }
            
                let promise_count = self
                    .promises_by_ballot
                    .get(ballot)
                    .map(|s| s.len())
                    .unwrap_or(0);
            
                if promise_count >= 3 && !self.accept_request_sent_by_ballot.contains(ballot) {
                    self.accept_request_sent_by_ballot.insert(*ballot);
                    
                    return vec![NodeAction::BroadcastAcceptRequest {
                        ballot: *ballot,
                        value: self.chosen_proposal_value.clone(),
                    }];
                }
            
                vec![]
            }

            MessageType::Accepted { ballot, value: _ } => {
                let proposer_id = *ballot;
            
                if node.id != proposer_id {
                    return vec![];
                }
            
                self.accepted_by_ballot
                    .entry(*ballot)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);
            
                let accepted_count = self
                    .accepted_by_ballot
                    .get(ballot)
                    .map(|s| s.len())
                    .unwrap_or(0);
            
                if accepted_count >= 3 && node.decided.is_none() {
                    node.decided = Some(VoteValue::Yes);
                }
            
                vec![]
            }

            _ => vec![],
        }
    }
}
