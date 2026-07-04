# VeriProtocol

VeriProtocol is a scheduler-aware consensus experimentation framework for studying how message scheduling policies influence the progress, latency, retries, and resilience of distributed consensus protocols under deterministic simulations.

The framework provides a reproducible environment for implementing consensus protocols, designing adversarial message schedulers, and evaluating protocol behavior under different delay models.

## Features

* Deterministic event-driven simulation
* Pluggable consensus protocol architecture
* Pluggable scheduler/adversary framework
* Seeded, reproducible experiments
* Configurable timeout and retry models
* Comprehensive execution metrics
* Experiment automation for large parameter sweeps

## Implemented Protocols

* Simple Proposal → Vote → Commit protocol
* Single-Proposer Paxos
* Partial-timeout Paxos model

## Scheduler Policies

### Baseline

* FIFO
* Random

### Delay-Based

* DelayNode
* DelayProposal
* DelayVote
* DelayCommit
* BoundedDelay
* ProbabilisticDelay

### Adversarial

* QuorumBlocking
* Uniform Budget Delay
* Uniform Active Budget Delay
* Phase-Balanced Budget Delay
* Progress-Aware Quorum Delay

## Metrics

VeriProtocol records protocol behavior including:

* Scheduler steps
* Messages sent and delivered
* Timeout events
* Retry count
* Ballot progression
* Decision latency
* Safety violations
* Critical message delays

## Running Experiments

```bash
cargo run -- fifo

cargo run -- random 10 42

cargo run -- quorum-blocking

cargo run -- progress-aware-quorum-delay
```

## Current Research

Current experiments investigate how adversarial message schedulers affect Paxos progress under bounded delay budgets and per-message delay limits. The framework has been used to compare active-ballot, phase-balanced, and quorum-aware scheduling strategies under deterministic simulations.

## Roadmap

* Multi-Paxos
* Raft
* PBFT
* HotStuff
* Byzantine fault injection
* Adaptive adversarial schedulers
* Network partition and recovery models
* Large-scale benchmarking
