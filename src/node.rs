use crate::message::Message;

pub struct Node {
    pub id: u64,
    pub messages_received: u64,
}
impl Node {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            messages_received: 0,
        }
    }
}


impl Node {
    pub fn receive(&mut self, msg: &Message) {
        self.messages_received += 1;
        println!(
            "Node {} received payload {} in round {}",
            self.id,
            msg.payload,
            msg.round
        );
        println!(
            "Node {} received {} messages",
            self.id,
            self.messages_received
        );
    }
}