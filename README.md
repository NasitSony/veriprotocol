# VeriProtocol

VeriProtocol is a scheduler-aware consensus simulation framework for studying how message delivery policies affect consensus progress, decision latency, and communication behavior.

## Current Status

- 4-node consensus simulator
- Protocol abstraction
- Simple Proposal -> Vote -> Commit protocol
- Pluggable schedulers
- Reproducible seeded experiments
- Metrics and findings documentation

## Supported Schedulers

- FIFO
- Random
- DelayNode
- DelayProposal
- DelayVote
- DelayCommit
- BoundedDelay
- ProbabilisticDelay
- QuorumBlocking

## Supported Protocols

- Simple Consensus
- Two-Phase Consensus
- Timeout Consensus
- Basic Paxos
- Raft (scheduler-oriented)
  

## Example

```bash
cargo run -- fifo
cargo run -- random 10 42
cargo run -- quorum-block
