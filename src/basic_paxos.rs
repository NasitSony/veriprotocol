use crate::message::{Message, MessageType, VoteValue};
use crate::node::{Node, NodeAction};
use crate::protocol::Protocol;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaxosPhase {
    Idle,
    WaitingForPromises,
    WaitingForAccepted,
    Decided,
}

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
    pub proposed_values_by_ballot: HashMap<u64, String>,
    pub quorum_size: usize,
    pub highest_nack_seen: u64,

    pub phase: PaxosPhase,
    pub phase_elapsed: u64,
    pub phase_timeout: u64,
}

impl BasicPaxosProtocol {
    pub fn new() -> Self {
        let mut proposed_values_by_ballot = HashMap::new();
        proposed_values_by_ballot.insert(1, "v1".to_string());

        Self {
            current_ballot: 1,
            proposer_id: 1,

            promises_by_ballot: HashMap::new(),
            accepted_by_ballot: HashMap::new(),
            accept_request_sent_by_ballot: HashSet::new(),

            highest_accepted_ballot: None,
            chosen_proposal_value: "v1".to_string(),

            retry_count: 0,
            max_retries: 20,

            proposed_values_by_ballot,
            quorum_size: 3,
            highest_nack_seen: 0,

            phase: PaxosPhase::Idle,
            phase_elapsed: 0,
            phase_timeout: 15,
        }
    }

    pub fn new_with_proposer(proposer_id: u64, ballot: u64, value: String) -> Self {
        let mut proposed_values_by_ballot = HashMap::new();
        proposed_values_by_ballot.insert(1, "v1".to_string());

        Self {
            current_ballot: ballot,
            proposer_id,

            promises_by_ballot: HashMap::new(),
            accepted_by_ballot: HashMap::new(),

            accept_request_sent_by_ballot: HashSet::new(),

            highest_accepted_ballot: None,
            chosen_proposal_value: value,

            retry_count: 0,
            max_retries: 20,

            proposed_values_by_ballot,
            quorum_size: 3,
            highest_nack_seen: 0,

            phase: PaxosPhase::Idle,
            phase_elapsed: 0,
            phase_timeout: 15,
        }
    }
}

impl BasicPaxosProtocol {
    fn quorum_size(&self) -> usize {
        3 // TODO: dynamic quorum
    }

    pub fn enter_phase(&mut self, phase: PaxosPhase) {
        self.phase = phase;
        self.phase_elapsed = 0;
    }

    pub fn with_phase_timeout(mut self, timeout: u64) -> Self {
        self.phase_timeout = timeout;
        self
    }

    fn promise_count(&self) -> usize {
        self.promises_by_ballot
            .get(&self.current_ballot)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    fn accepted_count(&self) -> usize {
        self.accepted_by_ballot
            .get(&self.current_ballot)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    pub fn with_phase(mut self, phase: PaxosPhase) -> Self {
        self.phase = phase;
        self.phase_elapsed = 0;
        self
    }

    pub fn on_tick(&mut self) -> Vec<NodeAction> {
        match self.phase {
            PaxosPhase::WaitingForPromises => {
                if self.promise_count() >= self.quorum_size() {
                    return vec![];
                }

                println!(
                    "[PAXOS-TICK] phase={:?} elapsed={} timeout={} ballot={} promises={} accepted={}",
                    self.phase,
                    self.phase_elapsed,
                    self.phase_timeout,
                    self.current_ballot,
                    self.promise_count(),
                    self.accepted_count()
                );

                self.phase_elapsed += 1;

                if self.phase_elapsed >= self.phase_timeout {
                    return self
                        .on_timeout()
                        .map(|action| vec![action])
                        .unwrap_or_default();
                }

                vec![]
            }

            PaxosPhase::WaitingForAccepted => {
                if self.accepted_count() >= self.quorum_size() {
                    return vec![];
                }

                self.phase_elapsed += 1;

                if self.phase_elapsed >= self.phase_timeout {
                    return self
                        .on_timeout()
                        .map(|action| vec![action])
                        .unwrap_or_default();
                }

                vec![]
            }

            PaxosPhase::Idle | PaxosPhase::Decided => vec![],
        }
    }

    pub fn with_quorum_size(mut self, quorum_size: usize) -> Self {
        self.quorum_size = quorum_size;
        self
    }

    fn retry_with_higher_ballot(&mut self) -> Option<NodeAction> {
        if self.phase == PaxosPhase::Decided {
            return None;
        }

        if self.retry_count >= self.max_retries {
            self.phase = PaxosPhase::Idle;
            return None;
        }

        self.retry_count += 1;
        self.current_ballot += 1;

        self.promises_by_ballot.clear();
        self.accepted_by_ballot.clear();
        self.accept_request_sent_by_ballot.clear();

        self.highest_accepted_ballot = None;
        self.chosen_proposal_value = "v1".to_string();

        self.enter_phase(PaxosPhase::WaitingForPromises);

        Some(NodeAction::BroadcastPrepare {
            ballot: self.current_ballot,
        })
    }

    fn value_for_ballot(&self, ballot: u64) -> String {
        self.proposed_values_by_ballot
            .get(&ballot)
            .cloned()
            .unwrap_or_else(|| self.chosen_proposal_value.clone())
    }

    pub fn with_ballot_value(mut self, ballot: u64, value: &str) -> Self {
        self.proposed_values_by_ballot
            .insert(ballot, value.to_string());
        self
    }

    pub fn on_timeout(&mut self) -> Option<NodeAction> {
        self.retry_with_higher_ballot()
    }
}

impl Protocol for BasicPaxosProtocol {
    fn uses_timeout(&self) -> bool {
        true
    }

    fn on_tick(&mut self) -> Vec<NodeAction> {
        self.on_tick()
    }

    fn on_timeout(&mut self) -> Vec<NodeAction> {
        if let Some(action) = self.retry_with_higher_ballot() {
            vec![action]
        } else {
            vec![]
        }
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
                    println!(
                        "[NACK-CAUSE] node={} incoming={:?} incoming_ballot={} promised_ballot={}",
                        node.id, msg.msg_type, ballot, node.promised_ballot
                    );

                    println!(
                        "[RETRY-MOTIF] cause=stale_prepare node={} incoming_ballot={} promised_ballot={} gap={}",
                        node.id,
                        ballot,
                        node.promised_ballot,
                        node.promised_ballot.saturating_sub(*ballot),
                    );
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
                        value: value.clone(),
                    }]
                } else {
                    println!(
                        "[NACK-CAUSE] node={} incoming={:?} incoming_ballot={} promised_ballot={}",
                        node.id, msg.msg_type, ballot, node.promised_ballot
                    );

                    println!(
                        "[RETRY-MOTIF] cause=stale_accept_request node={} incoming_ballot={} promised_ballot={} gap={}",
                        node.id,
                        ballot,
                        node.promised_ballot,
                        node.promised_ballot.saturating_sub(*ballot),
                    );
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

                if *ballot != self.current_ballot {
                    return vec![];
                }

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

                if promise_count >= self.quorum_size
                    && !self.accept_request_sent_by_ballot.contains(ballot)
                    && *ballot == self.current_ballot
                {
                    self.accept_request_sent_by_ballot.insert(*ballot);

                    self.enter_phase(PaxosPhase::WaitingForAccepted);

                    let proposed_value = if self.highest_accepted_ballot.is_some() {
                        self.chosen_proposal_value.clone()
                    } else {
                        self.value_for_ballot(*ballot)
                    };

                    println!(
                        "[PAXOS-QUORUM-PROMISE] ballot={} count={} quorum={} current={}",
                        ballot, promise_count, self.quorum_size, self.current_ballot
                    );

                    return vec![NodeAction::BroadcastAcceptRequest {
                        ballot: *ballot,
                        value: proposed_value,
                    }];
                }

                vec![]
            }

            MessageType::Accepted { ballot, value } => {
                //let proposer_id = 1;//*ballot;

                if *ballot != self.current_ballot {
                    return vec![];
                }

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

                if accepted_count >= self.quorum_size && node.decided.is_none() {
                    node.decided = Some(VoteValue::Yes);
                    self.enter_phase(PaxosPhase::Decided);
                    println!(
                        "[PAXOS-QUORUM-ACCEPTED] ballot={} count={} quorum={} current={}",
                        ballot, accepted_count, self.quorum_size, self.current_ballot
                    );

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
                /*println!(
                    "NACK: promised={}, current={}",
                    promised_ballot,
                    self.current_ballot
                );*/

                println!(
                    "[PAXOS-NACK] from={} to={} promised={} current={}",
                    msg.from, msg.to, promised_ballot, self.current_ballot
                );

                if node.id != self.proposer_id {
                    return vec![];
                }

                if *ballot != self.current_ballot {
                    println!(
                        "[PAXOS-NACK-IGNORED] stale_nack ballot={} current={} promised={}",
                        ballot, self.current_ballot, promised_ballot
                    );
                    return vec![];
                }

                if *promised_ballot > self.highest_nack_seen
                    && *promised_ballot >= self.current_ballot
                {
                    self.highest_nack_seen = *promised_ballot;
                    self.current_ballot = promised_ballot + 1;

                    println!("[PAXOS-RETRY] new_ballot={}", self.current_ballot);

                    return vec![NodeAction::BroadcastPrepare {
                        ballot: self.current_ballot,
                    }];
                }

                vec![]
            }

            MessageType::MembershipChange { new_node_count } => {
                vec![NodeAction::SendMembershipAck {
                    to: msg.from,
                    new_node_count: *new_node_count,
                }]
            }

            MessageType::MembershipAck { new_node_count: _ } => {
                vec![]
            }

            MessageType::Timeout => {
                return self
                    .on_timeout()
                    .map(|action| vec![action])
                    .unwrap_or_default();
            }

            _ => vec![],
        }
    }
}
