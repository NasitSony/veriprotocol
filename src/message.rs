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
