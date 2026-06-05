// metrics.rs

pub struct Metrics {
    pub messages_sent: usize,
    pub messages_delivered: usize,
    pub decisions: usize,
    pub decision_delivery_count: usize,
}


impl Metrics {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_delivered: 0,
            decisions: 0,
            decision_delivery_count: 0,
        }
    }
}