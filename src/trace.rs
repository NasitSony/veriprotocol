#[allow(dead_code)]
#[derive(Debug)]
pub enum TraceEvent {
    Send,
    Deliver,
    Receive,
    StateTransition,
    ProposalQuorum,
    VoteQuorum,
    CommitQuorum,
    Decision,
}

pub struct Config {
    pub print_trace: bool,
    pub print_state_changes: bool,
    pub print_quorums: bool,
    pub print_decisions: bool,
}

pub fn trace(
    config: &Config,
    event: TraceEvent,
    details: &str,
) {
    match event {
        TraceEvent::Send
        | TraceEvent::Deliver
        | TraceEvent::Receive => {
            if config.print_trace {
                println!("[{:?}] {}", event, details);
            }
        }

        TraceEvent::StateTransition => {
            if config.print_state_changes {
                println!("[{:?}] {}", event, details);
            }
        }

        TraceEvent::ProposalQuorum
        | TraceEvent::VoteQuorum
        | TraceEvent::CommitQuorum => {
            if config.print_quorums {
                println!("[{:?}] {}", event, details);
            }
        }

        TraceEvent::Decision => {
            if config.print_decisions {
                println!("[{:?}] {}", event, details);
            }
        }
    }
}