# VeriProtocol Research Context

## Project Overview

**VeriProtocol** is a scheduler-aware experimental framework for evaluating distributed consensus protocols under different message scheduling strategies.

The project does **not** propose a new consensus algorithm.

Instead, it studies how message scheduling influences protocol progress while preserving correctness.

---

# Research Goal

Primary research question:

> How does bounded adversarial message scheduling affect consensus protocol progress while preserving eventual message delivery?

The contribution is an **experimental methodology**, not a new protocol.

---

# Research Philosophy

The scheduler is **not** the contribution.

The contribution is a protocol-agnostic evaluation framework that systematically compares consensus protocols under:

- FIFO scheduling
- Random scheduling
- Bounded adversarial scheduling

while preserving eventual delivery.

The framework measures protocol sensitivity to message ordering.

---

# Current Protocols

Implemented:

- ✅ Simple Consensus
- ✅ Two-Phase Consensus
- ✅ Timeout Consensus
- ✅ Basic Paxos
- ✅ Raft

Planned:

- HotStuff
- Committee MVBA
- (Possibly PBFT later)

---

# Paxos Status

Implemented:

- Prepare
- Promise
- AcceptRequest
- Accepted
- Retry
- Multi-proposer
- Dynamic quorum
- Dynamic node count
- Membership change placeholder

Scenarios:

- Basic consensus
- Retry
- Multi-proposer
- Scaling
- Membership placeholder

---

# Raft Status

Implemented:

- RequestVote
- VoteResponse
- Leader election
- AppendEntries heartbeat
- AppendResponse
- Leader crash / re-election
- Partition heal
- Membership change placeholder

Scenarios:

- Leader election
- Heartbeat
- Leader crash
- Partition heal
- Membership change

---

# Simulator Architecture

Simulation execution pipeline:

```
run()

↓

count_message_metrics()

↓

protocol.handle_message()

↓

apply_action()
```

Simulation has already been refactored to separate:

- message metrics
- protocol execution
- action handling

---

# Scheduler Framework

Scheduler interface:

```rust
pub trait Scheduler {
    fn choose_next(&mut self, queue: &mut Vec<Message>) -> SchedulerOutcome;
}
```

Existing schedulers:

- FIFO
- Random
- TimeoutFirst
- DelayScheduler
- ProposalDelayScheduler
- VoteDelayScheduler
- CommitDelayScheduler
- DelayLeaderScheduler
- BoundedDelayScheduler
- BoundedDelayLeaderScheduler
- ProbabilisticDelayScheduler
- QuorumBlockingScheduler

---

# Current Branch

```
experiment-v1
```

---

# Immediate Next Task

Implement:

```
CriticalMessageDelayScheduler
```

Purpose:

Delay protocol-critical messages while preserving eventual delivery.

Initially target:

Paxos

- Promise
- Accepted
- MembershipAck

Raft

- VoteResponse
- AppendResponse
- RaftConfigAck

---

# Initial Experiments

Compare:

- FIFO
- Random
- CriticalMessageDelay

Measure:

- scheduler steps
- retries
- elections
- timeouts
- leader changes
- membership completion
- messages sent
- messages delivered
- protocol completion

---

# Long-Term Experimental Roadmap

Phase 1

- Paxos ✓

Phase 2

- Raft ✓

Phase 3

- Scheduler experiments

Phase 4

- HotStuff

Phase 5

- Committee MVBA

Phase 6

- Cross-protocol evaluation

Phase 7

- Workshop paper

---

# Intended Research Contribution

The paper does **not** claim a new scheduler.

The paper introduces a scheduler-aware evaluation methodology for distributed consensus.

The framework evaluates protocol sensitivity to bounded adversarial message scheduling.

The focus is understanding how scheduling changes protocol behavior, including:

- retries
- elections
- view changes
- message complexity
- completion latency
- scheduler-induced protocol work

rather than proposing new consensus algorithms.

---

# Future Research Direction

Primary protocol comparison:

- Paxos vs Raft

Second stage:

- HotStuff vs Committee MVBA

The ultimate objective is to compare consensus protocol families under identical scheduler models and quantify scheduler sensitivity using a unified experimental framework.