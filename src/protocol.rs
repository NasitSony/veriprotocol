use crate::message::{Message, MessageType, VoteValue};
use crate::node::{Node, NodeAction};
use crate::state::NodeState;

pub trait Protocol {
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction>;

    // in Protocol trait
    //fn initial_actions(&self, node_id: usize) -> Vec<NodeAction> {
    //  vec![NodeAction::BroadcastProposal]
    //}

    fn should_send_initial_proposal(&self, _node_id: usize) -> bool {
        true
    }
    fn uses_timeout(&self) -> bool {
        false
    }

    fn on_timeout(&mut self) -> Vec<NodeAction> {
        vec![]
    }

    fn on_tick(&mut self) -> Vec<NodeAction> {
        vec![]
    }
}

pub struct SimpleConsensusProtocol;

impl SimpleConsensusProtocol {
    pub fn new() -> Self {
        Self
    }
}

pub struct TwoPhaseProtocol;

impl TwoPhaseProtocol {
    pub fn new() -> Self {
        Self
    }
}

impl Protocol for SimpleConsensusProtocol {
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        // copy existing receive() logic here
        //pub fn receive(&mut self, msg: &Message) -> Vec<NodeAction> {
        node.messages_received += 1;
        let key = (msg.msg_type.clone(), msg.value.clone());

        let count = node.vote_counts.entry(key).or_insert(0);
        *count += 1;

        match msg.msg_type {
            MessageType::Proposal => {
                let proposal_yes = node.count(MessageType::Proposal, VoteValue::Yes);
                let proposal_no = node.count(MessageType::Proposal, VoteValue::No);

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
                let vote_no = node.count(MessageType::Vote, VoteValue::No);

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
                let commit_no = node.count(MessageType::Commit, VoteValue::No);

                if node.quorum_reached(MessageType::Commit, VoteValue::Yes, commit_yes) {
                    // let old_state = self.state.clone();
                    node.state = NodeState::Committed;
                    node.decided = Some(VoteValue::Yes);

                    // println!("Commit quorum reached in Node {}: YES={}", self.id, commit_yes);
                    // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                    //println!("Node {} DECIDED YES", self.id);

                    return vec![];
                } else if node.quorum_reached(MessageType::Commit, VoteValue::No, commit_no) {
                    //  let old_state = self.state.clone();
                    node.state = NodeState::Committed;
                    node.decided = Some(VoteValue::No);

                    //  println!("Commit quorum reached in Node {}: YES={}", self.id, commit_no);
                    // println!("Node {} state changed from {:?} to {:?}", self.id, old_state, self.state);
                    //  println!("Node {} DECIDED YES", self.id);

                    return vec![];
                }

                /* trace(
                    TraceEvent::StateTransition,
                    &format!("Node {} {:?} -> {:?}", self.id, old_state, self.state),
                ); */
            }

            MessageType::Timeout => {
                // ignored by this protocol
            }

            _ => {
                // Paxos messages ignored by non-Paxos protocols.
            }
        }
        return vec![];
    }
}

impl Protocol for TwoPhaseProtocol {
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        node.messages_received += 1;

        let key = (msg.msg_type.clone(), msg.value.clone());
        let count = node.vote_counts.entry(key).or_insert(0);
        *count += 1;

        match msg.msg_type {
            MessageType::Proposal => {
                let proposal_yes = node.count(MessageType::Proposal, VoteValue::Yes);
                let proposal_no = node.count(MessageType::Proposal, VoteValue::No);

                if node.quorum_reached(MessageType::Proposal, VoteValue::Yes, proposal_yes) {
                    node.state = NodeState::Proposed;
                    return vec![NodeAction::BroadcastVote(VoteValue::Yes)];
                } else if node.quorum_reached(MessageType::Proposal, VoteValue::No, proposal_no) {
                    node.state = NodeState::Proposed;
                    return vec![NodeAction::BroadcastVote(VoteValue::No)];
                }
            }

            MessageType::Vote => {
                let vote_yes = node.count(MessageType::Vote, VoteValue::Yes);
                let vote_no = node.count(MessageType::Vote, VoteValue::No);

                if node.quorum_reached(MessageType::Vote, VoteValue::Yes, vote_yes) {
                    node.state = NodeState::Committed;
                    node.decided = Some(VoteValue::Yes);
                    return vec![];
                } else if node.quorum_reached(MessageType::Vote, VoteValue::No, vote_no) {
                    node.state = NodeState::Committed;
                    node.decided = Some(VoteValue::No);
                    return vec![];
                }
            }

            MessageType::Commit => {
                // TwoPhaseProtocol does not use Commit.
            }

            MessageType::Timeout => {
                // ignored by this protocol
            }

            _ => {
                // Paxos messages ignored by non-Paxos protocols.
            }
        }

        vec![]
    }
}

pub struct TimeoutProtocol {
    pub num_nodes: u64,
}

impl TimeoutProtocol {
    pub fn new(num_nodes: u64) -> Self {
        Self { num_nodes }
    }

    fn leader_for_view(&self, view: u64) -> u64 {
        (view % self.num_nodes) + 1
    }
}

impl Protocol for TimeoutProtocol {
    fn should_send_initial_proposal(&self, node_id: usize) -> bool {
        node_id as u64 == self.leader_for_view(0)
    }

    fn uses_timeout(&self) -> bool {
        true
    }

    /*fn initial_actions(&self, node_id: usize) -> Vec<NodeAction> {
        if node_id == self.current_leader() {
            vec![NodeAction::BroadcastProposal]
        } else {
            vec![]
        }
    }*/
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        node.messages_received += 1;

        let key = (msg.msg_type.clone(), msg.value.clone());
        let count = node.vote_counts.entry(key).or_insert(0);
        *count += 1;

        match msg.msg_type {
            MessageType::Proposal => {
                if msg.round < node.view {
                    return vec![NodeAction::StaleMessageIgnored];
                }
                if msg.from == node.leader {
                    node.state = NodeState::Proposed;
                    return vec![NodeAction::BroadcastVote(VoteValue::Yes)];
                }
            }

            MessageType::Timeout => {
                node.view += 1;

                let new_leader = self.leader_for_view(node.view);

                node.leader = new_leader;

                if node.id == new_leader {
                    return vec![NodeAction::BroadcastProposal];
                }

                if msg.round < node.view {
                    return vec![NodeAction::StaleMessageIgnored];
                }

                return vec![];
            }

            MessageType::Vote => {
                let vote_yes = node.count(MessageType::Vote, VoteValue::Yes);
                let vote_no = node.count(MessageType::Vote, VoteValue::No);

                if node.quorum_reached(MessageType::Vote, VoteValue::Yes, vote_yes) {
                    node.state = NodeState::Committed;
                    node.decided = Some(VoteValue::Yes);
                    return vec![];
                } else if node.quorum_reached(MessageType::Vote, VoteValue::No, vote_no) {
                    node.state = NodeState::Committed;
                    node.decided = Some(VoteValue::No);
                    return vec![];
                }
            }

            MessageType::Commit => {
                // later
            }

            _ => {
                // Paxos messages ignored by non-Paxos protocols.
            }
        }

        vec![]
    }
}
