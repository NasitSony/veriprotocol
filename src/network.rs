use crate::message::Message;
use crate::scheduler::{
    BoundedDelayLeaderScheduler, BoundedDelayScheduler, CommitDelayScheduler,
    CriticalMessageDelayScheduler, DelayLeaderScheduler, DelayScheduler, FifoScheduler,
    PaxosBallotOverlapScheduler, PaxosGapOneBacklogScheduler, PaxosGapOneScheduler,
    PaxosOverlapScheduler, PaxosProgressScheduler, PaxosRetryAdversaryScheduler,
    PaxosRetryScheduler, ProbabilisticDelayScheduler, ProposalDelayScheduler,
    QuorumBlockingScheduler, RandomScheduler, Scheduler, SchedulerOutcome,
    TargetedBudgetDelayScheduler, TimeoutFirstScheduler, UniformBudgetDelayScheduler,
    VoteDelayScheduler, InterleavedUniformBudgetDelayScheduler, InterleavedTargetedBudgetDelayScheduler,
    InterleavedProgressTargetedBudgetDelayScheduler, ProbInterleavedUniformBudgetDelayScheduler, ProbInterleavedTargetedBudgetDelayScheduler,

};

use std::collections::HashMap;
//pub scheduler: FifoScheduler,

pub struct Network {
    pub queue: Vec<Message>,
    pub scheduler: Box<dyn Scheduler>,
}

impl Network {
    pub fn new(scheduler_name: &str, seed: u64, max_delay: usize, delay_probability: f64,) -> Self {
        let scheduler: Box<dyn Scheduler> = match scheduler_name {
            "fifo" => Box::new(FifoScheduler::new()),
            "random" => Box::new(RandomScheduler::new(seed)),
            "delay" => Box::new(DelayScheduler::new(1)),
            "delay-commit" => Box::new(CommitDelayScheduler::new()),
            "delay-vote" => Box::new(VoteDelayScheduler::new()),
            "delay-proposal" => Box::new(ProposalDelayScheduler::new()),
            "bounded-delay" => Box::new(BoundedDelayScheduler::new(3)),
            "probabilistic-delay" => Box::new(ProbabilisticDelayScheduler::new(3, seed)),
            "quorum-block" => Box::new(QuorumBlockingScheduler::new()),
            "timeout-first" => Box::new(TimeoutFirstScheduler),
            "delay-leader" => Box::new(DelayLeaderScheduler::new()),
            "bounded-delay-leader" => Box::new(BoundedDelayLeaderScheduler::new(max_delay)),
            "critical-delay" => Box::new(CriticalMessageDelayScheduler {
                max_delay: max_delay as usize,
                delayed: Vec::new(),
                delays_used: 0,
            }),
            "paxos-retry-adversary" => Box::new(PaxosRetryAdversaryScheduler {
                delayed_accepts: Vec::new(),
                max_delay: max_delay as usize,
                delays_used: 0,
            }),

            "paxos-retry-scheduler" => Box::new(PaxosRetryScheduler {
                held_accepts: Vec::new(),
                max_delay: max_delay as usize,
                delays_used: 0,
            }),

            "paxos-overlap" => Box::new(PaxosOverlapScheduler {
                quorum_size: 3, // temporary hardcode for 5 nodes
                delayed_accepted: Vec::new(),
                held_lower_ballot: Vec::new(),
                accepted_seen: HashMap::new(),
                max_delay: max_delay as usize,
                delays_used: 0,
            }),

            "paxos-progress" => Box::new(PaxosProgressScheduler {
                quorum_size: 3, // temporary for 5 nodes
                held_lower_ballot: Vec::new(),
                promise_seen: HashMap::new(),
                max_delay: max_delay as usize,
                delays_used: 0,
            }),

            "ballot-overlap" => Box::new(PaxosBallotOverlapScheduler {
                max_delay: max_delay as usize,
                delays_used: 0,
                held: Vec::new(),
                promise_seen: HashMap::new(),
            }),

            "gap1" => Box::new(PaxosGapOneScheduler {
                max_delay: max_delay as usize,
                delays_used: 0,
                held_by_ballot: HashMap::new(),
                prepare_seen: HashMap::new(),
                quorum_size: 3, // temporary for 5 nodes

                max_held_backlog: 0,
                held_inserts: 0,
                held_releases: 0,
            }),

            "gap1-backlog" => Box::new(PaxosGapOneBacklogScheduler {
                max_delay: max_delay as usize,
                delays_used: 0,
                held_by_ballot: HashMap::new(),
                prepare_seen: HashMap::new(),
                quorum_size: 3,
                release_after_held: 4,
            }),

            "uniform-budget-delay" => Box::new(UniformBudgetDelayScheduler::new(max_delay)),

            "targeted-budget-delay" => Box::new(TargetedBudgetDelayScheduler::new(max_delay)),


            "interleaved-uniform-budget-delay" => {
                Box::new(InterleavedUniformBudgetDelayScheduler::new(max_delay, 2))
            }

            "interleaved-targeted-budget-delay" => {
                Box::new(InterleavedTargetedBudgetDelayScheduler::new(max_delay, 2))
            }

            "interleaved-progress-targeted-budget-delay" => {
                Box::new(InterleavedProgressTargetedBudgetDelayScheduler::new(max_delay, 2))
            }

            "prob-interleaved-uniform-budget-delay" => {
                Box::new(ProbInterleavedUniformBudgetDelayScheduler::new(max_delay, delay_probability))
            }

            "prob-interleaved-targeted-budget-delay" => {
                Box::new(ProbInterleavedTargetedBudgetDelayScheduler::new(max_delay, delay_probability))
            }
            _ => {
                println!("Unknown scheduler {}, using fifo", scheduler_name);
                Box::new(FifoScheduler::new())
            }
        };

        Self {
            queue: Vec::new(),
            scheduler,
        }
    }

    pub fn send(&mut self, msg: Message) {
        /*trace(
            &self.config,
            TraceEvent::Decision,
            &format!("{} -> {}", msg.from, msg.to),
        );
        println!(
            "Message sent: {} -> {}, type: {:?}, value: {:?}",
            msg.from, msg.to, msg.msg_type, msg.value
        );*/
        self.queue.push(msg);
    }

    pub fn deliver_next(&mut self) -> SchedulerOutcome {
        self.scheduler.choose_next(&mut self.queue)
    }
}
