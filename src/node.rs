use crate::message::Message;
use crate::message::MessageType;
use crate::state::NodeState;

pub struct Node {
    pub id: u64,
    pub messages_received: u64,
    pub state: NodeState,
}
impl Node {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            messages_received: 0,
            state: NodeState::Idle,
        }
    }
}


impl Node {
    pub fn receive(&mut self, msg: &Message) {
        self.messages_received += 1;
        println!(
            "Node {} received payload {} in round {}",
            self.id,
            msg.payload,
            msg.round
        );
        println!(
            "Node {} received {} messages",
            self.id,
            self.messages_received
        );

        match msg.msg_type {
            MessageType::Proposal => {
                println!("Node {} received PROPOSAL", self.id);
                println!("Node {} state changed from {:?} to PROPOSED", self.id, self.state);
                self.state = NodeState::Proposed;
            }
        
            MessageType::Vote => {
                println!("Node {} received VOTE", self.id);
                println!("Node {} state changed from {:?} to VOTED", self.id, self.state);
                self.state = NodeState::Voted;
            }
        
            MessageType::Commit => {
                println!("Node {} received COMMIT", self.id);
                println!("Node {} state changed from {:?} to COMMITTED", self.id, self.state);
                self.state = NodeState::Committed;
            }
        }
    }
}