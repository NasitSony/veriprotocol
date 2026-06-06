use crate::message::Message;
use rand::RngExt;
use rand::rngs::StdRng;
use rand::SeedableRng;

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