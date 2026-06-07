use crate::message::{Message, VoteValue, MessageType};
use crate::node::{Node, NodeAction};
use crate::state::NodeState;

pub trait Protocol {
    fn handle_message(
        &mut self,
        node: &mut Node,
        msg: &Message,
    ) -> Vec<NodeAction>;
}

pub struct SimpleConsensusProtocol;

impl SimpleConsensusProtocol {
    pub fn new() -> Self {
        Self
    }
}


impl Protocol for SimpleConsensusProtocol {
    fn handle_message(
        &mut self,
        node: &mut Node,
        msg: &Message,
    ) -> Vec<NodeAction> {

        // copy existing receive() logic here
        //pub fn receive(&mut self, msg: &Message) -> Vec<NodeAction> {
            node.messages_received += 1;
            let key = (msg.msg_type.clone(), msg.value.clone());
    
            let count = node.vote_counts.entry(key).or_insert(0);
            *count += 1;
            
    
            match msg.msg_type {
                MessageType::Proposal => {
                    let proposal_yes = node.count(MessageType::Proposal, VoteValue::Yes);
                    let proposal_no  = node.count(MessageType::Proposal, VoteValue::No);
                
                    if node.quorum_reached(MessageType::Proposal, VoteValue::Yes, proposal_yes) {
                       // let old_state = self.state.clone();
                       node.state = NodeState::Proposed;
                
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
                    } else if node.quorum_reached(MessageType::Proposal, VoteValue::No, proposal_no) {
                      //  let old_state = self.state.clone();
                        node.state = NodeState::Proposed;
                
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
    
                    let vote_yes = node.count(MessageType::Vote, VoteValue::Yes);
                    let vote_no  = node.count(MessageType::Vote, VoteValue::No);
     
                   
                    if node.quorum_reached(MessageType::Vote, VoteValue::Yes, vote_yes) {
                        //let old_state = self.state.clone();
                        node.state = NodeState::Voted;
                    
                       // println!("Vote quorum reached in Node {}: YES={}", self.id, vote_yes);
                       // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                        //println!("Node {} should now broadcast Commit(YES)", self.id);
                    
                        return vec![NodeAction::BroadcastCommit(VoteValue::Yes)];
                    } else if node.quorum_reached(MessageType::Vote, VoteValue::No, vote_no) {
                        //let old_state = self.state.clone();
                        node.state = NodeState::Voted;
                    
                        //println!("Vote quorum reached in Node {}: NO={}", self.id, vote_no);
                       // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                        //println!("Node {} should now broadcast Commit(NO)", self.id);
                    
                        return vec![NodeAction::BroadcastCommit(VoteValue::No)];
                    }
                }
            
                MessageType::Commit => {
                    let commit_yes = node.count(MessageType::Commit, VoteValue::Yes);
                    let commit_no  = node.count(MessageType::Commit, VoteValue::No);
     
                   
                    if node.quorum_reached(
                     MessageType::Commit,
                     VoteValue::Yes,
                     commit_yes,
                     ) {
                       // let old_state = self.state.clone();
                        node.state = NodeState::Committed;
                        node.decided = Some(VoteValue::Yes);
                    
                       // println!("Commit quorum reached in Node {}: YES={}", self.id, commit_yes);
                       // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                        //println!("Node {} DECIDED YES", self.id);
                    
                        return vec![];                 }else if node.quorum_reached(
                         MessageType::Commit,
                         VoteValue::No, 
                         commit_no,
                     ) {
                      //  let old_state = self.state.clone();
                        node.state = NodeState::Committed;
                        node.decided = Some(VoteValue::No);
                    
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
}