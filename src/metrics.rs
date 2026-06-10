// metrics.rs

pub struct Metrics {
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub decisions: u64,
    pub messages_delivered_until_decision: u64,
    pub messages_sent_until_decision: u64,
    pub timeouts_triggered: u64,
    pub view_changes: u64,
    pub stale_messages_ignored: u64,
    pub scheduler_steps: u64,
}


impl Metrics {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_delivered: 0,
            decisions: 0,
            messages_delivered_until_decision: 0,
            messages_sent_until_decision: 0,
            timeouts_triggered: 0,
            view_changes: 0,
            stale_messages_ignored: 0,
            scheduler_steps: 0,
        }
    }
}

impl Metrics {
    pub fn print(&self) {
        println!("\n=== Metrics ===");
        println!("Messages Sent: {}", self.messages_sent);
        println!("Messages Delivered: {}", self.messages_delivered);
        println!("Scheduler Steps: {}", self.scheduler_steps);
        println!("Stale Messages Ignored: {}", self.stale_messages_ignored);
        println!(
            "Messages Sent Until Decision: {}",
            self.messages_sent_until_decision
        );
        println!(
            "Undelivered Messages At Decision: {}",
            self.messages_sent_until_decision - self.messages_delivered_until_decision
        );
        println!("Timeout triggered: {}", self.timeouts_triggered);
        println!("View changes: {}", self.view_changes);
        println!("Decisions: {}", self.decisions);
    }
}