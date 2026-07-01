use crate::message::{Message, MessageType, VoteValue};
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::{HashMap, HashSet};
use rand::Rng;


pub enum SchedulerOutcome {
    Deliver(Message),
    Delay,
    Empty,
}

pub struct PaxosRetryScheduler {
    pub held_accepts: Vec<Message>,
    pub max_delay: usize,
    pub delays_used: usize,
}

pub trait Scheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome;
}

fn is_critical_message(msg: &Message) -> bool {
    matches!(
        msg.msg_type,
        MessageType::Promise { .. }
            | MessageType::Accepted { .. }
            | MessageType::MembershipAck { .. }
            | MessageType::VoteResponse { .. }
            | MessageType::AppendResponse { .. }
            | MessageType::RaftConfigAck { .. }
    )
}

fn highest_ballot(queue: &[Message]) -> Option<u64> {
    queue.iter().filter_map(paxos_ballot).max()
}

fn distinct_ballots(queue: &[Message]) -> Vec<u64> {
    let mut ballots = queue.iter().filter_map(paxos_ballot).collect::<Vec<u64>>();

    ballots.sort();
    ballots.dedup();
    ballots
}

fn deliver(queue: &mut Vec<Message>, index: usize) -> SchedulerOutcome {
    let msg = queue.remove(index);
    let ballots = distinct_ballots(queue);

    println!(
        "[SCHED] Deliver {:?} ballot={:?} highest={:?} queue={} distinct_ballots={:?} count={}",
        msg.msg_type,
        paxos_ballot(&msg),
        highest_ballot(queue),
        queue.len(),
        ballots,
        ballots.len(),
    );

    SchedulerOutcome::Deliver(msg)
}

pub struct FifoScheduler;

pub struct TimeoutFirstScheduler;

pub struct RandomScheduler {
    rng: StdRng,
}
pub struct DelayScheduler {
    pub delayed_node: u64,
}
pub struct CommitDelayScheduler;

pub struct VoteDelayScheduler;

pub struct ProposalDelayScheduler;

pub struct BoundedDelayScheduler {
    pub max_delay: usize,
}

pub struct DelayLeaderScheduler;
pub struct BoundedDelayLeaderScheduler {
    max_delays: usize,
}

pub struct ProbabilisticDelayScheduler {
    max_delay: usize,
    rng: StdRng,
}

pub struct QuorumBlockingScheduler {
    delivered_counts: HashMap<(u64, MessageType, VoteValue), usize>,
}

pub struct CriticalMessageDelayScheduler {
    pub max_delay: usize,
    pub delayed: Vec<Message>,
    pub delays_used: usize,
}

pub struct PaxosRetryAdversaryScheduler {
    pub delayed_accepts: Vec<Message>,
    pub max_delay: usize,
    pub delays_used: usize,
}

impl CommitDelayScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl VoteDelayScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl ProposalDelayScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl FifoScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl RandomScheduler {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl DelayScheduler {
    pub fn new(delayed_node: u64) -> Self {
        Self { delayed_node }
    }
}

impl BoundedDelayScheduler {
    pub fn new(max_delay: usize) -> Self {
        Self {
            max_delay: max_delay,
        }
    }
}

impl ProbabilisticDelayScheduler {
    pub fn new(max_delay: usize, seed: u64) -> Self {
        Self {
            max_delay: max_delay,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl QuorumBlockingScheduler {
    pub fn new() -> Self {
        Self {
            delivered_counts: HashMap::new(),
        }
    }
}

impl DelayLeaderScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl BoundedDelayLeaderScheduler {
    pub fn new(max_delays: usize) -> Self {
        Self {
            max_delays: max_delays,
        }
    }
}

impl Scheduler for FifoScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            SchedulerOutcome::Empty
        } else {
            let msg = queue.remove(0);

            println!(
                "[SCHED] Deliver {:?} ballot={:?} highest_in_queue={:?}",
                msg.msg_type,
                paxos_ballot(&msg),
                highest_ballot(queue),
            );

            SchedulerOutcome::Deliver(msg)
        }
    }
}

impl Scheduler for RandomScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let index = self.rng.random_range(0..queue.len());
        let msg = queue.remove(index);

        let ballots = distinct_ballots(queue);

        println!(
            "[RANDOM-SCHED] Deliver {:?} ballot={:?} highest={:?} queue={} distinct_ballots={:?} count={}",
            msg.msg_type,
            paxos_ballot(&msg),
            highest_ballot(queue),
            queue.len(),
            ballots,
            ballots.len(),
        );

        SchedulerOutcome::Deliver(msg)
    }
}

impl Scheduler for DelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.to == self.delayed_node {
                queue.push(msg);
                return SchedulerOutcome::Delay;
            } else {
                return SchedulerOutcome::Deliver(msg);
            }
        }

        SchedulerOutcome::Empty
    }
}

impl Scheduler for CommitDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.msg_type == MessageType::Commit {
                queue.push(msg);
                return SchedulerOutcome::Delay;
            } else {
                return SchedulerOutcome::Deliver(msg);
            }
        }
        SchedulerOutcome::Empty
    }
}

impl Scheduler for VoteDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.msg_type == MessageType::Vote {
                queue.push(msg);
                return SchedulerOutcome::Delay;
            } else {
                return SchedulerOutcome::Deliver(msg);
            }
        }

        SchedulerOutcome::Empty
    }
}

impl Scheduler for ProposalDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.msg_type == MessageType::Proposal {
                queue.push(msg);
                return SchedulerOutcome::Delay;
            } else {
                return SchedulerOutcome::Deliver(msg);
            }
        }
        SchedulerOutcome::Empty
    }
}

impl Scheduler for BoundedDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let mut msg = queue.remove(0);

            if msg.delay_count < self.max_delay {
                msg.delay_count += 1;
                queue.push(msg);
                return SchedulerOutcome::Delay;
            } else {
                return SchedulerOutcome::Deliver(msg);
            }
        }
        SchedulerOutcome::Empty
    }
}

impl Scheduler for ProbabilisticDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let mut msg = queue.remove(0);

            if msg.delay_count < self.max_delay {
                let coin = self.rng.random_range(0..2);

                if coin == 0 {
                    msg.delay_count += 1;
                    queue.push(msg);
                    continue;
                }
                return SchedulerOutcome::Delay;
            }
            return SchedulerOutcome::Deliver(msg);
        }

        if queue.is_empty() {
            SchedulerOutcome::Empty
        } else {
            SchedulerOutcome::Deliver(queue.remove(0))
        }
    }
}

impl Scheduler for QuorumBlockingScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            let key = (msg.to, msg.msg_type.clone(), msg.value.clone());

            let delivered_so_far = *self.delivered_counts.get(&key).unwrap_or(&0);

            // If this would be the 3rd matching message,
            // delay it by moving it to the back.
            if delivered_so_far == 2 {
                queue.push(msg);
                continue;
            }

            self.delivered_counts.insert(key, delivered_so_far + 1);

            return SchedulerOutcome::Deliver(msg);
        }

        // If every message would complete quorum,
        // force deliver one to avoid deadlock.
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        } else {
            let msg = queue.remove(0);

            let key = (msg.to, msg.msg_type.clone(), msg.value.clone());

            let delivered_so_far = *self.delivered_counts.get(&key).unwrap_or(&0);

            self.delivered_counts.insert(key, delivered_so_far + 1);

            SchedulerOutcome::Deliver(msg)
        }
    }
}

impl Scheduler for TimeoutFirstScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if let Some(pos) = queue
            .iter()
            .position(|msg| msg.msg_type == MessageType::Timeout)
        {
            return SchedulerOutcome::Deliver(queue.remove(pos));
        }

        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        } else {
            return SchedulerOutcome::Deliver(queue.remove(0));
        }
    }
}

impl Scheduler for DelayLeaderScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if let Some(pos) = queue.iter().position(|msg| msg.from != 1) {
            return SchedulerOutcome::Deliver(queue.remove(pos));
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

impl Scheduler for BoundedDelayLeaderScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        let leader = 1;
        let i = 0;

        if queue[i].from == leader && queue[i].delay_count < self.max_delays {
            let mut msg = queue.remove(i);
            msg.delay_count += 1;
            queue.push(msg);
            return SchedulerOutcome::Delay;
        }

        SchedulerOutcome::Deliver(queue.remove(i))
    }
}

impl Scheduler for CriticalMessageDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            if !self.delayed.is_empty() {
                let msg = self.delayed.remove(0);
                println!("Releasing delayed message: {:?}", msg.msg_type);
                return SchedulerOutcome::Deliver(msg);
            }
            return SchedulerOutcome::Empty;
        }

        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(is_critical_message) {
                let msg = queue.remove(pos);
                println!("Delaying critical message: {:?}", msg.msg_type);
                self.delayed.push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    return SchedulerOutcome::Deliver(queue.remove(0));
                }
            }
        }

        if !queue.is_empty() {
            return SchedulerOutcome::Deliver(queue.remove(0));
        }

        if !self.delayed.is_empty() {
            let msg = self.delayed.remove(0);
            println!("Releasing delayed message: {:?}", msg.msg_type);
            return SchedulerOutcome::Deliver(msg);
        }

        SchedulerOutcome::Empty
    }
}

fn is_paxos_accept_request(msg: &Message) -> bool {
    matches!(msg.msg_type, MessageType::AcceptRequest { .. })
}

impl Scheduler for PaxosRetryAdversaryScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            if !self.delayed_accepts.is_empty() {
                return SchedulerOutcome::Deliver(self.delayed_accepts.remove(0));
            }
            return SchedulerOutcome::Empty;
        }

        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(is_paxos_accept_request) {
                let msg = queue.remove(pos);
                self.delayed_accepts.push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    return SchedulerOutcome::Deliver(queue.remove(0));
                }
            }
        }

        if !queue.is_empty() {
            return SchedulerOutcome::Deliver(queue.remove(0));
        }

        if !self.delayed_accepts.is_empty() {
            return SchedulerOutcome::Deliver(self.delayed_accepts.remove(0));
        }

        SchedulerOutcome::Empty
    }
}

fn paxos_ballot(msg: &Message) -> Option<u64> {
    match &msg.msg_type {
        MessageType::Prepare { ballot } => Some(*ballot),
        MessageType::Promise { ballot, .. } => Some(*ballot),
        MessageType::AcceptRequest { ballot, .. } => Some(*ballot),
        MessageType::Accepted { ballot, .. } => Some(*ballot),
        MessageType::Nack { ballot, .. } => Some(*ballot),
        _ => None,
    }
}

fn is_accept_request(msg: &Message) -> bool {
    matches!(&msg.msg_type, MessageType::AcceptRequest { .. })
}

fn has_higher_ballot(queue: &[Message], ballot: u64) -> bool {
    queue
        .iter()
        .any(|m| paxos_ballot(m).map(|b| b > ballot).unwrap_or(false))
}

impl Scheduler for PaxosRetryScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            if !self.held_accepts.is_empty() {
                let msg = self.held_accepts.remove(0);

                println!(
                    "[PAXOS-RETRY-SCHED] Release held {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                return SchedulerOutcome::Deliver(msg);
            }
            return SchedulerOutcome::Empty;
        }

        // 1. If there is an AcceptRequest and a higher ballot exists,
        // hold the old AcceptRequest.
        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(|msg| {
                if let MessageType::AcceptRequest { ballot, .. } = &msg.msg_type {
                    has_higher_ballot(queue, *ballot)
                } else {
                    false
                }
            }) {
                let msg = queue.remove(pos);

                println!(
                    "[PAXOS-RETRY-SCHED] Hold {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                self.held_accepts.push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    //return SchedulerOutcome::Deliver(queue.remove(0));
                    return deliver(queue, 0);
                }
            }
        }

        // 2. Prefer higher-ballot messages.
        if let Some(max_ballot) = queue.iter().filter_map(paxos_ballot).max() {
            if let Some(pos) = queue
                .iter()
                .position(|m| paxos_ballot(m) == Some(max_ballot))
            {
                return deliver(queue, pos);
                //return SchedulerOutcome::Deliver(queue.remove(pos));
            }
        }

        // 3. Release held old AcceptRequest only after newer ballot activity.
        if !self.held_accepts.is_empty() {
            let msg = self.held_accepts.remove(0);

            println!(
                "[PAXOS-RETRY-SCHED] Release held {:?} ballot={:?} highest={:?} queue={}",
                msg.msg_type,
                paxos_ballot(&msg),
                highest_ballot(queue),
                queue.len(),
            );

            return SchedulerOutcome::Deliver(msg);
        }

        return deliver(queue, 0);

        //SchedulerOutcome::Deliver(queue.remove(0))
    }
}

pub struct PaxosOverlapScheduler {
    pub quorum_size: usize,
    pub delayed_accepted: Vec<Message>,
    pub held_lower_ballot: Vec<Message>,
    pub accepted_seen: HashMap<u64, usize>,
    pub max_delay: usize,
    pub delays_used: usize,
}

fn has_lower_ballot(queue: &[Message], ballot: u64) -> bool {
    queue
        .iter()
        .any(|m| paxos_ballot(m).map(|b| b < ballot).unwrap_or(false))
}

impl Scheduler for PaxosOverlapScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            if !self.held_lower_ballot.is_empty() {
                let msg = self.held_lower_ballot.remove(0);

                println!(
                    "[PAXOS-OVERLAP] Release lower-ballot {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                return SchedulerOutcome::Deliver(msg);
            }

            if !self.delayed_accepted.is_empty() {
                let msg = self.delayed_accepted.remove(0);

                println!(
                    "[PAXOS-OVERLAP] Release held accepted {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                return SchedulerOutcome::Deliver(msg);
            }

            return SchedulerOutcome::Empty;
        }

        // Rule 1:
        // Hold lower-ballot messages when higher-ballot messages exist.
        // This creates ballot overlap by allowing higher ballots to advance first.
        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(|msg| {
                if let Some(b) = paxos_ballot(msg) {
                    has_higher_ballot(queue, b)
                } else {
                    false
                }
            }) {
                let msg = queue.remove(pos);

                println!(
                    "[PAXOS-OVERLAP] Hold lower-ballot {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                self.held_lower_ballot.push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    return deliver(queue, 0);
                }
            }
        }

        // Rule 2:
        // Delay quorum-completing Accepted(b) if lower-ballot traffic remains.
        // This prevents the current ballot from deciding too early.
        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(|msg| {
                if let MessageType::Accepted { ballot, .. } = &msg.msg_type {
                    let seen = *self.accepted_seen.get(ballot).unwrap_or(&0);
                    let would_complete_quorum = seen + 1 >= self.quorum_size;

                    would_complete_quorum && has_lower_ballot(queue, *ballot)
                } else {
                    false
                }
            }) {
                let msg = queue.remove(pos);

                println!(
                    "[PAXOS-OVERLAP] Hold quorum Accepted {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                self.delayed_accepted.push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    return deliver(queue, 0);
                }
            }
        }

        // Release a held lower-ballot message while newer ballot traffic still exists.
        // This creates overlap before the newer ballot fully drains.
        if !self.held_lower_ballot.is_empty() {
            if let Some(highest) = highest_ballot(queue) {
                let msg = self.held_lower_ballot.remove(0);

                println!(
                    "[PAXOS-OVERLAP] Early release lower-ballot {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    Some(highest),
                    queue.len(),
                );

                return SchedulerOutcome::Deliver(msg);
            }
        }

        // Rule 3:
        // Prefer higher-ballot messages, so newer ballots advance before old messages are released.
        if let Some(max_ballot) = queue.iter().filter_map(paxos_ballot).max() {
            if let Some(pos) = queue
                .iter()
                .position(|m| paxos_ballot(m) == Some(max_ballot))
            {
                let msg = queue.remove(pos);

                if let MessageType::Accepted { ballot, .. } = &msg.msg_type {
                    let count = self.accepted_seen.entry(*ballot).or_insert(0);
                    *count += 1;
                }

                println!(
                    "[PAXOS-OVERLAP] Deliver {:?} ballot={:?} highest={:?} queue={}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                    queue.len(),
                );

                return SchedulerOutcome::Deliver(msg);
            }
        }

        // Rule 4:
        // Release held lower-ballot messages after higher-ballot progress.
        /* if !self.held_lower_ballot.is_empty() {
            let msg = self.held_lower_ballot.remove(0);

            println!(
                "[PAXOS-OVERLAP] Release lower-ballot {:?} ballot={:?} highest={:?} queue={}",
                msg.msg_type,
                paxos_ballot(&msg),
                highest_ballot(queue),
                queue.len(),
            );

            return SchedulerOutcome::Deliver(msg);
        }*/

        if !self.delayed_accepted.is_empty() {
            let msg = self.delayed_accepted.remove(0);

            println!(
                "[PAXOS-OVERLAP] Release held accepted {:?} ballot={:?} highest={:?} queue={}",
                msg.msg_type,
                paxos_ballot(&msg),
                highest_ballot(queue),
                queue.len(),
            );

            return SchedulerOutcome::Deliver(msg);
        }

        deliver(queue, 0)
    }
}

pub struct PaxosProgressScheduler {
    pub quorum_size: usize,
    pub held_lower_ballot: Vec<Message>,
    pub promise_seen: HashMap<u64, usize>,
    pub max_delay: usize,
    pub delays_used: usize,
}

impl Scheduler for PaxosProgressScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            if !self.held_lower_ballot.is_empty() {
                let msg = self.held_lower_ballot.remove(0);
                println!(
                    "[PAXOS-PROGRESS] Release at empty {:?} ballot={:?}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                );
                return SchedulerOutcome::Deliver(msg);
            }
            return SchedulerOutcome::Empty;
        }

        // Hold lower-ballot messages while higher ballot exists.
        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(|msg| {
                if let Some(b) = paxos_ballot(msg) {
                    has_higher_ballot(queue, b)
                } else {
                    false
                }
            }) {
                let msg = queue.remove(pos);
                println!(
                    "[PAXOS-PROGRESS] Hold lower {:?} ballot={:?} highest={:?}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                );
                self.held_lower_ballot.push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    return deliver(queue, 0);
                }
            }
        }

        // Prefer higher-ballot messages.
        if let Some(max_ballot) = queue.iter().filter_map(paxos_ballot).max() {
            if let Some(pos) = queue
                .iter()
                .position(|m| paxos_ballot(m) == Some(max_ballot))
            {
                let msg = queue.remove(pos);

                if let MessageType::Promise { ballot, .. } = &msg.msg_type {
                    let count = self.promise_seen.entry(*ballot).or_insert(0);
                    *count += 1;

                    println!(
                        "[PAXOS-PROGRESS] Promise progress ballot={} count={} quorum={}",
                        ballot, count, self.quorum_size,
                    );

                    if *count >= self.quorum_size && !self.held_lower_ballot.is_empty() {
                        queue.insert(0, msg);

                        let held = self.held_lower_ballot.remove(0);
                        println!(
                            "[PAXOS-PROGRESS] Release lower after promise quorum {:?} ballot={:?}",
                            held.msg_type,
                            paxos_ballot(&held),
                        );
                        return SchedulerOutcome::Deliver(held);
                    }
                }

                println!(
                    "[PAXOS-PROGRESS] Deliver {:?} ballot={:?} highest={:?}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                    highest_ballot(queue),
                );

                return SchedulerOutcome::Deliver(msg);
            }
        }

        if !self.held_lower_ballot.is_empty() {
            let msg = self.held_lower_ballot.remove(0);
            println!(
                "[PAXOS-PROGRESS] Release fallback {:?} ballot={:?}",
                msg.msg_type,
                paxos_ballot(&msg),
            );
            return SchedulerOutcome::Deliver(msg);
        }

        deliver(queue, 0)
    }
}

pub struct PaxosBallotOverlapScheduler {
    pub max_delay: usize,
    pub delays_used: usize,
    pub held: Vec<Message>,
    pub promise_seen: HashMap<u64, usize>,
}

fn has_multiple_ballots(queue: &[Message]) -> bool {
    let mut ballots = queue.iter().filter_map(paxos_ballot).collect::<Vec<u64>>();

    ballots.sort();
    ballots.dedup();

    ballots.len() >= 2
}

impl Scheduler for PaxosBallotOverlapScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            if !self.held.is_empty() {
                let msg = self.held.remove(0);

                println!(
                    "[BALLOT-OVERLAP] Release held {:?} ballot={:?}",
                    msg.msg_type,
                    paxos_ballot(&msg),
                );

                return SchedulerOutcome::Deliver(msg);
            }

            return SchedulerOutcome::Empty;
        }

        // 1. If multiple ballots exist, hold some lower-ballot traffic.
        // This prevents the lower ballot from draining cleanly.
        if self.delays_used < self.max_delay && has_multiple_ballots(queue) {
            if let Some(max_ballot) = queue.iter().filter_map(paxos_ballot).max() {
                if let Some(pos) = queue
                    .iter()
                    .position(|msg| paxos_ballot(msg).map(|b| b < max_ballot).unwrap_or(false))
                {
                    let msg = queue.remove(pos);

                    println!(
                        "[BALLOT-OVERLAP] Hold lower {:?} ballot={:?} max={}",
                        msg.msg_type,
                        paxos_ballot(&msg),
                        max_ballot,
                    );

                    self.held.push(msg);
                    self.delays_used += 1;

                    if !queue.is_empty() {
                        return deliver(queue, 0);
                    }
                }
            }
        }

        // 2. Prefer newer Prepare / Promise messages to start or advance new ballots.
        if let Some(pos) = queue.iter().position(|msg| {
            matches!(
                msg.msg_type,
                MessageType::Prepare { .. } | MessageType::Promise { .. }
            )
        }) {
            let msg = queue.remove(pos);

            if let MessageType::Promise { ballot, .. } = &msg.msg_type {
                let count = self.promise_seen.entry(*ballot).or_insert(0);
                *count += 1;
            }

            println!(
                "[BALLOT-OVERLAP] Deliver progress {:?} ballot={:?} held={}",
                msg.msg_type,
                paxos_ballot(&msg),
                self.held.len(),
            );

            return SchedulerOutcome::Deliver(msg);
        }

        // 3. If a held lower-ballot message exists, release it now to create collision.
        if !self.held.is_empty() {
            let msg = self.held.remove(0);

            println!(
                "[BALLOT-OVERLAP] Release collision {:?} ballot={:?}",
                msg.msg_type,
                paxos_ballot(&msg),
            );

            return SchedulerOutcome::Deliver(msg);
        }

        // 4. Otherwise deliver normally.
        deliver(queue, 0)
    }
}

pub struct PaxosGapOneScheduler {
    pub max_delay: usize,
    pub delays_used: usize,
    pub held_by_ballot: HashMap<u64, Vec<Message>>,
    pub prepare_seen: HashMap<u64, usize>,
    pub quorum_size: usize,

    pub max_held_backlog: usize,
    pub held_inserts: usize,
    pub held_releases: usize,
}

fn is_gap1_stale_candidate(msg: &Message, queue: &[Message]) -> bool {
    if let Some(b) = paxos_ballot(msg) {
        matches!(
            msg.msg_type,
            MessageType::Prepare { .. }
                | MessageType::Promise { .. }
                | MessageType::AcceptRequest { .. }
        ) && queue.iter().any(|m| paxos_ballot(m) == Some(b + 1))
    } else {
        false
    }
}

impl Scheduler for PaxosGapOneScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        println!(
            "[GAP1-STATS] max_backlog={} inserts={} releases={}",
            self.max_held_backlog, self.held_inserts, self.held_releases
        );

        if queue.is_empty() {
            let ballots = self.held_by_ballot.keys().cloned().collect::<Vec<_>>();

            for ballot in ballots {
                if let Some(vec) = self.held_by_ballot.get_mut(&ballot) {
                    if !vec.is_empty() {
                        let msg = vec.remove(0);

                        println!(
                            "[GAP1] Release at empty {:?} ballot={:?}",
                            msg.msg_type,
                            paxos_ballot(&msg)
                        );

                        return SchedulerOutcome::Deliver(msg);
                    }
                }
            }

            return SchedulerOutcome::Empty;
        }

        // 1. Hold stale b messages when b+1 exists.
        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(|m| is_gap1_stale_candidate(m, queue)) {
                let msg = queue.remove(pos);
                let b = paxos_ballot(&msg).unwrap();

                println!(
                    "[GAP1] Hold stale {:?} ballot={} waiting_for_prepare_quorum={}",
                    msg.msg_type,
                    b,
                    b + 1
                );

                self.held_by_ballot.entry(b).or_default().push(msg);
                self.delays_used += 1;

                self.held_inserts += 1;
                self.max_held_backlog = self.max_held_backlog.max(total_held(&self.held_by_ballot));

                if !queue.is_empty() {
                    return deliver(queue, 0);
                }
            }
        }

        // 2. Prefer Prepare messages; they advance promised_ballot.
        if let Some(pos) = queue
            .iter()
            .position(|m| matches!(m.msg_type, MessageType::Prepare { .. }))
        {
            let msg = queue.remove(pos);

            let ballot = match &msg.msg_type {
                MessageType::Prepare { ballot } => *ballot,
                _ => unreachable!(),
            };

            let count = self.prepare_seen.entry(ballot).or_insert(0);
            *count += 1;

            println!(
                "[GAP1] Deliver Prepare ballot={} count={} quorum={}",
                ballot, count, self.quorum_size
            );

            let stale_ballot = ballot.saturating_sub(1);

            if *count >= self.quorum_size {
                if let Some(held_vec) = self.held_by_ballot.get_mut(&stale_ballot) {
                    if !held_vec.is_empty() {
                        queue.insert(0, msg);

                        let held = held_vec.remove(0);

                        println!(
                            "[GAP1] Release stale after Prepare quorum {:?} stale_ballot={} newer_ballot={}",
                            held.msg_type, stale_ballot, ballot
                        );

                        return SchedulerOutcome::Deliver(held);
                    }
                }
            }

            return SchedulerOutcome::Deliver(msg);
        }

        // 3. If no Prepare exists, release any held message whose b+1 already reached quorum.
        let ballots = self.held_by_ballot.keys().cloned().collect::<Vec<_>>();

        for b in ballots {
            let newer = b + 1;

            let ready = self
                .prepare_seen
                .get(&newer)
                .map(|c| *c >= self.quorum_size)
                .unwrap_or(false);

            if ready {
                if let Some(vec) = self.held_by_ballot.get_mut(&b) {
                    if !vec.is_empty() {
                        let msg = vec.remove(0);

                        println!(
                            "[GAP1] Release stale fallback {:?} stale_ballot={} newer_ballot={}",
                            msg.msg_type, b, newer
                        );

                        return SchedulerOutcome::Deliver(msg);
                    }
                }
            }
        }

        if !self.held_by_ballot.is_empty() && queue.len() <= 1 {
            let ballots = self.held_by_ballot.keys().cloned().collect::<Vec<_>>();
            self.held_releases += 1;

            for b in ballots {
                if let Some(vec) = self.held_by_ballot.get_mut(&b) {
                    if !vec.is_empty() {
                        let msg = vec.remove(0);

                        println!(
                            "[GAP1] Deadlock fallback release {:?} ballot={:?}",
                            msg.msg_type,
                            paxos_ballot(&msg)
                        );

                        return SchedulerOutcome::Deliver(msg);
                    }
                }
            }
        }

        deliver(queue, 0)
    }
}

pub struct PaxosGapOneBacklogScheduler {
    pub max_delay: usize,
    pub delays_used: usize,
    pub held_by_ballot: HashMap<u64, Vec<Message>>,
    pub prepare_seen: HashMap<u64, usize>,
    pub quorum_size: usize,
    pub release_after_held: usize,
}

fn total_held(held: &HashMap<u64, Vec<Message>>) -> usize {
    held.values().map(|v| v.len()).sum()
}

impl Scheduler for PaxosGapOneBacklogScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            let ballots = self.held_by_ballot.keys().cloned().collect::<Vec<_>>();

            for b in ballots {
                if let Some(vec) = self.held_by_ballot.get_mut(&b) {
                    if !vec.is_empty() {
                        let msg = vec.remove(0);

                        println!(
                            "[GAP1-BACKLOG] Release at empty {:?} ballot={:?}",
                            msg.msg_type,
                            paxos_ballot(&msg)
                        );

                        return SchedulerOutcome::Deliver(msg);
                    }
                }
            }

            return SchedulerOutcome::Empty;
        }

        // 1. Hold stale b messages when b+1 exists.
        if self.delays_used < self.max_delay {
            if let Some(pos) = queue.iter().position(|m| is_gap1_stale_candidate(m, queue)) {
                let msg = queue.remove(pos);
                let b = paxos_ballot(&msg).unwrap();

                println!(
                    "[GAP1-BACKLOG] Hold stale {:?} ballot={} total_held={}",
                    msg.msg_type,
                    b,
                    total_held(&self.held_by_ballot) + 1
                );

                self.held_by_ballot.entry(b).or_default().push(msg);
                self.delays_used += 1;

                if !queue.is_empty() {
                    return deliver(queue, 0);
                }
            }
        }

        // 2. Prefer Prepare messages to keep generating newer ballots.
        if let Some(pos) = queue
            .iter()
            .position(|m| matches!(m.msg_type, MessageType::Prepare { .. }))
        {
            let msg = queue.remove(pos);

            let ballot = match &msg.msg_type {
                MessageType::Prepare { ballot } => *ballot,
                _ => unreachable!(),
            };

            let count = self.prepare_seen.entry(ballot).or_insert(0);
            *count += 1;

            println!(
                "[GAP1-BACKLOG] Deliver Prepare ballot={} count={} quorum={} total_held={}",
                ballot,
                count,
                self.quorum_size,
                total_held(&self.held_by_ballot)
            );

            return SchedulerOutcome::Deliver(msg);
        }

        // 3. Release only after enough stale backlog has accumulated.
        if total_held(&self.held_by_ballot) >= self.release_after_held {
            let mut ballots = self.held_by_ballot.keys().cloned().collect::<Vec<_>>();
            ballots.sort();

            for b in ballots {
                let newer = b + 1;
                let ready = self
                    .prepare_seen
                    .get(&newer)
                    .map(|c| *c >= self.quorum_size)
                    .unwrap_or(false);

                if ready {
                    if let Some(vec) = self.held_by_ballot.get_mut(&b) {
                        if !vec.is_empty() {
                            let msg = vec.remove(0);

                            println!(
                                "[GAP1-BACKLOG] Release stale {:?} stale_ballot={} newer_ballot={} total_held={}",
                                msg.msg_type,
                                b,
                                newer,
                                total_held(&self.held_by_ballot)
                            );

                            return SchedulerOutcome::Deliver(msg);
                        }
                    }
                }
            }
        }

        // 4. Otherwise deliver normally.
        deliver(queue, 0)
    }
}

#[derive(Debug, Clone)]
pub struct UniformBudgetDelayScheduler {
    total_budget: usize,
    remaining_budget: usize,
    spent_budget: usize,
    pub delayed_prepare: u64,
    pub delayed_promise: u64,
    pub delayed_accept_request: u64,
    pub delayed_accepted: u64,
    pub delayed_other: u64,
}

impl UniformBudgetDelayScheduler {
    pub fn new(total_budget: usize) -> Self {
        Self {
            total_budget,
            remaining_budget: total_budget,
            spent_budget: 0,
            delayed_prepare: 0,
            delayed_promise: 0,
            delayed_accept_request: 0,
            delayed_accepted: 0,
            delayed_other: 0,
        }
    }

    pub fn total_budget(&self) -> usize {
        self.total_budget
    }

    pub fn remaining_budget(&self) -> usize {
        self.remaining_budget
    }

    pub fn spent_budget(&self) -> usize {
        self.spent_budget
    }

    fn spend_one(&mut self) {
        if self.remaining_budget > 0 {
            self.remaining_budget -= 1;
            self.spent_budget += 1;
        }
    }
}

impl Scheduler for UniformBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.remaining_budget > 0 {
            self.spend_one();

            let mut rng = rand::rng();
            let idx = rng.random_range(0..queue.len());
            let msg = queue.remove(idx);

            println!(
                "[UNIFORM] idx={} queue_len={} delayed={:?}",
                idx,
                queue.len(),
                msg.msg_type
            );

            match &msg.msg_type {
                MessageType::Prepare { .. } => self.delayed_prepare += 1,
                MessageType::Promise { .. } => self.delayed_promise += 1,
                MessageType::AcceptRequest { .. } => self.delayed_accept_request += 1,
                MessageType::Accepted { .. } => self.delayed_accepted += 1,
                _ => self.delayed_other += 1,
            }

            let msg_type = format!("{:?}", msg.msg_type);
            queue.push(msg);

            println!(
                "[UNIFORM-BUDGET-DELAY] spent={} remaining={} total={} queue_len={} delayed={}",
                self.spent_budget,
                self.remaining_budget,
                self.total_budget,
                queue.len(),
                msg_type
            );

            return SchedulerOutcome::Delay;
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

#[derive(Debug, Clone)]
pub struct TargetedBudgetDelayScheduler {
    total_budget: usize,
    remaining_budget: usize,
    spent_budget: usize,
    pub delayed_prepare: u64,
    pub delayed_promise: u64,
    pub delayed_accept_request: u64,
    pub delayed_accepted: u64,
    pub delayed_other: u64,
}

impl TargetedBudgetDelayScheduler {
    pub fn new(total_budget: usize) -> Self {
        Self {
            total_budget,
            remaining_budget: total_budget,
            spent_budget: 0,
            delayed_prepare: 0,
            delayed_promise: 0,
            delayed_accept_request: 0,
            delayed_accepted: 0,
            delayed_other: 0,
        }
    }

    fn spend_one(&mut self) {
        if self.remaining_budget > 0 {
            self.remaining_budget -= 1;
            self.spent_budget += 1;
        }
    }

    fn is_critical(msg: &Message) -> bool {
        matches!(
            msg.msg_type,
            MessageType::Promise { .. } | MessageType::Accepted { .. }
        )
    }
}

impl Scheduler for TargetedBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.remaining_budget > 0 && Self::is_critical(&queue[0]) {
            self.spend_one();

            let msg = queue.remove(0);
            match &msg.msg_type {
                MessageType::Prepare { .. } => self.delayed_prepare += 1,

                MessageType::Promise { .. } => self.delayed_promise += 1,

                MessageType::AcceptRequest { .. } => {
                    self.delayed_accept_request += 1;
                }

                MessageType::Accepted { .. } => {
                    self.delayed_accepted += 1;
                }

                _ => self.delayed_other += 1,
            }
            let msg_type = format!("{:?}", msg.msg_type);
            queue.push(msg);

            println!(
                "[TARGETED-BUDGET-DELAY] spent={} remaining={} total={} queue_len={} delayed={}",
                self.spent_budget,
                self.remaining_budget,
                self.total_budget,
                queue.len(),
                msg_type
            );

            println!("==========================");
            println!("Targeted Delay Profile");
            println!("Prepare       : {}", self.delayed_prepare);
            println!("Promise       : {}", self.delayed_promise);
            println!("AcceptRequest : {}", self.delayed_accept_request);
            println!("Accepted      : {}", self.delayed_accepted);
            println!("Other         : {}", self.delayed_other);
            println!("==========================");

            return SchedulerOutcome::Delay;
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}
pub struct InterleavedUniformBudgetDelayScheduler {
    total_budget: usize,
    remaining_budget: usize,
    spent_budget: usize,
    delay_every: usize,
    step: usize,
}

impl InterleavedUniformBudgetDelayScheduler {
    pub fn new(total_budget: usize, delay_every: usize) -> Self {
        Self {
            total_budget,
            remaining_budget: total_budget,
            spent_budget: 0,
            delay_every,
            step: 0,
        }
    }

    fn should_delay(&mut self) -> bool {
        self.step += 1;
        self.remaining_budget > 0 && self.step % self.delay_every == 0
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }
}

impl Scheduler for InterleavedUniformBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.should_delay() {
            self.spend_one();

            let mut rng = rand::rng();
            let idx = rng.random_range(0..queue.len());

            let msg = queue.remove(idx);
            let msg_type = format!("{:?}", msg.msg_type);
            queue.push(msg);

            println!(
                "[INTERLEAVED-UNIFORM-DELAY] step={} spent={} remaining={} queue_len={} delayed={}",
                self.step,
                self.spent_budget,
                self.remaining_budget,
                queue.len(),
                msg_type
            );

            return SchedulerOutcome::Delay;
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}



pub struct InterleavedTargetedBudgetDelayScheduler {
    total_budget: usize,
    remaining_budget: usize,
    spent_budget: usize,
    delay_every: usize,
    step: usize,
}

impl InterleavedTargetedBudgetDelayScheduler {
    pub fn new(total_budget: usize, delay_every: usize) -> Self {
        Self {
            total_budget,
            remaining_budget: total_budget,
            spent_budget: 0,
            delay_every,
            step: 0,
        }
    }

    fn should_delay(&mut self) -> bool {
        self.step += 1;
        self.remaining_budget > 0 && self.step % self.delay_every == 0
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }

    fn is_critical(msg: &Message) -> bool {
        matches!(
            msg.msg_type,
            MessageType::Promise { .. }
                | MessageType::Accepted { .. }
                | MessageType::Nack { .. }
        )
    }
}

impl Scheduler for InterleavedTargetedBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.should_delay() {
            if let Some(idx) = queue.iter().position(Self::is_critical) {
                self.spend_one();

                let msg = queue.remove(idx);
                let msg_type = format!("{:?}", msg.msg_type);
                queue.push(msg);

                println!(
                    "[INTERLEAVED-TARGETED-DELAY] step={} spent={} remaining={} queue_len={} delayed={}",
                    self.step,
                    self.spent_budget,
                    self.remaining_budget,
                    queue.len(),
                    msg_type
                );

                return SchedulerOutcome::Delay;
            }
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

pub struct InterleavedProgressTargetedBudgetDelayScheduler {
    total_budget: usize,
    remaining_budget: usize,
    spent_budget: usize,
    delay_every: usize,
    step: usize,
}

impl InterleavedProgressTargetedBudgetDelayScheduler {
    pub fn new(total_budget: usize, delay_every: usize) -> Self {
        Self {
            total_budget,
            remaining_budget: total_budget,
            spent_budget: 0,
            delay_every,
            step: 0,
        }
    }

    fn should_delay(&mut self) -> bool {
        self.step += 1;
        self.remaining_budget > 0 && self.step % self.delay_every == 0
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }

    fn is_progress_critical(msg: &Message) -> bool {
        matches!(
            msg.msg_type,
            MessageType::Prepare { .. }
                | MessageType::Promise { .. }
                | MessageType::AcceptRequest { .. }
                | MessageType::Accepted { .. }
                | MessageType::Nack { .. }
        )
    }
}

impl Scheduler for InterleavedProgressTargetedBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.should_delay() {
            if let Some(idx) = queue.iter().position(Self::is_progress_critical) {
                self.spend_one();

                let msg = queue.remove(idx);
                let msg_type = format!("{:?}", msg.msg_type);
                queue.push(msg);

                println!(
                    "[INTERLEAVED-PROGRESS-TARGETED-DELAY] step={} spent={} remaining={} queue_len={} delayed={}",
                    self.step,
                    self.spent_budget,
                    self.remaining_budget,
                    queue.len(),
                    msg_type
                );

                return SchedulerOutcome::Delay;
            }
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

pub struct ProbInterleavedUniformBudgetDelayScheduler {
    remaining_budget: usize,
    spent_budget: usize,
    delay_probability: f64,
    rng: StdRng,
}

impl ProbInterleavedUniformBudgetDelayScheduler {
    pub fn new(total_budget: usize, p: f64, seed: u64) -> Self {
        Self {
            remaining_budget: total_budget,
            spent_budget: 0,
            delay_probability: p,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }
}

impl Scheduler for ProbInterleavedUniformBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

       //et mut rng = rand::rng();

        let should_delay =
            self.remaining_budget > 0 &&
            self.rng.random_bool(self.delay_probability);

        if should_delay {
            self.spend_one();

            let idx = self.rng.random_range(0..queue.len());

            let msg = queue.remove(idx);
            let msg_type = format!("{:?}", msg.msg_type);
            queue.push(msg);

            println!(
                "[PROB-INTERLEAVED-UNIFORM] spent={} remaining={} queue_len={} delayed={}",
                self.spent_budget,
                self.remaining_budget,
                queue.len(),
                msg_type
            );

            return SchedulerOutcome::Delay;
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

pub struct ProbInterleavedTargetedBudgetDelayScheduler {
    remaining_budget: usize,
    spent_budget: usize,
    delay_probability: f64,
    rng: rand::rngs::StdRng,
}







impl ProbInterleavedTargetedBudgetDelayScheduler {
    pub fn new(total_budget: usize, delay_probability: f64, seed: u64) -> Self {
        Self {
            remaining_budget: total_budget,
            spent_budget: 0,
            delay_probability,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }

    fn is_critical(msg: &Message) -> bool {
        matches!(
            msg.msg_type,
            MessageType::Promise { .. }
                | MessageType::Accepted { .. }
        )
    }

   fn random_critical_index(
        queue: &[Message],
        rng: &mut impl rand::Rng,
    ) -> Option<usize> {
        let critical_indices: Vec<usize> = queue
            .iter()
            .enumerate()
            .filter_map(|(i, msg)| {
                if Self::is_critical(msg) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if critical_indices.is_empty() {
            None
        } else {
            let j = rng.random_range(0..critical_indices.len());
            Some(critical_indices[j])
        }
    }
}

impl Scheduler for ProbInterleavedTargetedBudgetDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }
        
        let should_delay =
            self.remaining_budget > 0 &&
            self.rng.random_bool(self.delay_probability);

        if should_delay {
            self.spend_one();

            let idx = if let Some(idx) = Self::random_critical_index(queue, &mut self.rng) {
                idx
            } else {
                self.rng.random_range(0..queue.len())
            };

            let msg = queue.remove(idx);
            let msg_type = format!("{:?}", msg.msg_type);

            queue.push(msg);

            println!(
                "[PROB-INTERLEAVED-TARGETED] spent={} remaining={} queue_len={} delayed={}",
                self.spent_budget,
                self.remaining_budget,
                queue.len(),
                msg_type
            );

            return SchedulerOutcome::Delay;
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

pub struct DeadlineAwareQuorumDelayScheduler {
    remaining_budget: usize,
    spent_budget: usize,
    quorum_size: usize,
    rng: StdRng,
}

impl DeadlineAwareQuorumDelayScheduler {
    pub fn new(total_budget: usize, quorum_size: usize, seed: u64) -> Self {
        Self {
            remaining_budget: total_budget,
            spent_budget: 0,
            quorum_size,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }

    fn is_promise(msg: &Message) -> bool {
        matches!(msg.msg_type, MessageType::Promise { .. })
    }

    fn is_accepted(msg: &Message) -> bool {
        matches!(msg.msg_type, MessageType::Accepted { .. })
    }

    fn count_deliverable_before_idx<F>(
        queue: &[Message],
        idx: usize,
        predicate: F,
    ) -> usize
    where
        F: Fn(&Message) -> bool,
    {
        queue[..idx].iter().filter(|m| predicate(m)).count()
    }

    fn find_quorum_blocking_message(&self, queue: &[Message]) -> Option<usize> {
        for (idx, msg) in queue.iter().enumerate() {
            match msg.msg_type {
                MessageType::Promise { .. } => {
                    let before =
                        Self::count_deliverable_before_idx(queue, idx, Self::is_promise);

                    if before + 1 >= self.quorum_size {
                        return Some(idx);
                    }
                }

                MessageType::Accepted { .. } => {
                    let before =
                        Self::count_deliverable_before_idx(queue, idx, Self::is_accepted);

                    if before + 1 >= self.quorum_size {
                        return Some(idx);
                    }
                }

                _ => {}
            }
        }

        None
    }
}

impl Scheduler for DeadlineAwareQuorumDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.remaining_budget > 0 {
            if let Some(idx) = self.find_quorum_blocking_message(queue) {
                self.spend_one();

                let msg = queue.remove(idx);
                let msg_type = format!("{:?}", msg.msg_type);
                queue.push(msg);

                println!(
                    "[DEADLINE-QUORUM-DELAY] spent={} remaining={} queue_len={} delayed={}",
                    self.spent_budget,
                    self.remaining_budget,
                    queue.len(),
                    msg_type
                );

                return SchedulerOutcome::Delay;
            }
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}

pub struct BoundedQuorumUsefulDelayScheduler {
    remaining_budget: usize,
    spent_budget: usize,
    quorum_size: usize,
    max_consecutive_delay: u64,
    consecutive_delay: HashMap<String, u64>,
    rng: StdRng,
}

impl BoundedQuorumUsefulDelayScheduler {
    pub fn new(
        total_budget: usize,
        quorum_size: usize,
        max_consecutive_delay: u64,
        proposer_id: u64,
        seed: u64,
    ) -> Self {
        Self {
            remaining_budget: total_budget,
            spent_budget: 0,
            quorum_size,
            max_consecutive_delay,
            consecutive_delay: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn is_promise_for(msg: &Message, ballot: u64, proposer_id: u64) -> bool {
        matches!(
            msg.msg_type,
            MessageType::Promise { ballot: b, .. } if b == ballot && msg.to == proposer_id
        )
    }

    fn is_accepted_for(msg: &Message, ballot: u64, proposer_id: u64) -> bool {
        matches!(
            msg.msg_type,
            MessageType::Accepted { ballot: b, .. } if b == ballot && msg.to == proposer_id
        )
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }

    fn message_key(msg: &Message) -> String {
        format!("{}-{}-{:?}", msg.from, msg.to, msg.msg_type)
    }

    fn can_delay(&self, msg: &Message) -> bool {
        let key = Self::message_key(msg);
        self.consecutive_delay.get(&key).copied().unwrap_or(0)
            < self.max_consecutive_delay
    }

    fn record_delay(&mut self, msg: &Message) {
        let key = Self::message_key(msg);
        let count = self.consecutive_delay.entry(key.clone()).or_insert(0);
        *count += 1;

        println!(
            "[CAP-TRACE] key={} count={} max={}",
            key,
            count,
            self.max_consecutive_delay
        );
    }

    fn record_delivery(&mut self, msg: &Message) {
        let key = Self::message_key(msg);
        self.consecutive_delay.remove(&key);
    }

    fn ballot(msg: &Message) -> Option<u64> {
        match msg.msg_type {
            MessageType::Prepare { ballot }
            | MessageType::Promise { ballot, .. }
            | MessageType::AcceptRequest { ballot, .. }
            | MessageType::Accepted { ballot, .. }
            | MessageType::Nack { ballot, .. } => Some(ballot),
            _ => None,
        }
    }

    fn latest_ballot(queue: &[Message]) -> Option<u64> {
        queue.iter().filter_map(Self::ballot).max()
    }

    

    fn count_before<F>(queue: &[Message], idx: usize, pred: F) -> usize
    where
        F: Fn(&Message) -> bool,
    {
        queue[..idx].iter().filter(|m| pred(m)).count()
    }

    fn find_candidate(&mut self, queue: &[Message]) -> Option<usize> {
        let latest = Self::latest_ballot(queue)?;

        let mut candidates = Vec::new();
        let proposer_id = 1;

        for (idx, msg) in queue.iter().enumerate() {
            match msg.msg_type {
                MessageType::Promise { ballot, .. } if ballot == latest => {
                    let before =
                       Self::count_before(queue, idx, |m|
                            Self::is_promise_for(m, latest, proposer_id)
                        );

                    if before < self.quorum_size && self.can_delay(msg) {
                        candidates.push(idx);
                    }
                }

                MessageType::Accepted { ballot, .. } if ballot == latest => {
                    let before =
                       Self::count_before(queue, idx, |m|
                            Self::is_accepted_for(m, latest, proposer_id)
                        );

                    if before < self.quorum_size && self.can_delay(msg) {
                        candidates.push(idx);
                    }
                }

                _ => {}
            }
        }

        if candidates.is_empty() {
            None
        } else {
            let j = self.rng.random_range(0..candidates.len());
            Some(candidates[j])
        }
    }
}

impl Scheduler for BoundedQuorumUsefulDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.remaining_budget > 0 {
            if let Some(idx) = self.find_candidate(queue) {
                self.spend_one();

                let msg = queue.remove(idx);
                self.record_delay(&msg);

                let msg_type = format!("{:?}", msg.msg_type);
                queue.push(msg);

                println!(
                    "[BOUNDED-QUORUM-USEFUL-DELAY] spent={} remaining={} queue_len={} delayed={}",
                    self.spent_budget,
                    self.remaining_budget,
                    queue.len(),
                    msg_type
                );

                return SchedulerOutcome::Delay;
            }
        }

        let msg = queue.remove(0);
        self.record_delivery(&msg);

        if matches!(msg.msg_type, MessageType::Promise { .. }) {
            println!(
                "[BOUNDED-QUORUM-USEFUL-DELIVER] from={} to={} type={:?}",
                msg.from,
                msg.to,
                msg.msg_type
            );
        }

        SchedulerOutcome::Deliver(msg)
    }
}

pub struct ProgressAwareQuorumDelayScheduler {
    remaining_budget: usize,
    spent_budget: usize,
    quorum_size: usize,
    max_consecutive_delay: u64,
    consecutive_delay: HashMap<String, u64>,
    delay_limit: HashMap<String, u64>,
    delivered_promises: HashMap<u64, HashSet<u64>>,
    delivered_accepted: HashMap<u64, HashSet<u64>>,
    rng: StdRng,
}

impl ProgressAwareQuorumDelayScheduler {
    pub fn new(
        total_budget: usize,
        quorum_size: usize,
        max_consecutive_delay: u64,
        seed: u64,
    ) -> Self {
        Self {
            remaining_budget: total_budget,
            spent_budget: 0,
            quorum_size,
            max_consecutive_delay,
            consecutive_delay: HashMap::new(),
            delay_limit: HashMap::new(),
            delivered_promises: HashMap::new(),
            delivered_accepted: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn spend_one(&mut self) {
        self.remaining_budget -= 1;
        self.spent_budget += 1;
    }

    fn message_key(msg: &Message) -> String {
        format!("{}-{}-{:?}", msg.from, msg.to, msg.msg_type)
    }

    fn can_delay(&mut self, msg: &Message) -> bool {
        let key = Self::message_key(msg);
        let count = self.consecutive_delay.get(&key).copied().unwrap_or(0);
        let limit = self.delay_limit_for(msg);

        count < limit
    }

    fn record_delay(&mut self, msg: &Message) {
        let key = Self::message_key(msg);
        let count = self.consecutive_delay.entry(key.clone()).or_insert(0);
        *count += 1;

        let limit = self.delay_limit.get(&key).copied().unwrap_or(0);

        println!(
            "[PROGRESS-CAP] key={} count={} limit={} max={}",
            key,
            count,
            limit,
            self.max_consecutive_delay
        );
    }

    fn record_delivery(&mut self, msg: &Message) {
        let key = Self::message_key(msg);
        self.consecutive_delay.remove(&key);

        match msg.msg_type {
            MessageType::Promise { ballot, .. } => {
                self.delivered_promises
                    .entry(ballot)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);
            }

            MessageType::Accepted { ballot, .. } => {
                self.delivered_accepted
                    .entry(ballot)
                    .or_insert_with(HashSet::new)
                    .insert(msg.from);
            }

            _ => {}
        }
    }

    fn delay_limit_for(&mut self, msg: &Message) -> u64 {
        let key = Self::message_key(msg);

        if let Some(limit) = self.delay_limit.get(&key) {
            *limit
        } else {
            let limit = self.rng.random_range(0..=self.max_consecutive_delay);
            self.delay_limit.insert(key, limit);
            limit
        }
    }

    fn ballot(msg: &Message) -> Option<u64> {
        match msg.msg_type {
            MessageType::Prepare { ballot }
            | MessageType::Promise { ballot, .. }
            | MessageType::AcceptRequest { ballot, .. }
            | MessageType::Accepted { ballot, .. }
            | MessageType::Nack { ballot, .. } => Some(ballot),
            _ => None,
        }
    }

    fn latest_ballot(queue: &[Message]) -> Option<u64> {
        queue.iter().filter_map(Self::ballot).max()
    }

    fn promise_count(&self, ballot: u64) -> usize {
        self.delivered_promises
            .get(&ballot)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    fn accepted_count(&self, ballot: u64) -> usize {
        self.delivered_accepted
            .get(&ballot)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    fn find_quorum_forming_candidate(&mut self, queue: &[Message]) -> Option<usize> {
        let latest = Self::latest_ballot(queue)?;

        let mut candidates = Vec::new();

        for (idx, msg) in queue.iter().enumerate() {
            match msg.msg_type {
                MessageType::Promise { ballot, .. } if ballot == latest => {
                    let count = self.promise_count(ballot);

                    // Only delay the Promise that would complete quorum.
                    if count + 1 >= self.quorum_size && self.can_delay(msg) {
                        candidates.push(idx);
                    }
                }

                MessageType::Accepted { ballot, .. } if ballot == latest => {
                    let count = self.accepted_count(ballot);

                    // Only delay the Accepted that would complete quorum.
                    if count + 1 >= self.quorum_size && self.can_delay(msg) {
                        candidates.push(idx);
                    }
                }

                _ => {}
            }
        }

        if candidates.is_empty() {
            None
        } else {
            let j = self.rng.random_range(0..candidates.len());
            Some(candidates[j])
        }
    }
}

impl Scheduler for ProgressAwareQuorumDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if self.remaining_budget > 0 {
            if let Some(idx) = self.find_quorum_forming_candidate(queue) {
                self.spend_one();

                let msg = queue.remove(idx);
                self.record_delay(&msg);

                let msg_type = format!("{:?}", msg.msg_type);
                queue.push(msg);

                println!(
                    "[PROGRESS-QUORUM-DELAY] spent={} remaining={} queue_len={} delayed={}",
                    self.spent_budget,
                    self.remaining_budget,
                    queue.len(),
                    msg_type
                );

                return SchedulerOutcome::Delay;
            }
        }

        let msg = queue.remove(0);
        self.record_delivery(&msg);

        if matches!(msg.msg_type, MessageType::Promise { .. } | MessageType::Accepted { .. }) {
            println!(
                "[PROGRESS-QUORUM-DELIVER] from={} to={} type={:?}",
                msg.from,
                msg.to,
                msg.msg_type
            );
        }

        SchedulerOutcome::Deliver(msg)
    }
}