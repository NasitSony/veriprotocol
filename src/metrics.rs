// metrics.rs
use std::collections::HashSet;

pub struct Metrics {
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub decisions: u64,
    pub messages_delivered_until_decision: u64,
    pub messages_sent_until_decision: u64,
    pub timeouts_triggered: u64,
    pub view_changes: u64,
    pub stale_messages_ignored: u64,
    pub scheduler_steps: u64,

    pub prepare_messages: u64,
    pub promise_messages: u64,
    pub accept_requests: u64,
    pub accepted_messages: u64,

    pub chosen_values: HashSet<String>,
    pub safety_violation: bool,

    pub nack_messages: u64,
    pub paxos_retries: u64,
    pub max_ballot_seen: u64,

    pub paxos_retry_exhausted: bool,

    pub membership_changes: u64,
    pub membership_acks: u64,

    pub request_vote_messages: u64,
    pub vote_response_messages: u64,
    pub votes_granted: u64,
    pub votes_rejected: u64,

    pub raft_leader_elected: bool,
    pub raft_leader_id: Option<u64>,
    pub raft_election_count: u64,

}

impl Metrics {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_delivered: 0,
            decisions: 0,
            messages_delivered_until_decision: 0,
            messages_sent_until_decision: 0,
            timeouts_triggered: 0,
            view_changes: 0,
            stale_messages_ignored: 0,
            scheduler_steps: 0,
            prepare_messages: 0,
            promise_messages: 0,
            accept_requests: 0,
            accepted_messages: 0,

            chosen_values: HashSet::new(),
            safety_violation: false,

            nack_messages: 0,
            paxos_retries: 0,
            max_ballot_seen: 0,

            paxos_retry_exhausted: false,

            membership_changes: 0,
            membership_acks:0,

            request_vote_messages: 0,
            vote_response_messages: 0,
            votes_granted: 0,
            votes_rejected: 0,

            raft_leader_elected: false,
            raft_leader_id: None,
            raft_election_count: 0,

        }
    }
}

impl Metrics {
    pub fn print(&self) {
        println!("\n=== Metrics ===");
        println!("Messages Sent: {}", self.messages_sent);
        println!("Messages Delivered: {}", self.messages_delivered);
        println!("Scheduler Steps: {}", self.scheduler_steps);
        println!("Stale Messages Ignored: {}", self.stale_messages_ignored);
        println!(
            "Messages Sent Until Decision: {}",
            self.messages_sent_until_decision
        );
        println!(
            "Undelivered Messages At Decision: {}",
            self.messages_sent_until_decision - self.messages_delivered_until_decision
        );
        println!("Timeout triggered: {}", self.timeouts_triggered);
        println!("View changes: {}", self.view_changes);
        println!("Decisions: {}", self.decisions);


        println!("Prepare Messages: {}", self.prepare_messages);
        println!("Promise Messages: {}", self.promise_messages);
        println!("Accept Requests: {}", self.accept_requests);
        println!("Accepted Messages: {}", self.accepted_messages);

        println!("Raft Leader Elected: {}", self.raft_leader_elected);
        println!("Raft Leader Id: {:?}", self.raft_leader_id);
        println!("Raft Election Count: {}", self.raft_election_count);

        println!("Nack Messages: {}", self.nack_messages);
        println!("Paxos Retries: {}", self.paxos_retries);
        println!("Max Ballot Seen: {}", self.max_ballot_seen);

        println!("Membership Changes: {}", self.membership_changes);
        println!("Membership Acks: {}", self.membership_acks);

        println!("Chosen Values: {:?}", self.chosen_values);
        println!("Safety Violation: {}", self.safety_violation);

        println!("Paxos Retry Exhausted: {}", self.paxos_retry_exhausted);


    }
}
