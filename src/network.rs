use crate::message::Message;

pub struct Network {
    pub queue: Vec<Message>,
}

impl Network {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn send(&mut self, msg: Message) {
        self.queue.push(msg);
    }

    pub fn deliver_next(&mut self) -> Option<Message> {
        self.queue.pop()
    }
}