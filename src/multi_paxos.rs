use crate::message::{Message, MessageType, VoteValue};
use crate::node::{Node, NodeAction};
use crate::protocol::Protocol;
use std::collections::{HashMap, HashSet};

// Proposer lifecycle:
//
// WaitingForPromises
//   -> quorum Promise
//   -> WaitingForAccepted
//
// WaitingForAccepted
//   -> quorum Accepted
//   -> Decided
//
// WaitingForPromises / WaitingForAccepted
//   -> timeout or higher-ballot NACK
//   -> retry with next owned ballot
//
// Any phase
//   -> max retries exceeded
//   -> Idle

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiPaxosPhase {
    Idle,
    WaitingForPromises,
    WaitingForAccepted,
    Decided,
}

pub struct ProposerState {
    pub proposer_id: u64,
    pub current_ballot: u64,
    pub proposed_value: String,

    pub promises_by_ballot: HashMap<u64, HashSet<u64>>,
    pub accepted_by_ballot: HashMap<u64, HashSet<u64>>,

    pub highest_accepted_ballot: Option<u64>,
    pub chosen_value: String,

    pub accept_request_sent: HashSet<u64>,

    pub retry_count: u64,
    pub phase: MultiPaxosPhase,
    pub phase_elapsed: u64,
}

impl ProposerState {
    pub fn new(proposer_id: u64, ballot: u64, value: String) -> Self {
        Self {
            proposer_id,
            current_ballot: ballot,
            proposed_value: value.clone(),
            promises_by_ballot: HashMap::new(),
            accepted_by_ballot: HashMap::new(),
            highest_accepted_ballot: None,
            chosen_value: value,
            accept_request_sent: HashSet::new(),
            retry_count: 0,
            phase: MultiPaxosPhase::WaitingForPromises,
            phase_elapsed: 0,
        }
    }
}

pub struct MultiPaxosProtocol {
    pub proposers: HashMap<u64, ProposerState>,
    pub quorum_size: usize,
    pub phase_timeout: u64,
    pub max_retries: u64,
}

impl MultiPaxosProtocol {
    pub fn new(quorum_size: usize, phase_timeout: u64) -> Self {
        let mut proposers = HashMap::new();

        proposers.insert(1, ProposerState::new(1, 1, "v1".to_string()));
        proposers.insert(2, ProposerState::new(2, 2, "v2".to_string()));

        proposers.insert(3, ProposerState::new(3, 3, "v3".to_string()));

        Self {
            proposers,
            quorum_size,
            phase_timeout,
            max_retries: 10,
        }
    }

    pub fn start_all_proposers(&mut self) {
        for proposer in self.proposers.values_mut() {
            proposer.phase = MultiPaxosPhase::WaitingForPromises;
            proposer.phase_elapsed = 0;
        }
    }


    fn retry_proposer(
        &mut self,
        proposer_id: u64,
        observed_ballot: Option<u64>,
    ) -> Option<NodeAction> {
        let ballot_stride = self.proposers.len() as u64;
        let proposer = self.proposers.get_mut(&proposer_id)?;

        if proposer.phase == MultiPaxosPhase::Decided {
            return None;
        }

        proposer.retry_count += 1;

        if proposer.retry_count > self.max_retries {
            proposer.phase = MultiPaxosPhase::Idle;
            return None;
        }

        proposer.current_ballot = match observed_ballot {
            Some(promised) => {
                let mut next = proposer.current_ballot + ballot_stride;

                while next <= promised {
                    next += ballot_stride;
                }

                next
            }
            None => proposer.current_ballot + ballot_stride,
        };

        proposer.promises_by_ballot.clear();
        proposer.accepted_by_ballot.clear();
        proposer.accept_request_sent.clear();
        proposer.highest_accepted_ballot = None;
        proposer.chosen_value = proposer.proposed_value.clone();
        proposer.phase = MultiPaxosPhase::WaitingForPromises;
        proposer.phase_elapsed = 0;

        Some(NodeAction::BroadcastPrepareFrom {
            from: proposer.proposer_id,
            ballot: proposer.current_ballot,
        })
    }
}

impl Protocol for MultiPaxosProtocol {
    fn uses_timeout(&self) -> bool {
        true
    }

    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        match &msg.msg_type {
            MessageType::Prepare { ballot } => {
                if *ballot >= node.promised_ballot {
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
                let proposer = match self.proposers.get_mut(&msg.to) {
                    Some(p) => p,
                    None => return vec![],
                };

                println!(
                    "PROMISE proposer={} msg.to={} ballot={} current_ballot={}",
                    proposer.proposer_id,
                    msg.to,
                    ballot,
                    proposer.current_ballot
                );

                if *ballot != proposer.current_ballot {
                    return vec![];
                }

                proposer
                    .promises_by_ballot
                    .entry(*ballot)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);

                if let (Some(ab), Some(av)) = (accepted_ballot, accepted_value) {
                    if proposer.highest_accepted_ballot.is_none()
                        || *ab > proposer.highest_accepted_ballot.unwrap()
                    {
                        proposer.highest_accepted_ballot = Some(*ab);
                        proposer.chosen_value = av.clone();
                    }
                }

                let promise_count = proposer
                    .promises_by_ballot
                    .get(ballot)
                    .map(|s| s.len())
                    .unwrap_or(0);

                    println!(
                        "ballot={} promise_count={}",
                        ballot,
                        promise_count
                    );


                    println!(
                        "quorum={} accept_sent={}",
                        self.quorum_size,
                        proposer.accept_request_sent.contains(ballot)
                    );

                if promise_count >= self.quorum_size
                    && !proposer.accept_request_sent.contains(ballot)
                {
                    proposer.accept_request_sent.insert(*ballot);
                    proposer.phase = MultiPaxosPhase::WaitingForAccepted;
                    proposer.phase_elapsed = 0;

                    let value = proposer.chosen_value.clone();

                    return vec![NodeAction::BroadcastAcceptRequest {
                        ballot: *ballot,
                        value,
                    }];
                }

                vec![]
            }

            MessageType::Accepted { ballot, value } => {
                let proposer = match self.proposers.get_mut(&msg.to) {
                    Some(p) => p,
                    None => return vec![],
                };

                if *ballot != proposer.current_ballot {
                    return vec![];
                }

                proposer
                    .accepted_by_ballot
                    .entry(*ballot)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);

                let accepted_count = proposer
                    .accepted_by_ballot
                    .get(ballot)
                    .map(|s| s.len())
                    .unwrap_or(0);

                if accepted_count >= self.quorum_size && node.decided.is_none() {
                    node.decided = Some(VoteValue::Yes);
                    proposer.phase = MultiPaxosPhase::Decided;

                    return vec![NodeAction::RecordChosen {
                        value: value.clone(),
                    }];
                }

                vec![]
            }

            MessageType::Nack {
                ballot,
                promised_ballot,
            } => {
                let proposer = match self.proposers.get(&msg.to) {
                    Some(p) => p,
                    None => return vec![],
                };

                if *ballot != proposer.current_ballot {
                    return vec![];
                }

                if *promised_ballot < proposer.current_ballot {
                    return vec![];
                }

                self.retry_proposer(msg.to, Some(*promised_ballot))
                    .into_iter()
                    .collect()
            }

            _ => vec![],
        }
    }

    fn on_tick(&mut self) -> Vec<NodeAction> {

        if self
            .proposers
            .values()
            .any(|p| p.phase == MultiPaxosPhase::Decided)
        {
            return vec![];
        }

        

        let mut actions = Vec::new();

        let proposer_ids: Vec<u64> = self.proposers.keys().copied().collect();
        

        for proposer_id in proposer_ids {
            let should_retry = {
                let proposer = self.proposers.get_mut(&proposer_id).unwrap();

                match proposer.phase {
                    MultiPaxosPhase::WaitingForPromises
                    | MultiPaxosPhase::WaitingForAccepted => {
                        proposer.phase_elapsed += 1;
                        proposer.phase_elapsed >= self.phase_timeout
                    }

                    _ => false,
                }
            };

            if should_retry {
                
                println!(
                    "[MULTI-PAXOS-TIMEOUT] proposer={} ballot={} phase={:?}",
                    proposer_id,
                    self.proposers[&proposer_id].current_ballot,
                    self.proposers[&proposer_id].phase,
                );

                if let Some(action) = self.retry_proposer(proposer_id, None) {
                    actions.push(action);
                }
            }
        }

        actions
    }

}