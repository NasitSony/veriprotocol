# VeriProtocol

VeriProtocol is a scheduler-aware consensus experimentation framework for studying how message scheduling policies influence the progress, latency, retries, and resilience of distributed consensus protocols under deterministic simulations.

Unlike traditional consensus simulators that focus primarily on protocol correctness, VeriProtocol treats the **message scheduler as a first-class experimental component**, enabling systematic evaluation of adversarial scheduling strategies under bounded delay budgets and protocol-aware delivery policies.

---

# Why VeriProtocol?

Distributed consensus protocols are typically evaluated assuming FIFO delivery, random message ordering, or network faults.

VeriProtocol investigates a different question:

> **How does the message scheduler itself influence protocol progress?**

By making scheduling policies pluggable, the framework enables controlled experiments comparing random, protocol-aware, and adversarial schedulers while preserving deterministic execution and reproducible results.

---

# Design Principles

* Deterministic event-driven simulation
* Reproducible seeded experiments
* Pluggable consensus protocols
* Pluggable scheduler framework
* Separation between protocol logic and network scheduling
* Extensible metrics and experiment infrastructure

---

# Architecture

```text
                    +----------------------+
                    | Consensus Protocol   |
                    +----------------------+
                               |
                               v
+-------------+     +------------------+     +-------------+
| Scheduler   |<--->|    Simulator     |<--->|   Network   |
+-------------+     +------------------+     +-------------+
                               |
                               v
                      +----------------+
                      |    Metrics     |
                      +----------------+
```

---

# Repository Structure

```text
src/
├── basic_paxos.rs
├── metrics.rs
├── network.rs
├── node.rs
├── protocol.rs
├── scheduler.rs
├── simulator.rs
├── trace.rs

docs/
├── architecture.md
├── experiments.md
├── findings.md
└── project_overview.md
```

---

# Features

* Deterministic event-driven simulation
* Seeded reproducible experiments
* Pluggable protocol abstraction
* Pluggable scheduler/adversary framework
* Timeout and retry modeling
* Configurable delay budgets
* Per-message delay caps
* Comprehensive protocol metrics
* Automated experiment support

---

# Implemented Protocols

| Protocol                        | Status  |
| ------------------------------- | ------- |
| Simple Proposal → Vote → Commit | ✅       |
| Single-Proposer Paxos           | ✅       |
| Partial Timeout Paxos           | ✅       |
| Multi-Paxos                     | 🚧      |
| Raft                            | Planned |
| PBFT                            | Planned |
| HotStuff                        | Planned |

---

# Scheduler Taxonomy

## Baseline

* FIFO
* Random

## Delay-Based

* DelayNode
* DelayProposal
* DelayVote
* DelayCommit
* BoundedDelay
* ProbabilisticDelay

## Budget-Aware

* Uniform Budget Delay
* Uniform Active Budget Delay

## Protocol-Aware

* Phase-Balanced Budget Delay
* Progress-Aware Quorum Delay
* QuorumBlocking

---

# Metrics

VeriProtocol records protocol behavior including:

* Messages sent
* Messages delivered
* Scheduler steps
* Decision latency
* Timeout events
* Retry count
* Ballot progression
* Safety violations
* Critical message delays
* Chosen values

---

# Running Experiments

Basic examples:

```bash
cargo run -- fifo

cargo run -- random 10 42

cargo run -- quorum-blocking

cargo run -- progress-aware-quorum-delay
```

General format:

```text
cargo run -- <scheduler> <runs> <seed> <protocol> <timeout> <budget> <nodes> <loss_probability>
```

Example:

```bash
cargo run -- progress-aware-quorum-delay 100 42 paxos-partial-timeout 100 120 25 0.2
```

---

# Research Questions

VeriProtocol is designed to investigate questions such as:

* How do different scheduling policies affect consensus progress?
* When do bounded-delay adversaries trigger retries or timeouts?
* How do delay budget and per-message delay capacity interact?
* Which scheduling strategies are most disruptive for different consensus protocols?
* How do scheduler behaviors change from Paxos to Multi-Paxos, Raft, and Byzantine consensus?

---

# Current Research

Current work focuses on scheduler-aware evaluation of Paxos under deterministic simulations using bounded delay budgets and protocol-aware adversarial schedulers.

The framework has been used to compare:

* Active-ballot scheduling
* Phase-balanced scheduling
* Quorum-aware scheduling

Future work extends these studies to competing proposers, Multi-Paxos, Raft, and Byzantine fault-tolerant protocols.

---

# Roadmap

## Milestone 1 — Core Framework ✅

* Deterministic simulator
* Scheduler abstraction
* Metrics framework
* Simple consensus protocol

## Milestone 2 — Paxos ✅

* Single-Proposer Paxos
* Timeout and retry model
* Scheduler-aware experiments

## Milestone 3 — Multi-Paxos 🚧

* Stable leader optimization
* Competing proposers
* Value adoption
* Scheduler evaluation

## Milestone 4

* Raft
* Leader election
* Log replication

## Milestone 5

* PBFT
* HotStuff
* Byzantine schedulers
* Adaptive adversaries
* Network partition modeling

---

# Contributing

Contributions are welcome.

Areas of interest include:

* Consensus protocol implementations
* Scheduler and adversary models
* Network fault models
* Metrics and visualization
* Experiment automation
* Documentation and examples

---

# License

This project is released under the MIT License.
