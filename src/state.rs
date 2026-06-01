#[derive(Debug)]
pub enum NodeState {
    Idle,
    Proposed,
    Voted,
    Committed,
}