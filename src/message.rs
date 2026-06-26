#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
    Proposal,
    Vote,
    Commit,
    Timeout,

    // Basic Paxos
    Prepare {
        ballot: u64,
    },
    Promise {
        ballot: u64,
        accepted_ballot: Option<u64>,
        accepted_value: Option<String>,
    },
    AcceptRequest {
        ballot: u64,
        value: String,
    },
    Accepted {
        ballot: u64,
        value: String,
    },

    Nack {
        ballot: u64,
        promised_ballot: u64,
    },

    MembershipChange {
        new_node_count: usize,
    },
    
    MembershipAck {
       new_node_count: usize,
    },

    RequestVote {
    term: u64,
    candidate_id: u64,
    },

    VoteResponse {
        term: u64,
        vote_granted: bool,
    },

    AppendEntries {
        term: u64,
        leader_id: u64,
    },

    AppendResponse {
        term: u64,
        success: bool,
    },

    RaftConfigChange {
        term: u64,
        leader_id: u64,
        new_node_count: usize,
    },

    RaftConfigAck {
        term: u64,
        success: bool,
        new_node_count: usize,
    },
    
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VoteValue {
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message {
    pub from: u64,
    pub to: u64,
    pub round: u64,
    pub msg_type: MessageType,
    pub payload: String,
    pub value: VoteValue,
    pub delay_count: usize,
}
