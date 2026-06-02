use crate::message::Message;
use rand::Rng;
use rand::RngExt;

pub trait Scheduler {
    fn choose_next(&self, queue: &mut Vec<Message>) -> Option<Message>;
}

pub struct FifoScheduler;
pub struct RandomScheduler;

impl FifoScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl Scheduler for FifoScheduler {
    fn choose_next(&self, queue: &mut Vec<Message>) -> Option<Message> {
        queue.pop()
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