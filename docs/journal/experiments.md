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