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

    /// PREPARE
    ///
    /// - Reject ballots lower than the acceptor's promised ballot.
    /// - Otherwise update the promised ballot.
    /// - Return the acceptor's previously accepted ballot/value.
    fn handle_prepare(&mut self, node: &mut Node, msg: &Message, ballot: u64) -> Vec<NodeAction> {
        if ballot >= node.promised_ballot {
            node.promised_ballot = ballot;

            vec![NodeAction::SendPromise {
                to: msg.from,
                ballot,
                accepted_ballot: node.accepted_ballot,
                accepted_value: node.accepted_value.clone(),
            }]
        } else {
            vec![NodeAction::SendNack {
                to: msg.from,
                ballot,
                promised_ballot: node.promised_ballot,
            }]
        }
    }

    /// PROMISE
    ///
    /// - Count Promises only for the proposer's current ballot.
    /// - Ignore duplicate Promises from the same acceptor.
    /// - Adopt the value associated with the highest accepted ballot.
    /// - Broadcast AcceptRequest once Promise quorum is reached.
    fn handle_promise(
        &mut self,
        msg: &Message,
        ballot: u64,
        accepted_ballot: Option<u64>,
        accepted_value: Option<String>,
    ) -> Vec<NodeAction> {
        let quorum_size = self.quorum_size;

        let proposer = match self.proposers.get_mut(&msg.to) {
            Some(proposer) => proposer,
            None => return vec![],
        };

        if ballot != proposer.current_ballot {
            return vec![];
        }

        proposer
            .promises_by_ballot
            .entry(ballot)
            .or_insert_with(HashSet::new)
            .insert(msg.from);

        // Paxos value-adoption rule:
        // adopt the value associated with the highest accepted ballot
        // reported by the Promise quorum.
        if let (Some(previous_ballot), Some(previous_value)) = (accepted_ballot, accepted_value) {
            let should_adopt = proposer
                .highest_accepted_ballot
                .map(|highest| previous_ballot > highest)
                .unwrap_or(true);

            if should_adopt {
                proposer.highest_accepted_ballot = Some(previous_ballot);
                proposer.chosen_value = previous_value;
            }
        }

        let promise_count = proposer
            .promises_by_ballot
            .get(&ballot)
            .map(HashSet::len)
            .unwrap_or(0);

        println!(
            "[MULTI-PAXOS-PROMISE] proposer={} ballot={} count={} quorum={}",
            proposer.proposer_id, ballot, promise_count, quorum_size
        );

        if promise_count < quorum_size {
            return vec![];
        }

        // Prevent duplicate AcceptRequest broadcasts if additional Promises
        // arrive after quorum has already been reached.
        if !proposer.accept_request_sent.insert(ballot) {
            return vec![];
        }

        proposer.phase = MultiPaxosPhase::WaitingForAccepted;
        proposer.phase_elapsed = 0;

        vec![NodeAction::BroadcastAcceptRequest {
            ballot,
            value: proposer.chosen_value.clone(),
        }]
    }

    /// ACCEPT REQUEST
    ///
    /// - Reject ballots below the acceptor's promised ballot.
    /// - Otherwise promise the ballot and accept the proposed value.
    /// - Reply Accepted to the requesting proposer.
    fn handle_accept_request(
        &mut self,
        node: &mut Node,
        msg: &Message,
        ballot: u64,
        value: String,
    ) -> Vec<NodeAction> {
        if ballot >= node.promised_ballot {
            node.promised_ballot = ballot;
            node.accepted_ballot = Some(ballot);
            node.accepted_value = Some(value.clone());

            vec![NodeAction::SendAccepted {
                to: msg.from,
                ballot,
                value,
            }]
        } else {
            vec![NodeAction::SendNack {
                to: msg.from,
                ballot,
                promised_ballot: node.promised_ballot,
            }]
        }
    }

    /// ACCEPTED
    ///
    /// - Count Accepted messages only for the proposer's current ballot.
    /// - Ignore duplicate acknowledgements from the same acceptor.
    /// - Mark the value chosen after quorum.
    fn handle_accepted(
        &mut self,
        node: &mut Node,
        msg: &Message,
        ballot: u64,
        value: String,
    ) -> Vec<NodeAction> {
        let quorum_size = self.quorum_size;

        let proposer = match self.proposers.get_mut(&msg.to) {
            Some(proposer) => proposer,
            None => return vec![],
        };

        if ballot != proposer.current_ballot {
            return vec![];
        }

        proposer
            .accepted_by_ballot
            .entry(ballot)
            .or_insert_with(HashSet::new)
            .insert(msg.from);

        let accepted_count = proposer
            .accepted_by_ballot
            .get(&ballot)
            .map(HashSet::len)
            .unwrap_or(0);

        println!(
            "[MULTI-PAXOS-ACCEPTED] proposer={} ballot={} count={} quorum={}",
            proposer.proposer_id, ballot, accepted_count, quorum_size
        );

        if accepted_count < quorum_size {
            return vec![];
        }

        if proposer.phase == MultiPaxosPhase::Decided {
            return vec![];
        }

        proposer.phase = MultiPaxosPhase::Decided;
        proposer.phase_elapsed = 0;

        if node.decided.is_none() {
            node.decided = Some(VoteValue::Yes);
        }

        vec![NodeAction::RecordChosen { value }]
    }

    /// NACK
    ///
    /// - Ignore NACKs for stale proposer ballots.
    /// - Ignore NACKs that do not reveal a higher promised ballot.
    /// - Retry using the next ballot owned by this proposer that is strictly
    ///   greater than the observed promised ballot.
    fn handle_nack(&mut self, msg: &Message, ballot: u64, promised_ballot: u64) -> Vec<NodeAction> {
        let current_ballot = match self.proposers.get(&msg.to) {
            Some(proposer) => proposer.current_ballot,
            None => return vec![],
        };

        if ballot != current_ballot {
            return vec![];
        }

        if promised_ballot < current_ballot {
            return vec![];
        }

        println!(
            "[MULTI-PAXOS-NACK] proposer={} ballot={} promised_ballot={}",
            msg.to, ballot, promised_ballot
        );

        self.retry_proposer(msg.to, Some(promised_ballot))
            .into_iter()
            .collect()
    }
}

impl Protocol for MultiPaxosProtocol {
    fn uses_timeout(&self) -> bool {
        true
    }

    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        match &msg.msg_type {
            MessageType::Prepare { ballot } => self.handle_prepare(node, msg, *ballot),

            MessageType::Promise {
                ballot,
                accepted_ballot,
                accepted_value,
            } => self.handle_promise(msg, *ballot, *accepted_ballot, accepted_value.clone()),

            MessageType::AcceptRequest { ballot, value } => {
                self.handle_accept_request(node, msg, *ballot, value.clone())
            }

            MessageType::Accepted { ballot, value } => {
                self.handle_accepted(node, msg, *ballot, value.clone())
            }

            MessageType::Nack {
                ballot,
                promised_ballot,
            } => self.handle_nack(msg, *ballot, *promised_ballot),

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
                    MultiPaxosPhase::WaitingForPromises | MultiPaxosPhase::WaitingForAccepted => {
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
