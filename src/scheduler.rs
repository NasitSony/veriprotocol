use crate::message::{Message, MessageType, VoteValue};
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;

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

fn deliver(
    queue: &mut Vec<Message>,
    index: usize,
) -> SchedulerOutcome {
    let msg = queue.remove(index);

    println!(
        "[SCHED] Deliver {:?} ballot={:?} highest={:?} queue={}",
        msg.msg_type,
        paxos_ballot(&msg),
        highest_ballot(queue),
        queue.len(),
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

        SchedulerOutcome::Deliver(queue.remove(index))
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
        queue.iter().any(|m| {
            paxos_ballot(m)
                .map(|b| b > ballot)
                .unwrap_or(false)
        })
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

                return SchedulerOutcome::Deliver(msg)
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
            if let Some(pos) = queue.iter().position(|m| paxos_ballot(m) == Some(max_ballot)) {
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
    queue.iter().any(|m| {
        paxos_ballot(m)
            .map(|b| b < ballot)
            .unwrap_or(false)
    })
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
            if let Some(pos) = queue.iter().position(|m| paxos_ballot(m) == Some(max_ballot)) {
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
            if let Some(pos) = queue.iter().position(|m| paxos_ballot(m) == Some(max_ballot)) {
                let msg = queue.remove(pos);

                if let MessageType::Promise { ballot, .. } = &msg.msg_type {
                    let count = self.promise_seen.entry(*ballot).or_insert(0);
                    *count += 1;

                    println!(
                        "[PAXOS-PROGRESS] Promise progress ballot={} count={} quorum={}",
                        ballot,
                        count,
                        self.quorum_size,
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
    let mut ballots = queue
        .iter()
        .filter_map(paxos_ballot)
        .collect::<Vec<u64>>();

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
                if let Some(pos) = queue.iter().position(|msg| {
                    paxos_ballot(msg)
                        .map(|b| b < max_ballot)
                        .unwrap_or(false)
                }) {
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




