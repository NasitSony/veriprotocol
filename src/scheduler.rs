use crate::message::{Message, MessageType, VoteValue};
use rand::RngExt;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;

pub trait Scheduler {
    fn choose_next(
        &mut self,
        queue: &mut Vec<Message>,
    ) -> Option<Message>;
}

pub struct FifoScheduler;
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

impl Scheduler for FifoScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }
}

impl Scheduler for RandomScheduler {
    fn choose_next(
        &mut self,
        queue: &mut Vec<Message>,
    ) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let index = self.rng.random_range(0..queue.len());

        Some(queue.remove(index))
    }
}

impl Scheduler for DelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.to == self.delayed_node {
                queue.push(msg);
            } else {
                return Some(msg);
            }
        }

        // If every message targets delayed_node, eventually deliver one.
        Some(queue.remove(0))
    }
}

impl Scheduler for CommitDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.msg_type == MessageType::Commit {
                queue.push(msg);
            } else {
                return Some(msg);
            }
        }

        Some(queue.remove(0))
    }
}

impl Scheduler for VoteDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.msg_type == MessageType::Vote {
                queue.push(msg);
            } else {
                return Some(msg);
            }
        }

        Some(queue.remove(0))
    }
}

impl Scheduler for ProposalDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let len = queue.len();

        for _ in 0..len {
            let msg = queue.remove(0);

            if msg.msg_type == MessageType::Proposal {
                queue.push(msg);
            } else {
                return Some(msg);
            }
        }

        Some(queue.remove(0))
    }
}

impl Scheduler for BoundedDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let len = queue.len();

        for _ in 0..len {
            let mut msg = queue.remove(0);

            if msg.delay_count < self.max_delay {
                msg.delay_count += 1;
                queue.push(msg);
            } else {
                return Some(msg);
            }
        }

        Some(queue.remove(0))
    }
}

impl Scheduler for ProbabilisticDelayScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
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
            } 
            return Some(msg);
            
        }

        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }
}

impl Scheduler for QuorumBlockingScheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            return None;
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

            return Some(msg);
        }

        // If every message would complete quorum,
        // force deliver one to avoid deadlock.
        if queue.is_empty() {
            None
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

            Some(msg)
        }
    }
}