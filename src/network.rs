use crate::message::{Message};
use crate::scheduler::{FifoScheduler, Scheduler, RandomScheduler, DelayScheduler,CommitDelayScheduler, BoundedDelayLeaderScheduler, DelayLeaderScheduler, VoteDelayScheduler, ProposalDelayScheduler, BoundedDelayScheduler, ProbabilisticDelayScheduler, QuorumBlockingScheduler, TimeoutFirstScheduler};

//pub scheduler: FifoScheduler,

pub struct Network {
    pub queue: Vec<Message>,
    pub scheduler: Box<dyn Scheduler>,
}


impl Network {
    pub fn new(scheduler_name: &str, seed: u64) -> Self {
        let scheduler: Box<dyn Scheduler> = match scheduler_name {
            "fifo" => Box::new(FifoScheduler::new()),
            "random" => Box::new(RandomScheduler::new(seed)),
            "delay" => Box::new(DelayScheduler::new(1)),
            "delay-commit" => Box::new(CommitDelayScheduler::new()),
            "delay-vote" => Box::new(VoteDelayScheduler::new()),
            "delay-proposal" => Box::new(ProposalDelayScheduler::new()),
            "bounded-delay" => Box::new(BoundedDelayScheduler::new(3)),
            "probabilistic-delay" => Box::new(ProbabilisticDelayScheduler::new(3,seed)),
            "quorum-block" => Box::new(QuorumBlockingScheduler::new()),
            "timeout-first" => Box::new(TimeoutFirstScheduler),
            "delay-leader" => Box::new(DelayLeaderScheduler),
            "bounded-delay-leader" => Box::new(BoundedDelayLeaderScheduler::new(10)),
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

    pub fn deliver_next(&mut self) -> Option<Message> {
        self.scheduler.choose_next(&mut self.queue)
    }
}