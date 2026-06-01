use crate::message::Message;

pub trait Scheduler {
    fn choose_next(&self, queue: &mut Vec<Message>) -> Option<Message>;
}

pub struct FifoScheduler;

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