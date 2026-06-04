use crate::message::{Message, MessageType, VoteValue};
use crate::trace::{trace, TraceEvent};
use crate::scheduler::{FifoScheduler, Scheduler, RandomScheduler};

//pub scheduler: FifoScheduler,

pub struct Network {
    pub queue: Vec<Message>,
    pub scheduler: RandomScheduler,
}


impl Network {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            scheduler: RandomScheduler::new(),
        }
    }

    pub fn send(&mut self, msg: Message) {
        trace(
            TraceEvent::Send,
            &format!("{} -> {}", msg.from, msg.to),
        );
        self.queue.push(msg);
    }

    pub fn deliver_next(&mut self) -> Option<Message> {
        self.scheduler.choose_next(&mut self.queue)
    }
}