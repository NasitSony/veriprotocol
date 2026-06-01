use crate::message::Message;

pub struct Node {
    pub id: u64,
}

impl Node {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

impl Node {
    pub fn receive(&self, msg: &Message) {
        println!(
            "Node {} received payload {} in round {}",
            self.id,
            msg.payload,
            msg.round
        );
    }
}