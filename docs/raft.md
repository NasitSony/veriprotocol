# Raft Implementation

## Overview

Implemented protocol mechanisms

- Leader election
- RequestVote
- VoteResponse
- Heartbeats (AppendEntries)
- AppendResponse
- Leader crash and re-election
- Partition heal
- Membership change placeholder

---

## Implemented Scenarios

### 1. Leader Election

Description

Metrics

Expected outcome

---

### 2. Heartbeat

Description

Metrics

Expected outcome

---

### 3. Leader Crash

Description

Metrics

Expected outcome

---

### 4. Partition Heal

Description

Metrics

Expected outcome

---

### 5. Membership Change

Description

Metrics

Expected outcome

---

## Supported Metrics

- Leader elections
- Leader ID
- RequestVote
- VoteResponse
- AppendEntries
- AppendResponse
- Heartbeat successes
- Heartbeat rejections
- Configuration changes
- Configuration acknowledgements

---

## Current Limitations

This implementation intentionally omits:

- Log replication
- Log matching
- Commit index
- State machine application
- Snapshot installation
- Log compaction
- Persistent storage

The implementation focuses on scheduler-sensitive consensus behavior rather than replicated state machine semantics.

---

## Research Motivation

This implementation is intended for scheduler-aware evaluation.

The goal is to study how FIFO, random, and bounded adversarial message scheduling influence:

- leader election
- heartbeat stability
- re-election
- partition recovery
- membership reconfiguration