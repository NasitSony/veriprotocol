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

FIFO:          Decisions 4, Timeouts 4
DelayLeader:  Decisions 4, Timeouts 8
TimeoutFirst: Decisions 0, Timeouts 8

### Finding: Timeout Threshold Controls Recovery Behavior

Experiments with the TimeoutProtocol show that protocol behavior depends strongly on the relationship between scheduler delay and the timeout threshold.

With a large timeout threshold (T=20), all evaluated schedulers successfully reach a decision without triggering timeout or view-change mechanisms. Delayed messages arrive before the timeout expires and continue to contribute to useful progress.

With smaller timeout thresholds, scheduler-induced delays cause replicas to timeout and advance to newer views. Delayed proposals from older views are eventually delivered, but are classified as stale and ignored. Recovery is achieved through leader rotation and proposal retransmission in the new view.

These results highlight an important distinction between message delivery and useful delivery. Eventual delivery alone does not guarantee useful progress. A message may arrive after a timeout and no longer contribute to protocol advancement.

This observation motivates scheduler-aware evaluation of timeout-based consensus protocols, where protocol behavior depends not only on whether messages arrive, but also on when they arrive relative to timeout boundaries.

## Timeout Threshold Sensitivity

Protocol behavior exhibits a threshold effect with respect to the timeout parameter.

For timeout values between 0 and 10, all runs trigger timeout and view-change recovery before reaching a decision.

For timeout values of 15 and above, decisions are reached without timeout or view change.

Despite identical safety outcomes (all runs decide), smaller timeout values incur additional recovery overhead.

This suggests that protocol behavior is highly sensitive to the relationship between message delay and timeout configuration.

## Finding: Critical Timeout Threshold for DelayLeader

For the DelayLeader scheduler, we observed a sharp transition between `T = 12` and `T = 13`.

| T | Timeouts | View Changes | Stale Ignored | Decisions |
|---:|---:|---:|---:|---:|
| 11 | 4 | 4 | 3 | 4 |
| 12 | 4 | 4 | 3 | 4 |
| 13 | 0 | 0 | 0 | 4 |
| 14 | 0 | 0 | 0 | 4 |

This suggests a critical timeout threshold of `T* = 13` for DelayLeader in the current 4-node model.

Below this threshold, leader-originated messages are delayed long enough to trigger timeout-driven view recovery. At or above this threshold, the protocol reaches decision before timeout injection occurs.

## Finding: TimeoutFirst Requires a Larger Timeout Threshold

For the TimeoutFirst scheduler, the transition occurs between `T = 15` and `T = 16`.

| T | Timeouts | View Changes | Stale Ignored | Decisions |
|---:|---:|---:|---:|---:|
| 15 | 4 | 4 | 3 | 4 |
| 16 | 0 | 0 | 0 | 4 |
| 17 | 0 | 0 | 0 | 4 |
| 20 | 0 | 0 | 0 | 4 |

This suggests a critical timeout threshold of `T* = 16`.

Compared to DelayLeader (`T* = 13`), TimeoutFirst requires a larger timeout threshold to avoid unnecessary view changes.