# Basic Paxos in VeriProtocol

## Goal

This document describes the initial Basic Paxos implementation added to VeriProtocol.

The goal is not yet Multi-Paxos or production-grade Paxos. The current implementation focuses on the Basic Paxos happy path:

```text
Prepare
  ↓
Promise
  ↓
AcceptRequest
  ↓
Accepted
  ↓
Chosen
```

This gives VeriProtocol a second consensus protocol family beyond the original timeout-based protocol and makes the simulator more useful for comparing protocol behavior under different schedulers.

---

## Scope

### Current Scope

* Single proposer
* Four acceptors
* Single ballot
* Single proposed value (`v1`)
* Quorum size = 3
* FIFO scheduler support
* Bounded-delay-leader scheduler support

### Out of Scope

* Multi-Paxos
* Multiple competing proposers
* Leader election
* Timeout recovery
* Persistent acceptor state
* Reconfiguration
* Production deployment concerns

---

## Message Types

Basic Paxos introduces four new message types:

```rust
Prepare {
    ballot: u64
}

Promise {
    ballot: u64,
    accepted_ballot: Option<u64>,
    accepted_value: Option<String>
}

AcceptRequest {
    ballot: u64,
    value: String
}

Accepted {
    ballot: u64,
    value: String
}
```

---

## Node State

### Acceptor State

Each node maintains:

```rust
promised_ballot: u64
accepted_ballot: Option<u64>
accepted_value: Option<String>
```

### Proposer State

The proposer maintains:

```rust
promises: HashSet<u64>
accepted: HashSet<u64>
accept_request_sent: bool
```

---

## Protocol Flow

### Phase 1: Prepare

The proposer broadcasts:

```text
Prepare(ballot = 1)
```

Each acceptor checks:

```text
incoming_ballot > promised_ballot
```

If true:

```text
promised_ballot = incoming_ballot
```

and replies with:

```text
Promise(ballot = 1)
```

---

### Phase 2: Promise Quorum

The proposer collects Promise messages.

Once:

```text
|promises| >= quorum
```

the proposer broadcasts:

```text
AcceptRequest(ballot = 1, value = "v1")
```

The implementation uses:

```rust
accept_request_sent
```

to prevent duplicate AcceptRequest broadcasts.

---

### Phase 3: Accept Request

Each acceptor receives:

```text
AcceptRequest(ballot = 1, value = "v1")
```

If:

```text
ballot >= promised_ballot
```

then:

```text
accepted_ballot = 1
accepted_value = "v1"
```

and the acceptor replies:

```text
Accepted(ballot = 1, value = "v1")
```

---

### Phase 4: Accepted Quorum

The proposer collects Accepted messages.

Once:

```text
|accepted| >= quorum
```

the value is chosen:

```text
Chosen("v1")
```

and the simulator records:

```text
Decisions = 1
```

---

## VeriProtocol Execution Trace

Successful execution follows:

```text
Prepare
  ↓
Promise
  ↓
AcceptRequest
  ↓
Accepted
  ↓
Chosen
```

Example:

```text
Prepare(1)
  → Promise(1)
      → AcceptRequest(1, v1)
          → Accepted(1, v1)
              → Paxos chosen value: v1
```

---

## Experimental Results

### FIFO Scheduler

Command:

```bash
cargo run -- fifo 1 42 paxos
```

Observed:

```text
Messages Sent: 16
Messages Delivered: 16
Scheduler Steps: 16
Decisions: 1
```

Observation:

* Consensus successfully completes.
* No message loss.
* No timeout required.

---

### Bounded Delay Leader Scheduler

#### K = 1

Command:

```bash
cargo run -- bounded-delay-leader 1 42 paxos 8 1
```

Result:

```text
Messages Sent: 16
Messages Delivered: 16
Scheduler Steps: 20
Decisions: 1
```

#### K = 2

Command:

```bash
cargo run -- bounded-delay-leader 1 42 paxos 8 2
```

Result:

```text
Messages Sent: 16
Messages Delivered: 16
Scheduler Steps: 24
Decisions: 1
```

#### K = 5

Command:

```bash
cargo run -- bounded-delay-leader 1 42 paxos 8 5
```

Result:

```text
Messages Sent: 16
Messages Delivered: 16
Scheduler Steps: 36
Decisions: 1
```

---

## Observation

Increasing bounded leader delay increases scheduler work:

| K | Scheduler Steps | Decision |
| - | --------------: | -------: |
| 1 |              20 |        1 |
| 2 |              24 |        1 |
| 5 |              36 |        1 |

Observation:

```text
Higher delay budget
    ⇒ More scheduler work
    ⇒ Same consensus outcome
```

Basic Paxos remains live under these bounded-delay experiments.

---

## Regression Validation

Existing VeriProtocol protocols were re-executed after integrating Basic Paxos.

Results:

```text
Decisions: 4
Timeouts: 0
View Changes: 0
```

for:

* Simple Consensus
* Two-Phase Consensus
* Timeout Protocol

Observation:

```text
Basic Paxos integration did not break existing protocols.
```

---

## Current Limitations

This implementation represents only the Basic Paxos happy path.

Missing features include:

1. Multiple proposers
2. Higher-ballot recovery
3. Previously accepted value selection
4. Stable storage
5. Timeout handling
6. Retry logic
7. Learner role separation
8. Multi-Paxos leader optimization

---

## Future Work

### Short-Term

* Remove remaining compiler warnings
* Add Paxos-specific metrics
* Add scheduler comparison experiments
* Add Paxos result logging

### Medium-Term

* Competing proposer experiments
* Timeout-aware Paxos
* Delayed Promise scheduler
* Delayed AcceptRequest scheduler

### Long-Term

* Multi-Paxos
* Persistent acceptor state
* Leader leases
* Comparative evaluation against timeout-based protocols

---

## Why This Matters

VeriProtocol now supports multiple consensus families:

```text
Simple Consensus
Timeout Consensus
Basic Paxos
```

This transforms VeriProtocol from a protocol-specific simulator into a broader consensus experimentation framework.

The implementation also provides practical understanding of:

* Quorums
* Ballots
* Proposer logic
* Acceptor logic
* Message scheduling effects

which are directly relevant to distributed storage systems and Paxos-based systems such as Google Spanner.
