use crate::message::{Message, MessageType};
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
    fn handle_prepare(
        &mut self,
        node: &mut Node,
        msg: &Message,
        ballot: u64,
    ) -> Vec<NodeAction> {
        if ballot < node.promised_ballot {
            return vec![NodeAction::SendNack {
                to: msg.from,
                ballot,
                promised_ballot: node.promised_ballot,
            }];
        }

        node.promised_ballot = ballot;

        let accepted = node
            .accepted_slots
            .values()
            .cloned()
            .collect();

        vec![NodeAction::SendMPPromise {
            to: msg.from,
            ballot,
            accepted,
        }]
    }
}