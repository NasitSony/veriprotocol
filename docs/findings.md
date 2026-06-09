Finding 1:
Proposal delay reduced redundant votes.

Finding 2:
Vote delay reduced redundant commits.

Finding 3:
Uniform delay had little impact.

Finding 4:
Probabilistic delay behaves like network noise.

Finding 5:
Quorum-aware scheduling maximizes decision latency.

# VeriProtocol Findings

## Finding 1: Scheduler choice changes decision latency

Across repeated runs, different schedulers produced different decision-delivery counts:

| Scheduler | Decision Delivery Count |
|---|---:|
| DelayProposal | 35 |
| DelayVote | 39 |
| FIFO | 44 |
| DelayCommit | 44 |
| ProbabilisticDelay | 45 |
| DelayNode | 46 |
| QuorumBlocking | 47 |

## Finding 2: Protocol-aware scheduling is the strongest adversary so far

The QuorumBlocking scheduler produced the highest decision-delivery count.

This suggests that adversarial schedulers become more effective when they understand quorum formation rather than applying uniform or random delay.

## Finding 3: Not all delays are harmful

Some delays reduce redundant message delivery before decision. DelayProposal and DelayVote resulted in lower decision-delivery counts than FIFO.

## Finding 4: Reproducibility is now supported

VeriProtocol supports seeded randomized experiments, for example:

```bash
cargo run -- random 100 42
```

## Finding 5: Reproducibility is now supported
QuorumBlockingScheduler consistently produced the worst observed
decision latency across all runs. Unlike RandomScheduler, which
exhibited significant variability, QuorumBlockingScheduler
reliably delayed quorum formation and resulted in a stable
decision delivery count of 47.



Experiment: Timeout-first scheduling

Observation:
When timeout messages are delivered before proposal messages,
all nodes advance to a higher view before processing the proposal.

Result:
- Decisions: 0
- Timeouts Triggered: 4
- View Changes: 4
- Stale Messages Ignored: 4

Interpretation:
A scheduler can prevent progress without permanently dropping messages
by causing timeout events to occur before proposal delivery.


A scheduler does not need to drop messages forever. Delaying leader-originated messages until timeout events are processed can force a view change and make the original proposal stale.

In a leader-driven protocol, progress depends on timely delivery of the leader’s proposal. A fair scheduler may still delay the leader long enough for replicas to timeout and advance views. The proposal is eventually delivered, but it is stale by then, so liveness can fail without permanent message loss.

## Insight: Fair Delay Can Still Break Useful Progress

In a leader-driven protocol, progress depends not only on eventual message delivery, but on timely delivery of the leader’s proposal.

A fair scheduler may eventually deliver every message, while still delaying the leader’s proposal long enough for replicas to timeout and advance to a newer view. When the delayed proposal finally arrives, it is stale and ignored.

This creates an important distinction:

- The message was eventually delivered.
- The message was no longer useful when delivered.

Therefore, liveness degradation can occur without permanent message loss.

| Scheduler    | Decisions | Timeouts | View Changes | Stale Ignored |
| ------------ | --------- | -------- | ------------ | ------------- |
| FIFO         | 4         | 4        | 4            | 0             |
| TimeoutFirst | 0         | 4        | 4            | 4             |
| DelayLeader  | 0         | 4        | 4            | 4             |
