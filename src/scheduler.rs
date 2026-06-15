use crate::message::{Message, MessageType, VoteValue};
use rand::RngExt;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;



pub enum SchedulerOutcome {
    Deliver(Message),
    Delay,
    Empty,
}

pub trait Scheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome;
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
        Self { max_delay: max_delay }
    }
}

impl ProbabilisticDelayScheduler {
    pub fn new(max_delay: usize, seed: u64) -> Self {
        Self { max_delay: max_delay,
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
        Self { max_delays: max_delays,}
    }
}

impl Scheduler for FifoScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome {
        if queue.is_empty() {
            SchedulerOutcome::Empty
        } else {
            SchedulerOutcome::Deliver(queue.remove(0))
        }
    }
}

impl Scheduler for RandomScheduler {
    fn choose_next(
        &mut self,
        queue: &mut Vec<Message>,
    ) -> SchedulerOutcome {
        if queue.is_empty() {
            return SchedulerOutcome::Empty
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

            let key = (
                msg.to,
                msg.msg_type.clone(),
                msg.value.clone(),
            );

            let delivered_so_far = *self
                .delivered_counts
                .get(&key)
                .unwrap_or(&0);

            // If this would be the 3rd matching message,
            // delay it by moving it to the back.
            if delivered_so_far == 2 {
                queue.push(msg);
                continue;
            }

            self.delivered_counts.insert(
                key,
                delivered_so_far + 1,
            );

            return SchedulerOutcome::Deliver(msg);
        }

        // If every message would complete quorum,
        // force deliver one to avoid deadlock.
        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        } else {
            let msg = queue.remove(0);

            let key = (
                msg.to,
                msg.msg_type.clone(),
                msg.value.clone(),
            );

            let delivered_so_far = *self
                .delivered_counts
                .get(&key)
                .unwrap_or(&0);

            self.delivered_counts.insert(
                key,
                delivered_so_far + 1,
            );

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
            return SchedulerOutcome::Deliver(queue.remove(0))
        }
    }
}


impl Scheduler for DelayLeaderScheduler {
    fn choose_next(
        &mut self,
        queue: &mut Vec<Message>,
    ) -> SchedulerOutcome {

        if queue.is_empty() {
            return SchedulerOutcome::Empty;
        }

        if let Some(pos) = queue.iter().position(|msg| {
            msg.from != 1
        }) {
            return SchedulerOutcome::Deliver(queue.remove(pos));
        }

        SchedulerOutcome::Deliver(queue.remove(0))
    }
}


impl Scheduler for BoundedDelayLeaderScheduler {
    fn choose_next(
        &mut self,
        queue: &mut Vec<Message>,
    ) -> SchedulerOutcome {
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