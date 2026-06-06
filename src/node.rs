use crate::message::{Message, MessageType, VoteValue};
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
}

#[derive(Debug)]
pub enum NodeAction {
    BroadcastVote(VoteValue),
    BroadcastCommit(VoteValue),
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
        }
    }
}


impl Node {

    fn count(&self, msg_type: MessageType, value: VoteValue) -> usize {
        *self
            .vote_counts
            .get(&(msg_type, value))
            .unwrap_or(&0)
    }

    pub fn receive(&mut self, msg: &Message) -> Vec<NodeAction> {
        self.messages_received += 1;
        let key = (msg.msg_type.clone(), msg.value.clone());

        let count = self.vote_counts.entry(key).or_insert(0);
        *count += 1;
        

        match msg.msg_type {
            MessageType::Proposal => {
                let proposal_yes = self.count(MessageType::Proposal, VoteValue::Yes);
                let proposal_no  = self.count(MessageType::Proposal, VoteValue::No);
            
                if self.quorum_reached(MessageType::Proposal, VoteValue::Yes, proposal_yes) {
                   // let old_state = self.state.clone();
                    self.state = NodeState::Proposed;
            
                   /* println!(
                        "Proposal quorum reached in Node {}: YES={}",
                        self.id,
                        proposal_yes
                    );
            
                    println!(
                        "Node {} state changed from {:?} to {:?}",
                        self.id,
                        old_state,
                        self.state
                    );
            
                    println!(
                        "Node {} should now broadcast Vote(YES)",
                        self.id
                    );*/

                    return vec![NodeAction::BroadcastVote(VoteValue::Yes)];
                } else if self.quorum_reached(MessageType::Proposal, VoteValue::No, proposal_no) {
                  //  let old_state = self.state.clone();
                    self.state = NodeState::Proposed;
            
                   /* println!(
                        "Proposal quorum reached in Node {}: NO={}",
                        self.id,
                        proposal_no
                    );
            
                    println!(
                        "Node {} state changed from {:?} to {:?}",
                        self.id,
                        old_state,
                        self.state
                    );
            
                    println!(
                        "Node {} should now broadcast Vote(NO)",
                        self.id
                    );*/

                    return vec![NodeAction::BroadcastVote(VoteValue::No)];
                }
            }
            
        
            MessageType::Vote => {

                let vote_yes = self.count(MessageType::Vote, VoteValue::Yes);
                let vote_no  = self.count(MessageType::Vote, VoteValue::No);
 
               
                if self.quorum_reached(MessageType::Vote, VoteValue::Yes, vote_yes) {
                    //let old_state = self.state.clone();
                    self.state = NodeState::Voted;
                
                   // println!("Vote quorum reached in Node {}: YES={}", self.id, vote_yes);
                   // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                    //println!("Node {} should now broadcast Commit(YES)", self.id);
                
                    return vec![NodeAction::BroadcastCommit(VoteValue::Yes)];
                } else if self.quorum_reached(MessageType::Vote, VoteValue::No, vote_no) {
                    //let old_state = self.state.clone();
                    self.state = NodeState::Voted;
                
                    //println!("Vote quorum reached in Node {}: NO={}", self.id, vote_no);
                   // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                    //println!("Node {} should now broadcast Commit(NO)", self.id);
                
                    return vec![NodeAction::BroadcastCommit(VoteValue::No)];
                }
            }
        
            MessageType::Commit => {
                let commit_yes = self.count(MessageType::Commit, VoteValue::Yes);
                let commit_no  = self.count(MessageType::Commit, VoteValue::No);
 
               
                if self.quorum_reached(
                 MessageType::Commit,
                 VoteValue::Yes,
                 commit_yes,
                 ) {
                   // let old_state = self.state.clone();
                    self.state = NodeState::Committed;
                    self.decided = Some(VoteValue::Yes);
                
                   // println!("Commit quorum reached in Node {}: YES={}", self.id, commit_yes);
                   // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                    //println!("Node {} DECIDED YES", self.id);
                
                    return vec![];                 }else if self.quorum_reached(
                     MessageType::Commit,
                     VoteValue::No, 
                     commit_no,
                 ) {
                  //  let old_state = self.state.clone();
                    self.state = NodeState::Committed;
                    self.decided = Some(VoteValue::No);
                
                  //  println!("Commit quorum reached in Node {}: YES={}", self.id, commit_no);
                   // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                  //  println!("Node {} DECIDED YES", self.id);
                
                    return vec![];   }
                
               /* trace(
                    TraceEvent::StateTransition,
                    &format!("Node {} {:?} -> {:?}", self.id, old_state, self.state),
                ); */
            }

            
        }  return vec![];
    }

    fn quorum_reached(
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