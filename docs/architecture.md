# VeriProtocol Architecture

VeriProtocol is a scheduler-aware consensus simulation framework. Its goal is to evaluate how different message scheduling policies affect consensus progress, decision latency, and communication behavior.

## High-Level Architecture

```text
                  +----------------+
                  |   Simulation   |
                  +----------------+
                    |      |      |
                    |      |      |
                    v      v      v
              +--------+ +----------+ +---------+
              |Protocol| | Scheduler| | Metrics |
              +--------+ +----------+ +---------+
                    |
                    v
                 +------+
                 | Node |
                 +------+
                    ^
                    |
                 +---------+
                 | Network |
                 +---------+
```

## Components

### Simulation

`Simulation` is the main experiment driver. It initializes nodes, creates the network, selects a scheduler, runs message delivery, executes protocol actions, and records metrics.

The simulation loop repeatedly asks the network for the next deliverable message. When a message is delivered to a node, the selected protocol processes it and may return new actions such as broadcasting a vote or commit message.

### Protocol

`Protocol` defines the consensus logic.

The current implementation is `SimpleConsensusProtocol`, which follows a three-phase flow:

```text
Proposal -> Vote -> Commit -> Decision
```

The protocol is responsible for:

* processing delivered messages
* updating node state
* detecting quorum formation
* generating protocol actions

This separation allows VeriProtocol to support multiple consensus protocols in the future, such as PBFT-style, HotStuff-style, or MVBA-style protocols.

### Node

`Node` stores local protocol state.

Each node tracks:

* node id
* current state
* received message counts
* completed quorums
* decision value

After the protocol abstraction, nodes mostly act as state containers, while protocol-specific transition logic lives in the protocol implementation.

### Message

`Message` represents communication between nodes.

A message includes:

* sender
* receiver
* round
* message type
* vote value
* delay count

Current message types include:

```text
Proposal
Vote
Commit
```

Current vote values include:

```text
Yes
No
```

### Network

`Network` stores the pending message queue.

It does not decide which message should be delivered next by itself. Instead, it delegates that decision to the configured scheduler.

This allows the same protocol and node logic to be tested under different delivery policies.

### Scheduler

`Scheduler` controls message delivery order.

Implemented schedulers include:

* `FifoScheduler`
* `RandomScheduler`
* `DelayNodeScheduler`
* `DelayProposalScheduler`
* `DelayVoteScheduler`
* `DelayCommitScheduler`
* `BoundedDelayScheduler`
* `ProbabilisticDelayScheduler`
* `QuorumBlockingScheduler`

Schedulers can model neutral delivery, randomized delivery, bounded delay, probabilistic delay, phase-specific delay, node-specific delay, and quorum-aware adversarial delay.

The most important research idea in VeriProtocol is that scheduling is treated as a first-class experimental dimension.

### Metrics

`Metrics` records experiment outcomes.

Current metrics include:

* messages sent
* messages delivered
* decision delivery count
* messages sent until decision
* undelivered messages at decision
* number of decisions

These metrics allow comparison across different schedulers.

## Execution Flow

A typical run follows this flow:

```text
1. Simulation initializes nodes, protocol, network, scheduler, and metrics.
2. Initial Proposal messages are broadcast.
3. Scheduler selects the next message to deliver.
4. Network delivers the message.
5. Protocol processes the message for the target node.
6. Protocol may return actions such as BroadcastVote or BroadcastCommit.
7. Simulation converts actions into new network messages.
8. Metrics are updated.
9. The run stops when all nodes decide.
```

## Current Protocol Flow

The current simple protocol uses quorum size 3 with 4 nodes.

```text
Proposal quorum -> broadcast Vote
Vote quorum     -> broadcast Commit
Commit quorum   -> decide
```

This protocol is intentionally simple. Its purpose is to provide a baseline protocol for studying scheduler behavior before adding more realistic protocols.

## Research Goal

VeriProtocol explores the question:

```text
How does message scheduling affect consensus protocol progress?
```

Early experiments show that different schedulers can significantly change decision delivery count and communication behavior.

For example:

* delaying a node increases decision latency
* delaying proposal or vote messages can reduce redundant broadcasts
* uniform bounded delay behaves similarly to FIFO
* probabilistic delay behaves like noisy network latency
* quorum-aware scheduling produces the highest decision latency observed so far

## Future Direction

Planned extensions include:

* protocol plug-ins
* PBFT-style protocol
* HotStuff-style protocol
* MVBA-style protocol
* adaptive quorum-blocking schedulers
* timeout and leader-election experiments
* automated CSV result export
* scheduler vulnerability metrics
