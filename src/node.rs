use crate::message::{Message, MessageType, VoteValue};
use crate::state::NodeState;
use crate::trace::{trace, TraceEvent};


#[derive(Debug)]
pub struct Node {
    pub id: u64,
    pub messages_received: u64,
    pub state: NodeState,
    pub yes_votes: usize,
    pub no_votes: usize,
    pub decided: Option<VoteValue>,
}


impl Node {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            messages_received: 0,
            state: NodeState::Idle,
            yes_votes: 0,
            no_votes: 0,
            decided: None,
        }
    }
}


impl Node {
    pub fn receive(&mut self, msg: &Message) {
        self.messages_received += 1;
        /*trace(
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
        );*/// I do not need these messages right now

        match msg.msg_type {
            MessageType::Proposal => {
               // println!("Node {} received PROPOSAL", self.id);
               // println!("Node {} state changed from {:?} to PROPOSED", self.id, self.state);
                let old_state = self.state.clone();
                self.state = NodeState::Proposed;
                /*trace(
                    TraceEvent::StateTransition,
                    &format!("Node {} {:?} -> {:?}", self.id, old_state, self.state),
                );*/ // Omiting trace for state transition to PROPOSED, as it is not critical for the consensus outcome
            }
        
            MessageType::Vote => {
                let old_state = self.state.clone();
                self.state = NodeState::Voted;
                match msg.value {
                    VoteValue::Yes => {
                        self.yes_votes += 1;
                    }
                    VoteValue::No => {
                        self.no_votes += 1;
                    }

                    
                }
        
                println!(
                    "Node {}: YES={} NO={}",
                    self.id,
                    self.yes_votes,
                    self.no_votes
                );

                if self.yes_votes >= 3 && self.decided.is_none() {
                    self.decided = Some(VoteValue::Yes);
                    println!("Node {} DECIDED YES", self.id);
                }
                
                if self.no_votes >= 3 && self.decided.is_none() {
                    self.decided = Some(VoteValue::No);
                    println!("Node {} DECIDED NO", self.id);
                }
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