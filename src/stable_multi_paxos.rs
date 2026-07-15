use crate::message::{AcceptedSlot, Message, MessageType};
use crate::node::{Node, NodeAction};
use crate::protocol::Protocol;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderPhase {
    Preparing,
    Active,
}

pub struct StableMultiPaxos {
    pub leader_id: u64,
    pub ballot: u64,

    pub quorum_size: usize,

    pub phase: LeaderPhase,

    // Who promised this ballot?
    pub promises: HashSet<u64>,

    // slot -> proposed value
    pub proposals: HashMap<u64, String>,

    // slot -> acceptors
    pub accepted: HashMap<u64, HashSet<u64>>,

    // slot -> chosen value
    pub chosen: HashMap<u64, String>,
}

impl StableMultiPaxos {
    pub fn new(quorum_size: usize) -> Self {
        let mut proposals = HashMap::new();

        proposals.insert(1, "v1".to_string());
        proposals.insert(2, "v2".to_string());
        proposals.insert(3, "v3".to_string());

        Self {
            leader_id: 1,
            ballot: 1,

            quorum_size,

            phase: LeaderPhase::Preparing,

            promises: HashSet::new(),

            proposals,

            accepted: HashMap::new(),

            chosen: HashMap::new(),
        }
    }
}

impl StableMultiPaxos {
    fn handle_prepare(&mut self, node: &mut Node, msg: &Message, ballot: u64) -> Vec<NodeAction> {
        if ballot < node.promised_ballot {
            return vec![NodeAction::SendNack {
                to: msg.from,
                ballot,
                promised_ballot: node.promised_ballot,
            }];
        }

        node.promised_ballot = ballot;

        let accepted = node.accepted_slots.values().cloned().collect();

        vec![NodeAction::SendMPPromise {
            to: msg.from,
            ballot,
            accepted,
        }]
    }

    fn handle_promise(
        &mut self,
        node: &Node,
        msg: &Message,
        ballot: u64,
        accepted: &[AcceptedSlot],
    ) -> Vec<NodeAction> {
        if node.id != self.leader_id {
            return vec![];
        }

        if ballot != self.ballot {
            return vec![];
        }

        self.promises.insert(msg.from);

        // Fresh-cluster version:
        // accepted-state recovery will be added later.
        let _ = accepted;

        if self.promises.len() < self.quorum_size {
            return vec![];
        }

        // Phase 1 has already completed.
        if self.phase == LeaderPhase::Active {
            return vec![];
        }

        self.phase = LeaderPhase::Active;

        self.proposals
            .iter()
            .map(|(&slot, value)| NodeAction::BroadcastMPAcceptRequest {
                ballot: self.ballot,
                slot,
                value: value.clone(),
            })
            .collect()
    }

    fn handle_accept_request(
        &mut self,
        node: &mut Node,
        msg: &Message,
        ballot: u64,
        slot: u64,
        value: &str,
    ) -> Vec<NodeAction> {
        if ballot < node.promised_ballot {
            return vec![NodeAction::SendNack {
                to: msg.from,
                ballot,
                promised_ballot: node.promised_ballot,
            }];
        }

        node.promised_ballot = ballot;

        node.accepted_slots.insert(
            slot,
            AcceptedSlot {
                slot,
                ballot,
                value: value.to_string(),
            },
        );

        vec![NodeAction::SendMPAccepted {
            to: msg.from,
            ballot,
            slot,
            value: value.to_string(),
        }]
    }

    fn handle_accepted(
        &mut self,
        node: &Node,
        msg: &Message,
        ballot: u64,
        slot: u64,
        value: &str,
    ) -> Vec<NodeAction> {
        if node.id != self.leader_id {
            return vec![];
        }

        if ballot != self.ballot {
            return vec![];
        }

        let acceptors = self.accepted.entry(slot).or_insert_with(HashSet::new);

        acceptors.insert(msg.from);

        if acceptors.len() < self.quorum_size {
            return vec![];
        }

        // Avoid recording the same slot more than once.
        if self.chosen.contains_key(&slot) {
            return vec![];
        }

        self.chosen.insert(slot, value.to_string());

        vec![NodeAction::RecordMPChosen {
            slot,
            value: value.to_string(),
        }]
    }
}

impl Protocol for StableMultiPaxos {
    fn should_send_initial_proposal(&self, _node_id: usize) -> bool {
        false
    }

    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        match &msg.msg_type {
            MessageType::MPPrepare { ballot } => self.handle_prepare(node, msg, *ballot),

            MessageType::MPPromise { ballot, accepted } => {
                self.handle_promise(node, msg, *ballot, accepted)
            }

            MessageType::MPAcceptRequest {
                ballot,
                slot,
                value,
            } => self.handle_accept_request(node, msg, *ballot, *slot, value),

            MessageType::MPAccepted {
                ballot,
                slot,
                value,
            } => self.handle_accepted(node, msg, *ballot, *slot, value),

            _ => vec![],
        }
    }
}
