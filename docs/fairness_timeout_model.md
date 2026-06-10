# Fairness and Timeout Model

## Motivation

VeriProtocol studies how message scheduling affects consensus progress. To make liveness experiments meaningful, the scheduler model must distinguish between unfair message loss and fair but harmful delay.

A scheduler that delays messages forever can trivially prevent termination. VeriProtocol therefore separates unfair schedulers from fair schedulers.

## Eventual Fairness

A scheduler is eventually fair if every message that enters the network queue is eventually delivered.

This model is useful for asynchronous protocols, where late messages may still contribute to progress.

## K-Bounded Fairness

A scheduler is K-bounded fair if every message is delivered within at most K scheduler opportunities.

This model is useful for timeout-based or partially synchronous protocols.

## Timeout Interaction

Let:

- `K` = scheduler delay bound
- `T` = protocol timeout threshold

If:

```text
K <= T

then delayed messages still arrive before timeout. ```

If:

K > T

then messages are eventually delivered, but they may arrive too late for the current view.

This creates the distinction between:

- slow but useful delivery
- fair but harmful delivery
- timeout-triggering delay

Research Direction

The next stage of VeriProtocol is to study whether a fair scheduler can cause repeated timeout or view-change behavior without permanently dropping messages.

# Finding: Timeout-Driven View Progression and Recovery

## Background

The initial TimeoutProtocol model used a leader-based proposal mechanism. A timeout event caused nodes to advance their local view, but no recovery action was performed after the timeout.

## Extension

The protocol was extended with:

* View progression on timeout.
* Deterministic leader rotation based on view number.
* Recovery proposal generation by the new leader.
* Detection and rejection of stale proposals from earlier views.
* Metrics for timeout events, view changes, and stale messages.

Leader selection is defined as:

leader(view) = (view mod n) + 1

For four nodes:

* View 0 → Leader 1
* View 1 → Leader 2
* View 2 → Leader 3
* View 3 → Leader 4

## Observation

When a timeout occurs:

1. Nodes advance to a higher view.
2. A new leader is selected.
3. The new leader broadcasts a proposal for the new view.
4. Proposals from older views are ignored as stale.

This converts timeout handling from a pure failure event into a recovery mechanism.

## Experimental Results

### Before Recovery Proposal

Timeout-first and DelayLeader schedulers caused:

* Decisions = 0
* View Changes = 4
* Stale Messages Ignored > 0

Nodes advanced views, but no new proposal was generated.

### After Recovery Proposal

The new leader broadcasts a proposal after timeout.

Observed result:

* Decisions = 4
* View Changes = 4
* Protocol recovers and terminates.

## Interpretation

Eventual message delivery alone is insufficient for progress when timeout events advance views and invalidate older proposals.

However, view progression combined with leader rotation and recovery proposals restores liveness.

The current model demonstrates the interaction between:

* Scheduler behavior
* Timeout events
* View progression
* Leader rotation
* Protocol recovery


## Timeout Threshold Model

The current timeout model injects timeout messages immediately at simulation startup. This is useful for testing view progression, but it does not yet model the relationship between scheduler delay and timeout thresholds.

We define a timeout threshold `T` as:

```text
T = number of message delivery opportunities allowed before timeout becomes eligible
```

This means timeout is not injected immediately. Instead, the simulator first allows up to `T` message deliveries. After that point, timeout messages may be injected for nodes that have not yet decided.

### Relation to Scheduler Fairness

Let `K` denote a scheduler delay bound.

```text
K = maximum number of scheduler opportunities a message can be delayed
T = timeout threshold
```

Expected relationship:

```text
K <= T  => leader proposal should arrive before timeout
K > T   => proposal may eventually arrive, but too late for the current view
```

Thus, eventual fairness alone is not sufficient to guarantee progress in timeout-based protocols. A scheduler may eventually deliver all messages but still delay critical leader messages long enough for replicas to advance views.

### Experimental Goal

The goal is to evaluate how different schedulers affect:

* number of decisions
* number of timeouts
* number of view changes
* stale messages ignored
* messages delivered until decision
* undelivered messages at decision

### Planned Experiments

| Scheduler       | Timeout Threshold T | Expected Behavior                              |
| --------------- | ------------------: | ---------------------------------------------- |
| FIFO            |             small T | likely decides before or after one view change |
| TimeoutFirst    |             small T | timeout before proposal; view change likely    |
| DelayLeader     |             small T | leader proposal may become stale               |
| DelayLeader     |             large T | leader proposal may arrive before timeout      |
| BoundedDelay(K) |              K <= T | progress without unnecessary view change       |
| BoundedDelay(K) |               K > T | proposal may become stale and cause view churn |

### Research Question

How large must the timeout threshold `T` be, relative to the scheduler delay bound `K`, to preserve progress in a leader-based consensus protocol?


## Model Clarification: Delivery-Based vs Scheduler-Step Timeout

The current implementation uses a delivery-based timeout threshold:

```text
T_delivery = number of delivered messages before timeout becomes eligible

This means delayed/requeued messages do not consume timeout budget unless they are actually delivered.

For K-bounded scheduler experiments, a stronger model is:

T_step = number of scheduler opportunities before timeout becomes eligible

Under T_step, delaying a message still consumes scheduler budget, even if no message is delivered.

This distinction matters because bounded leader delay may not trigger timeout under T_delivery, even when many scheduler opportunities are spent delaying leader messages.```