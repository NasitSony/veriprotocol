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

