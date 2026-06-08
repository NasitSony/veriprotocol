use crate::message::{MessageType, VoteValue};
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
}

#[derive(Debug)]
pub enum NodeAction {
    BroadcastVote(VoteValue),
    BroadcastCommit(VoteValue),
    BroadcastTimeout,
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
        }
    }
}


impl Node {

   pub(crate) fn count(&self, msg_type: MessageType, value: VoteValue) -> usize {
        *self
            .vote_counts
            .get(&(msg_type, value))
            .unwrap_or(&0)
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