#[derive(Debug, Clone)]
pub enum NodeState {
    Idle,
    Proposed,
    Voted,
    Committed,
}