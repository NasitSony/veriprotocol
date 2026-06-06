#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
    Proposal,
    Vote,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VoteValue {
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: u64,
    pub to: u64,
    pub round: u64,
    pub msg_type: MessageType,
    pub payload: String,
    pub value: VoteValue,
    pub delay_count: usize,
}