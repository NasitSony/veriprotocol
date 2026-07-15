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

    // slot -> highest-ballot accepted value learned during Phase 1
    pub recovered: HashMap<u64, AcceptedSlot>,
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

            recovered: HashMap::new(),
        }
    }
}

impl StableMultiPaxos {
    fn record_recovered_slots(&mut self, accepted_slots: &[AcceptedSlot]) {
        for accepted in accepted_slots {
            let should_replace = self
                .recovered
                .get(&accepted.slot)
                .map(|current| accepted.ballot > current.ballot)
                .unwrap_or(true);

            if should_replace {
                self.recovered.insert(accepted.slot, accepted.clone());
            }
        }
    }

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
        // Only the intended leader processes Promise responses.
        if node.id != self.leader_id {
            return vec![];
        }

        // Ignore stale or unrelated ballots.
        if ballot != self.ballot {
            return vec![];
        }

        // Ignore duplicate Promise messages from the same acceptor.
        if !self.promises.insert(msg.from) {
            return vec![];
        }

        // Learn the highest-ballot accepted value for every reported slot.
        self.record_recovered_slots(accepted);

        if self.promises.len() < self.quorum_size {
            return vec![];
        }

        // Phase 1 has already completed.
        if self.phase == LeaderPhase::Active {
            return vec![];
        }

        self.phase = LeaderPhase::Active;

        self.build_phase2_actions()
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

    fn build_phase2_actions(&mut self) -> Vec<NodeAction> {
        let mut phase2_values = self.proposals.clone();

        // Recovered values must override fresh client values.
        for (&slot, recovered) in &self.recovered {
            phase2_values.insert(slot, recovered.value.clone());
        }

        self.proposals = phase2_values.clone();

        let mut slots: Vec<u64> = phase2_values.keys().copied().collect();
        slots.sort_unstable();

        slots
            .into_iter()
            .map(|slot| {
                let value = phase2_values
                    .get(&slot)
                    .expect("slot must exist in phase2_values")
                    .clone();

                NodeAction::BroadcastMPAcceptRequest {
                    ballot: self.ballot,
                    slot,
                    value,
                }
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::VoteValue;

    #[test]
    fn recovery_keeps_highest_ballot_per_slot() {
        let mut protocol = StableMultiPaxos::new(3);

        protocol.record_recovered_slots(&[
            AcceptedSlot {
                slot: 1,
                ballot: 2,
                value: "old".to_string(),
            },
            AcceptedSlot {
                slot: 1,
                ballot: 5,
                value: "new".to_string(),
            },
            AcceptedSlot {
                slot: 2,
                ballot: 3,
                value: "slot-2-value".to_string(),
            },
        ]);

        assert_eq!(protocol.recovered[&1].ballot, 5);
        assert_eq!(protocol.recovered[&1].value, "new");

        assert_eq!(protocol.recovered[&2].ballot, 3);
        assert_eq!(protocol.recovered[&2].value, "slot-2-value");
    }

    #[test]
    fn lower_ballot_does_not_replace_higher_ballot() {
        let mut protocol = StableMultiPaxos::new(3);

        protocol.record_recovered_slots(&[AcceptedSlot {
            slot: 1,
            ballot: 8,
            value: "safe-value".to_string(),
        }]);

        protocol.record_recovered_slots(&[AcceptedSlot {
            slot: 1,
            ballot: 4,
            value: "stale-value".to_string(),
        }]);

        assert_eq!(protocol.recovered[&1].ballot, 8);
        assert_eq!(protocol.recovered[&1].value, "safe-value");
    }

    fn mp_accept_value(actions: &[NodeAction], expected_slot: u64) -> Option<String> {
        actions.iter().find_map(|action| match action {
            NodeAction::BroadcastMPAcceptRequest { slot, value, .. } if *slot == expected_slot => {
                Some(value.clone())
            }

            _ => None,
        })
    }

    #[test]
    fn recovered_value_overrides_fresh_proposal() {
        let mut protocol = StableMultiPaxos::new(3);

        // The new leader originally intends to propose v1 for slot 1.
        assert_eq!(protocol.proposals.get(&1).unwrap(), "v1");

        let mut leader_node = Node::new(1);

        let promise_1 = Message {
            from: 2,
            to: 1,
            round: 0,
            msg_type: MessageType::MPPromise {
                ballot: 1,
                accepted: vec![AcceptedSlot {
                    slot: 1,
                    ballot: 4,
                    value: "previously-accepted".to_string(),
                }],
            },
            payload: String::new(),
            value: VoteValue::Yes,
            delay_count: 0,
        };

        let promise_2 = Message {
            from: 3,
            to: 1,
            round: 0,
            msg_type: MessageType::MPPromise {
                ballot: 1,
                accepted: vec![],
            },
            payload: String::new(),
            value: VoteValue::Yes,
            delay_count: 0,
        };

        let promise_3 = Message {
            from: 4,
            to: 1,
            round: 0,
            msg_type: MessageType::MPPromise {
                ballot: 1,
                accepted: vec![],
            },
            payload: String::new(),
            value: VoteValue::Yes,
            delay_count: 0,
        };

        assert!(
            protocol
                .handle_message(&mut leader_node, &promise_1)
                .is_empty()
        );

        assert!(
            protocol
                .handle_message(&mut leader_node, &promise_2)
                .is_empty()
        );

        let actions = protocol.handle_message(&mut leader_node, &promise_3);

        assert_eq!(protocol.phase, LeaderPhase::Active);

        assert_eq!(
            mp_accept_value(&actions, 1).as_deref(),
            Some("previously-accepted")
        );

        // Unrecovered slots still use their fresh values.
        assert_eq!(mp_accept_value(&actions, 2).as_deref(), Some("v2"));

        assert_eq!(mp_accept_value(&actions, 3).as_deref(), Some("v3"));

        // The leader's authoritative proposal map is updated too.
        assert_eq!(
            protocol.proposals.get(&1).map(String::as_str),
            Some("previously-accepted")
        );
    }

    #[test]
    fn phase1_selects_highest_accepted_ballot_across_promises() {
        let mut protocol = StableMultiPaxos::new(3);
        let mut leader_node = Node::new(1);

        let promises = vec![
            Message {
                from: 2,
                to: 1,
                round: 0,
                msg_type: MessageType::MPPromise {
                    ballot: 1,
                    accepted: vec![AcceptedSlot {
                        slot: 1,
                        ballot: 2,
                        value: "older-value".to_string(),
                    }],
                },
                payload: String::new(),
                value: VoteValue::Yes,
                delay_count: 0,
            },
            Message {
                from: 3,
                to: 1,
                round: 0,
                msg_type: MessageType::MPPromise {
                    ballot: 1,
                    accepted: vec![AcceptedSlot {
                        slot: 1,
                        ballot: 7,
                        value: "newer-value".to_string(),
                    }],
                },
                payload: String::new(),
                value: VoteValue::Yes,
                delay_count: 0,
            },
            Message {
                from: 4,
                to: 1,
                round: 0,
                msg_type: MessageType::MPPromise {
                    ballot: 1,
                    accepted: vec![AcceptedSlot {
                        slot: 1,
                        ballot: 5,
                        value: "middle-value".to_string(),
                    }],
                },
                payload: String::new(),
                value: VoteValue::Yes,
                delay_count: 0,
            },
        ];

        assert!(
            protocol
                .handle_message(&mut leader_node, &promises[0])
                .is_empty()
        );

        assert!(
            protocol
                .handle_message(&mut leader_node, &promises[1])
                .is_empty()
        );

        let actions = protocol.handle_message(&mut leader_node, &promises[2]);

        assert_eq!(mp_accept_value(&actions, 1).as_deref(), Some("newer-value"));

        assert_eq!(protocol.recovered[&1].ballot, 7);
        assert_eq!(protocol.recovered[&1].value, "newer-value");
    }

    #[test]
    fn duplicate_promise_does_not_count_twice() {
        let mut protocol = StableMultiPaxos::new(3);
        let mut leader_node = Node::new(1);

        let promise = Message {
            from: 2,
            to: 1,
            round: 0,
            msg_type: MessageType::MPPromise {
                ballot: 1,
                accepted: vec![],
            },
            payload: String::new(),
            value: VoteValue::Yes,
            delay_count: 0,
        };

        assert!(
            protocol
                .handle_message(&mut leader_node, &promise)
                .is_empty()
        );

        assert!(
            protocol
                .handle_message(&mut leader_node, &promise)
                .is_empty()
        );

        assert_eq!(protocol.promises.len(), 1);
        assert_eq!(protocol.phase, LeaderPhase::Preparing);
    }
}
