use crate::message::Message;
use rand::RngExt;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub trait Scheduler {
    fn choose_next(&self, queue: &mut Vec<Message>) -> Option<Message>;
}

pub struct FifoScheduler;
pub struct RandomScheduler {
    rng: StdRng,
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

impl Scheduler for FifoScheduler {
    fn choose_next(&self, queue: &mut Vec<Message>) -> Option<Message> {
        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }
}

impl Scheduler for RandomScheduler {
    fn choose_next(
        &self,
        queue: &mut Vec<Message>,
    ) -> Option<Message> {
        if queue.is_empty() {
            return None;
        }

        let mut rng = rand::rng();
        let index = rng.random_range(0..queue.len());

        Some(queue.remove(index))
    }
}