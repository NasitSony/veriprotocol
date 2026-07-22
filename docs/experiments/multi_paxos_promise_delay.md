# Multi-Paxos Promise Delay Experiment

## Research question

How does bounded delay of recovery-phase Multi-Paxos Promise messages affect
leader recovery latency?

## Configuration

- Protocol: Stable Multi-Paxos
- Nodes: 5
- Failed leader: node 1
- Replacement leader: node 2
- Leader failure step: 40
- Observation window: 200 scheduler steps
- Random seed: 42
- Delay target: `MPPromise` messages with ballot >= 2
- Delay budget: 20
- Maximum consecutive delay per message: 3

## Results

| Metric | FIFO baseline | Promise delay |
|---|---:|---:|
| Recovery completion step | 97 | 109 |
| Recovery slowdown | — | 12 steps |
| View changes | 1 | 1 |
| Maximum ballot | 2 | 2 |
| Chosen slots | 3 | 3 |
| Safety violation | false | false |
| Messages sent | 184 | 164 |
| Messages delivered | 181 | 163 |
| Multi-Paxos heartbeats | 117 | 99 |

## Observation

Bounded delay of recovery-phase Promise messages postponed the first delivered
heartbeat from the replacement leader from scheduler step 97 to step 109. This
represents a 12-step, approximately 12.4% increase in measured recovery latency
for this configuration.

The delay did not cause an additional election, increase the maximum ballot, lose
any chosen slot, or violate safety.

The delayed run delivered fewer heartbeats because both experiments stopped at
step 200, leaving fewer post-recovery scheduler steps after the later recovery.

## Limitation

This result uses one seed and one delay budget. Multiple seeds and budgets are
required before making a broader claim about Promise-delay sensitivity.