use crate::message::{Message, MessageType, VoteValue};
use crate::node::{Node, NodeAction};
use crate::protocol::Protocol;
use std::collections::{HashMap, HashSet};

pub struct BasicPaxosProtocol {
    pub current_ballot: u64,
    pub proposer_id: u64,
    pub promises_by_ballot: HashMap<u64, HashSet<u64>>,
    pub accepted_by_ballot: HashMap<u64, HashSet<u64>>,
    pub highest_accepted_ballot: Option<u64>,
    pub chosen_proposal_value: String,
    pub accept_request_sent_by_ballot: HashSet<u64>,
    pub retry_count: u64,
    pub max_retries: u64,
    
}

impl BasicPaxosProtocol {
    pub fn new() -> Self {
        Self {
            current_ballot: 1,
            proposer_id: 1,

            promises_by_ballot: HashMap::new(),
            accepted_by_ballot: HashMap::new(),
            accept_request_sent_by_ballot: HashSet::new(),

            highest_accepted_ballot: None,
            chosen_proposal_value: "v1".to_string(),

            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn new_with_proposer(
        proposer_id: u64,
        ballot: u64,
        value: String,
    ) -> Self {
        Self {
            current_ballot: ballot,
            proposer_id,

            promises_by_ballot: HashMap::new(),
            accepted_by_ballot: HashMap::new(),

           
            accept_request_sent_by_ballot: HashSet::new(),

            highest_accepted_ballot: None,
            chosen_proposal_value: value,

            retry_count: 0,
            max_retries: 3,
        }
    }
}

impl BasicPaxosProtocol {
    fn retry_with_higher_ballot(&mut self) -> Option<NodeAction> {
        if self.retry_count >= self.max_retries {
            return None;
        }

        self.retry_count += 1;
        self.current_ballot += 1;

        self.promises_by_ballot.clear();
        self.accepted_by_ballot.clear();
        self.accept_request_sent_by_ballot.clear();

        self.highest_accepted_ballot = None;
        self.chosen_proposal_value = "v1".to_string();

        

        Some(NodeAction::BroadcastPrepare {
            ballot: self.current_ballot,
        })
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
                    vec![NodeAction::SendNack {
                        to: msg.from,
                        ballot: *ballot,
                        promised_ballot: node.promised_ballot,
                    }]
                }
            }

            MessageType::AcceptRequest { ballot, value } => {
               
                println!("Broadcasting AcceptRequest({}, {})", ballot, value);
                
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
                    vec![NodeAction::SendNack {
                        to: msg.from,
                        ballot: *ballot,
                        promised_ballot: node.promised_ballot,
                    }]
                }
            }

            MessageType::Promise {
                ballot,
                accepted_ballot,
                accepted_value,
            } => {
                //let proposer_id = 1; //*ballot;
                let proposer_id = self.proposer_id;
            
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

            MessageType::Accepted { ballot, value } => {
                //let proposer_id = 1;//*ballot;

                let proposer_id = self.proposer_id;
            
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
                    return vec![NodeAction::RecordChosen {
                        value: value.clone(),
                    }];
                }
            
                vec![]
            }

            MessageType::Nack {
                ballot: _,
                promised_ballot,
            } => {

               

                if node.id != self.proposer_id {
                    return vec![];
                }
            
                if *promised_ballot >= self.current_ballot {
                    self.current_ballot = promised_ballot + 1;
            
                    return vec![NodeAction::BroadcastPrepare {
                        ballot: self.current_ballot,
                    }];
                }
            
                vec![]
            }

            _ => vec![],
        }
    }
}
