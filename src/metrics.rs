// metrics.rs

pub struct Metrics {
    pub messages_sent: usize,
    pub messages_delivered: usize,
    pub decisions: usize,
    pub decision_delivery_count: usize,
    pub messages_sent_until_decision: usize,
}


impl Metrics {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_delivered: 0,
            decisions: 0,
            decision_delivery_count: 0,
            messages_sent_until_decision: 0,
        }
    }
}

impl Metrics {
    pub fn print(&self) {
        println!("\n=== Metrics ===");
        println!("Messages Sent: {}", self.messages_sent);
        println!("Messages Delivered: {}", self.messages_delivered);
        println!("Decision Delivery Count: {}", self.decision_delivery_count);
        println!(
            "Messages Sent Until Decision: {}",
            self.messages_sent_until_decision
        );
        println!(
            "Undelivered Messages At Decision: {}",
            self.messages_sent_until_decision - self.decision_delivery_count
        );
        println!("Decisions: {}", self.decisions);
    }
}