# VeriProtocol Experiments

## Experiment 1: All YES votes

Nodes: 4
Quorum: 3

### FIFO

Messages Sent: 48
Decision Delivery Count: 44
Overhead: 8

### Random Run #1

Messages Sent: 48
Decision Delivery Count: 43
Overhead: 7

### Random Run #2

Messages Sent: 48
Decision Delivery Count: 42
Overhead: 6

## Notes

Ideal minimum deliveries:
36



Ideal: 36

FIFO:
Decision Delivery Count = 44
Overhead = +8

Random:
Decision Delivery Count = 45
Overhead = +9


## Experiment 2: Random Scheduler, 10 Runs

Nodes: 4  
Quorum: 3  
Ideal decision deliveries: 36  

Decision delivery counts:

43, 44, 44, 45, 42, 44, 47, 41, 46, 45

Min: 41  
Max: 47  
Average: 44.1  
Average overhead: +8.1  

Observation:
Random scheduling changes decision latency. Some schedules reach consensus faster than FIFO, while others are slower.

Random Scheduler, 10 runs
Min: 41
Max: 46
Average: 43.70
Ideal: 36
Average overhead: +7.70

Random Scheduler, 10 runs, seed=42
Min: 43
Max: 47
Average: 44.90
Ideal: 36
Average overhead: +8.90


## Experiment 3: Random Scheduler (100 Runs)

Nodes: 4
Quorum: 3

Ideal decision deliveries: 36

Min: 37
Max: 47
Average: 44.05

Observation:
Random scheduling introduces delivery overhead.
Best observed execution was within one delivery
of the theoretical minimum.


## Experiment 4: Delay Scheduler

Configuration

- Nodes: 4
- Quorum: 3
- Scheduler: DelayScheduler
- Policy: Delay messages to a selected node by moving them to the back of the queue

Results (10 runs)

Decision Delivery Count:
46, 46, 46, 46, 46, 46, 46, 46, 46, 46

Summary

- Min: 46
- Max: 46
- Average: 46.00

Additional Metrics

- Messages Sent Until Decision: 48
- Undelivered Messages At Decision: 2

Observation

Delaying messages to a specific node increased decision latency compared to FIFO and Random schedulers. The current delay policy produced deterministic behavior and identical results across runs regardless of the delayed node.

## Experiment Infrastructure Milestone

- FIFO Scheduler
- Random Scheduler
- Delay Scheduler
- Seeded Randomness
- Multi-run Experiments
- Reproducible Results

Example:

cargo run -- random 100 42

## Experiment: VoteDelayScheduler

Configuration

* Nodes: 4
* Quorum: 3
* Scheduler: VoteDelayScheduler
* Runs: 10

Results

* Messages Sent: 44
* Decision Delivery Count: 39
* Undelivered Messages At Decision: 5
* Decisions: 4

Observation

VoteDelayScheduler reduced the number of messages required to reach consensus compared to FIFO and DelayNode schedulers.

Unlike other schedulers, consensus was reached before all nodes broadcast Commit messages. Only three Commit broadcasters were sufficient for all nodes to collect a Commit quorum and decide.

This reduced total messages sent from 48 to 44 and reduced decision delivery count from 44 to 39.

Takeaway

Message reordering does not always increase consensus latency. Certain delivery schedules can reduce redundant communication and allow consensus to complete with fewer messages.

## Experiment: DelayProposalScheduler

Configuration

* Nodes: 4
* Quorum: 3
* Scheduler: DelayProposalScheduler
* Runs: 10

Results

* Messages Sent: 44
* Decision Delivery Count: 35
* Undelivered Messages At Decision: 9
* Decisions: 4

Observation

Delaying Proposal messages produced the lowest decision delivery count observed so far.

Only three nodes broadcast Vote messages before consensus progressed to the Commit phase. Despite fewer Vote broadcasts, all nodes successfully reached a Commit quorum and decided.

This reduced decision delivery count from 44 (FIFO) to 35.

Takeaway

Message scheduling can significantly alter communication patterns. Delaying Proposal messages reduced redundant Vote broadcasts and allowed consensus to complete with fewer delivered messages than any previously evaluated scheduler.

DelayProposal reduced decision delivery count because delayed Proposal messages postponed some Vote broadcasts. Since quorum is 3, three nodes were sufficient to drive the Vote/Commit progression and allow all nodes to decide. The scheduler therefore reduced redundant phase participation rather than improving the protocol itself.

## Experiment: BoundedDelayScheduler

Configuration

- Nodes: 4
- Quorum: 3
- max_delay: 3
- Scheduler policy: every message may be postponed up to 3 times before delivery

Results

- Min Decision Delivery Count: 44
- Max Decision Delivery Count: 44
- Average Decision Delivery Count: 44.00

Observation

Uniform bounded delay behaved similarly to FIFO in this protocol. Since all message types were delayed equally, the scheduler did not selectively affect quorum formation or phase progression.

## Experiment: ProbabilisticDelayScheduler

Configuration

- Nodes: 4
- Quorum: 3
- Scheduler: ProbabilisticDelayScheduler
- max_delay: 3
- Delay rule: each message may be delayed probabilistically until max_delay

Results

- Runs: 5
- Min Decision Delivery Count: 44
- Max Decision Delivery Count: 46
- Average Decision Delivery Count: 45.00

Observation

Probabilistic delay produced variable decision latency while preserving liveness. Unlike uniform bounded delay, probabilistic delay introduced non-deterministic delivery order and slightly increased average decision latency compared to FIFO.

scheduler,runs,min,max,avg,observation
fifo,1,44,44,44.00,baseline
random,10,42,46,44.60,seeded random delivery
delay-node,10,46,46,46.00,delaying one node increases latency
delay-commit,10,44,44,44.00,commit delay behaves like FIFO
delay-vote,10,39,39,39.00,reduces redundant commit broadcasts
delay-proposal,?,35,35,35.00,reduces redundant vote delivery
bounded-delay,10,44,44,44.00,uniform delay behaves like FIFO
probabilistic-delay,5,44,46,45.00,noisy delay slightly worsens latency


## Experiment: QuorumBlockingScheduler

Configuration

- Nodes: 4
- Quorum: 3
- Scheduler: QuorumBlockingScheduler
- Policy: delay the 3rd matching message for each `(receiver, message_type, value)`

Result

- Messages Sent: 48
- Decision Delivery Count: 47
- Undelivered Messages At Decision: 1
- Decisions: 4

Observation

QuorumBlockingScheduler produced the highest decision delivery count observed so far. By delaying messages that would complete a quorum, the scheduler increased consensus latency beyond FIFO, Random, DelayNode, DelayCommit, DelayVote, and DelayProposal schedulers.

Takeaway

Targeting quorum-completing messages is more harmful than delaying a node or delaying a protocol phase uniformly.

## Experiment: TwoPhaseProtocol

TwoPhaseProtocol uses:

Proposal -> Vote -> Decision

Compared to SimpleConsensusProtocol:

- SimpleConsensusProtocol sends up to 48 messages.
- TwoPhaseProtocol sends up to 32 messages.

Results:

| Protocol | Scheduler | Decision Delivery Count |
|---|---:|---:|
| SimpleConsensusProtocol | FIFO | 44 |
| TwoPhaseProtocol | FIFO | 28 |
| TwoPhaseProtocol | Random | 30 |
| TwoPhaseProtocol | QuorumBlocking | 31 |

Observation:
TwoPhaseProtocol reduces total communication because it removes the Commit phase. QuorumBlockingScheduler remains more harmful than RandomScheduler, suggesting that quorum-aware scheduling continues to increase decision latency across protocols.


## Timeout Protocol Observation

Added view tracking and stale message filtering.

When a node advances to a higher view, messages from older views are ignored.

Motivation:
A delayed message may eventually arrive, but it may no longer be useful after timeout and view change.

This introduces a distinction between:

- eventually delivered messages
- useful messages

A fair scheduler may therefore satisfy eventual delivery while still causing repeated timeout behavior.

## Experiment: K-Bounded Leader Delay Timeout Sweep

CLI format:

```bash
cargo run -- <scheduler> <runs> <seed> <protocol> <T_step> <K>

cargo run -- bounded-delay-leader 1 42 timeout 6 2
cargo run -- bounded-delay-leader 1 42 timeout 8 2
cargo run -- bounded-delay-leader 1 42 timeout 10 2
cargo run -- bounded-delay-leader 1 42 timeout 12 2
cargo run -- bounded-delay-leader 1 42 timeout 16 2
cargo run -- bounded-delay-leader 1 42 timeout 20 2

cargo run -- bounded-delay-leader 1 42 timeout 8 5
cargo run -- bounded-delay-leader 1 42 timeout 12 5
cargo run -- bounded-delay-leader 1 42 timeout 16 5
cargo run -- bounded-delay-leader 1 42 timeout 20 5
cargo run -- bounded-delay-leader 1 42 timeout 24 5
cargo run -- bounded-delay-leader 1 42 timeout 32 5 
```
