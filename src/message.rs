pub enum MessageType {
    Proposal,
    Vote,
    Commit,
}

pub struct Message {
    pub from: u64,
    pub to: u64,
    pub round: u64,
    pub msg_type: MessageType,
    pub payload: String,
}