use crate::message::{Message, MessageType};
use crate::node::{Node, NodeAction, RaftRole};
use crate::protocol::Protocol;

pub struct RaftProtocol {
    pub leader_id: Option<u64>,
    pub election_count: u64,
    pub quorum_size: usize,
}

impl RaftProtocol {
    pub fn new(quorum_size: usize) -> Self {
        Self {
            leader_id: None,
            election_count: 0,
            quorum_size,
        }
    }
}

impl Protocol for RaftProtocol {
    fn handle_message(&mut self, _node: &mut Node, _msg: &Message) -> Vec<NodeAction> {
        vec![]
    }
}