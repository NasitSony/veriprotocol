use crate::message::Message;
use crate::message::MessageType;
use crate::state::NodeState;
use crate::trace::{trace, TraceEvent};

#[derive(Debug)]
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
        trace(
            TraceEvent::Receive,
            &format!("Node {} received {:?} from {}", self.id, msg.msg_type, msg.from),
        );
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
               // println!("Node {} received PROPOSAL", self.id);
               // println!("Node {} state changed from {:?} to PROPOSED", self.id, self.state);
                let old_state = self.state.clone();
                self.state = NodeState::Proposed;
                trace(
                    TraceEvent::StateTransition,
                    &format!("Node {} {:?} -> {:?}", self.id, old_state, self.state),
                );
            }
        
            MessageType::Vote => {
                let old_state = self.state.clone();
                self.state = NodeState::Voted;
                trace(
                    TraceEvent::StateTransition,
                    &format!("Node {} {:?} -> {:?}", self.id, old_state, self.state),
                );
            }
        
            MessageType::Commit => {
                let old_state = self.state.clone();
                self.state = NodeState::Committed;
                trace(
                    TraceEvent::StateTransition,
                    &format!("Node {} {:?} -> {:?}", self.id, old_state, self.state),
                ); 
            }
        }
    }
}