use crate::message::Message;
use crate::trace::{trace, TraceEvent};

pub struct Network {
    pub queue: Vec<Message>,
}

impl Network {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn send(&mut self, msg: Message) {
        trace(
            TraceEvent::Send,
            &format!("{} -> {}", msg.from, msg.to),
        );
        self.queue.push(msg);
    }

    pub fn deliver_next(&mut self) -> Option<Message> {
        self.queue.pop()
    }
}