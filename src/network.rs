use crate::message::Message;
use crate::scheduler::{
    BoundedDelayLeaderScheduler, BoundedDelayScheduler, BoundedQuorumUsefulDelayScheduler,
    CommitDelayScheduler, CriticalMessageDelayScheduler, DeadlineAwareQuorumDelayScheduler,
    DelayLeaderScheduler, DelayScheduler, FifoScheduler,
    InterleavedProgressTargetedBudgetDelayScheduler, InterleavedTargetedBudgetDelayScheduler,
    InterleavedUniformBudgetDelayScheduler, MPAcceptRequestBudgetDelayScheduler,
    MPPrepareBudgetDelayScheduler, MPPromiseBudgetDelayScheduler, PaxosBallotOverlapScheduler,
    PaxosGapOneBacklogScheduler, PaxosGapOneScheduler, PaxosOverlapScheduler,
    PaxosProgressScheduler, PaxosRetryAdversaryScheduler, PaxosRetryScheduler,
    PhaseBalancedBudgetDelayScheduler, ProbInterleavedTargetedBudgetDelayScheduler,
    ProbInterleavedUniformBudgetDelayScheduler, ProbabilisticDelayScheduler,
    ProgressAwareQuorumDelayScheduler, ProposalDelayScheduler, QuorumBlockingScheduler,
    RandomScheduler, Scheduler, SchedulerOutcome, TargetedBudgetDelayScheduler,
    TimeoutFirstScheduler, UniformActiveBudgetDelayScheduler, UniformBudgetDelayScheduler,
    UniformCappedBudgetDelayScheduler, VoteDelayScheduler,
};

use std::collections::HashMap;
//pub scheduler: FifoScheduler,

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkModel {
    GlobalQueue,
    PerSenderRoundRobin,
}

pub struct Network {
    // Existing model.
    pub queue: Vec<Message>,

    // Challenge model: one outbound queue per sender.
    pub sender_queues: HashMap<u64, Vec<Message>>,

    pub scheduler: Box<dyn Scheduler>,
    pub model: NetworkModel,

    // Used only by PerSenderRoundRobin.
    next_sender: u64,
}

impl Network {
    pub fn new(
        scheduler_name: &str,
        seed: u64,
        max_delay: usize,
        delay_probability: f64,
        quorum_size: usize,
        model: NetworkModel,
    ) -> Self {
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

            "interleaved-progress-targeted-budget-delay" => Box::new(
                InterleavedProgressTargetedBudgetDelayScheduler::new(max_delay, 2),
            ),

            "prob-interleaved-uniform-budget-delay" => Box::new(
                ProbInterleavedUniformBudgetDelayScheduler::new(max_delay, delay_probability, seed),
            ),

            "prob-interleaved-targeted-budget-delay" => {
                Box::new(ProbInterleavedTargetedBudgetDelayScheduler::new(
                    max_delay,
                    delay_probability,
                    seed,
                ))
            }

            "deadline-aware-quorum-delay" => Box::new(DeadlineAwareQuorumDelayScheduler::new(
                max_delay,
                quorum_size,
                seed,
            )),

            "bounded-quorum-useful-delay" => {
                Box::new(BoundedQuorumUsefulDelayScheduler::new(
                    max_delay,
                    quorum_size,
                    5, // max_consecutive_delay
                    1,
                    seed,
                ))
            }

            "progress-aware-quorum-delay" => Box::new(ProgressAwareQuorumDelayScheduler::new(
                max_delay,
                quorum_size,
                2,
                seed,
            )),

            "uniform-active-budget-delay" => {
                Box::new(UniformActiveBudgetDelayScheduler::new(max_delay, 2, seed))
            }

            "phase-balanced-budget-delay" => {
                Box::new(PhaseBalancedBudgetDelayScheduler::new(max_delay, 2, seed))
            }

            "uniform-capped-budget-delay" => {
                Box::new(UniformCappedBudgetDelayScheduler::new(max_delay, 2, seed))
            }

            "mp-promise-delay" => Box::new(MPPromiseBudgetDelayScheduler::new(max_delay, 3, seed)),

            "mp-prepare-delay" => Box::new(MPPrepareBudgetDelayScheduler::new(max_delay, 3, seed)),

            "mp-accept-request-delay" => {
                Box::new(MPAcceptRequestBudgetDelayScheduler::new(max_delay, 3, seed))
            }

            _ => {
                println!("Unknown scheduler {}, using fifo", scheduler_name);
                Box::new(FifoScheduler::new())
            }
        };

        Self {
            queue: Vec::new(),
            sender_queues: HashMap::new(),
            scheduler,
            model,
            next_sender: 1,
        }
    }

    pub fn send(&mut self, msg: Message) {
        match self.model {
            NetworkModel::GlobalQueue => {
                self.queue.push(msg);
            }

            NetworkModel::PerSenderRoundRobin => {
                self.sender_queues.entry(msg.from).or_default().push(msg);
            }
        }
    }

    pub fn deliver_next(&mut self) -> SchedulerOutcome {
        match self.model {
            NetworkModel::GlobalQueue => self.scheduler.choose_next(&mut self.queue),

            NetworkModel::PerSenderRoundRobin => {
                if self.sender_queues.values().all(|q| q.is_empty()) {
                    return SchedulerOutcome::Empty;
                }

                let mut sender_ids: Vec<u64> = self.sender_queues.keys().copied().collect();
                sender_ids.sort_unstable();

                if sender_ids.is_empty() {
                    return SchedulerOutcome::Empty;
                }

                for _ in 0..sender_ids.len() {
                    let sender = self.next_sender;

                    self.next_sender += 1;

                    let max_sender = *sender_ids.last().unwrap();
                    if self.next_sender > max_sender {
                        self.next_sender = 1;
                    }

                    if let Some(queue) = self.sender_queues.get_mut(&sender) {
                        if !queue.is_empty() {
                            return self.scheduler.choose_next(queue);
                        }
                    }
                }

                // Fallback in case sender IDs are not contiguous.
                for sender in sender_ids {
                    if let Some(queue) = self.sender_queues.get_mut(&sender) {
                        if !queue.is_empty() {
                            self.next_sender = sender + 1;
                            return self.scheduler.choose_next(queue);
                        }
                    }
                }

                SchedulerOutcome::Empty
            }
        }
    }

    pub fn queue_len(&self) -> usize {
        match self.model {
            NetworkModel::GlobalQueue => self.queue.len(),

            NetworkModel::PerSenderRoundRobin => self.sender_queues.values().map(|q| q.len()).sum(),
        }
    }
}
