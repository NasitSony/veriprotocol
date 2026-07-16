use crate::message::{AcceptedSlot, MessageType, VoteValue};
use crate::state::NodeState;
//use crate::trace::{trace, TraceEvent};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Node {
    pub id: u64,
    pub messages_received: u64,
    pub state: NodeState,
    pub decided: Option<VoteValue>,
    pub vote_counts: HashMap<(MessageType, VoteValue), usize>,
    pub completed_quorums: HashSet<(MessageType, VoteValue)>,
    pub(crate) view: u64,
    pub(crate) leader: u64,

    // Basic Paxos acceptor state
    pub(crate) promised_ballot: u64,
    pub(crate) accepted_ballot: Option<u64>,
    pub(crate) accepted_value: Option<String>,

    pub accepted_slots: HashMap<u64, AcceptedSlot>,

    pub raft_role: RaftRole,
    pub raft_current_term: u64,
    pub raft_voted_for: Option<u64>,
}

#[derive(Debug)]
pub enum NodeAction {
    BroadcastVote(VoteValue),
    BroadcastCommit(VoteValue),
    BroadcastTimeout,
    StaleMessageIgnored,
    BroadcastProposal,
    SendPromise {
        to: u64,
        ballot: u64,
        accepted_ballot: Option<u64>,
        accepted_value: Option<String>,
    },

    BroadcastAcceptRequest {
        ballot: u64,
        value: String,
    },

    SendAccepted {
        to: u64,
        ballot: u64,
        value: String,
    },

    BroadcastPrepare {
        ballot: u64,
    },

    BroadcastPrepareFrom {
        from: u64,
        ballot: u64,
    },

    RecordChosen {
        value: String,
    },

    SendNack {
        to: u64,
        ballot: u64,
        promised_ballot: u64,
    },

    BroadcastMembershipChange {
        new_node_count: usize,
    },

    SendMembershipAck {
        to: u64,
        new_node_count: usize,
    },

    BroadcastRequestVote {
        term: u64,
        candidate_id: u64,
    },

    SendVoteResponse {
        to: u64,
        term: u64,
        vote_granted: bool,
    },

    BecomeRaftLeader {
        leader_id: u64,
        term: u64,
    },

    BroadcastAppendEntries {
        term: u64,
        leader_id: u64,
    },

    SendAppendResponse {
        to: u64,
        term: u64,
        success: bool,
    },

    /*BroadcastRaftConfigChange {
        term: u64,
        leader_id: u64,
        new_node_count: usize,
    },*/
    SendRaftConfigAck {
        to: u64,
        term: u64,
        success: bool,
        new_node_count: usize,
    },

    ActivateRaftConfig {
        new_node_count: usize,
    },

    SendMPPromise {
        to: u64,
        ballot: u64,
        accepted: Vec<AcceptedSlot>,
    },

    BroadcastMPAcceptRequest {
        ballot: u64,
        slot: u64,
        value: String,
    },

    SendMPAccepted {
        to: u64,
        ballot: u64,
        slot: u64,
        value: String,
    },

    RecordMPChosen {
        slot: u64,
        value: String,
    },

    BroadcastMPPrepare {
        from: u64,
        ballot: u64,
    },

    BroadcastMPHeartbeat {
        leader_id: u64,
        ballot: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}

impl Node {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            messages_received: 0,
            state: NodeState::Idle,
            decided: None,
            vote_counts: HashMap::new(),
            completed_quorums: HashSet::new(),
            view: 0,
            leader: 1,
            promised_ballot: 0,
            accepted_ballot: None,
            accepted_value: None,
            accepted_slots: HashMap::new(),
            raft_role: RaftRole::Follower,
            raft_current_term: 0,
            raft_voted_for: None,
        }
    }
}

impl Node {
    pub(crate) fn count(&self, msg_type: MessageType, value: VoteValue) -> usize {
        *self.vote_counts.get(&(msg_type, value)).unwrap_or(&0)
    }

    pub(crate) fn quorum_reached(
        &mut self,
        msg_type: MessageType,
        value: VoteValue,
        count: usize,
    ) -> bool {
        let key = (msg_type, value);

        if count >= 3 && !self.completed_quorums.contains(&key) {
            self.completed_quorums.insert(key);
            return true;
        }

        false
    }
}
