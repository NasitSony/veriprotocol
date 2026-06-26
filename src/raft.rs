use crate::message::{Message, MessageType};
use crate::node::{Node, NodeAction, RaftRole};
use crate::protocol::Protocol;

use std::collections::{HashMap, HashSet};

pub struct RaftProtocol {
    pub leader_id: Option<u64>,
    pub election_count: u64,
    pub quorum_size: usize,
    pub votes_by_term: std::collections::HashMap<u64, HashSet<u64>>,
    pub leader_term: u64,
    pub config_acks_by_term: HashMap<u64, HashSet<u64>>,
}

impl RaftProtocol {
    pub fn new(quorum_size: usize) -> Self {
        Self {
            leader_id: None,
            election_count: 0,
            quorum_size,
            votes_by_term: std::collections::HashMap::new(),
            leader_term: 0,
            config_acks_by_term: HashMap::new(),
        }
    }
}

impl Protocol for RaftProtocol {
    fn handle_message(&mut self, node: &mut Node, msg: &Message) -> Vec<NodeAction> {
        match &msg.msg_type {
            MessageType::RequestVote { term, candidate_id } => {
                if *term > node.raft_current_term {
                    node.raft_current_term = *term;
                    node.raft_role = RaftRole::Follower;
                    node.raft_voted_for = None;
                }

                let vote_granted =
                    *term == node.raft_current_term
                        && (node.raft_voted_for.is_none()
                            || node.raft_voted_for == Some(*candidate_id));

                if vote_granted {
                    node.raft_voted_for = Some(*candidate_id);
                }

                vec![NodeAction::SendVoteResponse {
                    to: *candidate_id,
                    term: node.raft_current_term,
                    vote_granted,
                }]
            }

            MessageType::VoteResponse { term, vote_granted } => {
                if !*vote_granted {
                    return vec![];
                }

                let votes = self
                    .votes_by_term
                    .entry(*term)
                    .or_insert_with(HashSet::new);

                votes.insert(msg.from);

                if votes.len() >= self.quorum_size && *term > self.leader_term {
                    self.leader_id = Some(msg.to);
                    self.leader_term = *term;
                    self.election_count += 1;

                    return vec![NodeAction::BecomeRaftLeader {
                        leader_id: msg.to,
                        term: *term,
                    }];
                }

                vec![]
            }

            MessageType::VoteResponse { term, vote_granted } => {
                if !*vote_granted {
                    return vec![];
                }

                self.votes_by_term
                    .entry(*term)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);

                vec![]
            }

            MessageType::AppendEntries { term, leader_id } => {
                if *term >= node.raft_current_term {
                    node.raft_current_term = *term;
                    node.raft_role = RaftRole::Follower;
                    self.leader_id = Some(*leader_id);

                    return vec![NodeAction::SendAppendResponse {
                        to: *leader_id,
                        term: *term,
                        success: true,
                    }];
                }

                vec![NodeAction::SendAppendResponse {
                    to: *leader_id,
                    term: node.raft_current_term,
                    success: false,
                }]
            }

            MessageType::AppendResponse { term: _, success: _ } => {
                vec![]
            }

            MessageType::AppendEntries { term, leader_id } => {
                if *term >= node.raft_current_term {
                    node.raft_current_term = *term;
                    node.raft_role = RaftRole::Follower;
                    node.raft_voted_for = None;
                    self.leader_id = Some(*leader_id);

                    return vec![NodeAction::SendAppendResponse {
                        to: *leader_id,
                        term: *term,
                        success: true,
                    }];
                }

                vec![NodeAction::SendAppendResponse {
                    to: *leader_id,
                    term: node.raft_current_term,
                    success: false,
                }]
            }

            MessageType::RaftConfigChange {
                    term,
                    leader_id,
                    new_node_count,
                } => {
                    if *term >= node.raft_current_term {
                        node.raft_current_term = *term;
                        node.raft_role = RaftRole::Follower;
                        self.leader_id = Some(*leader_id);

                        return vec![NodeAction::SendRaftConfigAck {
                            to: *leader_id,
                            term: *term,
                            success: true,
                            new_node_count: *new_node_count,
                        }];
                    }

                    vec![NodeAction::SendRaftConfigAck {
                        to: *leader_id,
                        term: node.raft_current_term,
                        success: false,
                        new_node_count: *new_node_count,
                    }]
                }

             MessageType::RaftConfigAck {
                    term,
                    success,
                    new_node_count,
                } => {
                    if !*success {
                        return vec![];
                    }

                    let acks = self
                        .config_acks_by_term
                        .entry(*term)
                        .or_insert_with(HashSet::new);

                    acks.insert(msg.from);

                    if acks.len() >= self.quorum_size {
                        return vec![NodeAction::ActivateRaftConfig {
                            new_node_count: *new_node_count,
                        }];
                    }

                    vec![]
                }   

            _ => vec![],
        }
    } 
}