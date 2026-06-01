use crate::message::Message;

pub struct FifoScheduler;

impl FifoScheduler {
    pub fn new() -> Self {
        Self
    }

    pub fn choose_next(&self, queue: &mut Vec<Message>) -> Option<Message> {
        queue.pop()
    }
}