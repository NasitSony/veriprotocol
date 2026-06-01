#[derive(Debug)]
pub enum TraceEvent {
    Send,
    Deliver,
    Receive,
    StateTransition,
}


pub fn trace(event: TraceEvent, details: &str) {
    println!("[{:?}] {}", event, details);
}