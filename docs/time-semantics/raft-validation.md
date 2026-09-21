# Raft Time-Semantics Validation

## Purpose

Test whether the time-semantics effect observed in Multi-Paxos also appears in a timeout-sensitive Raft scenario.

This experiment is intended as cross-protocol mechanism evidence, not as a direct quantitative comparison between Raft and Multi-Paxos.

## Configuration

- Protocol: Raft
- Nodes: 5
- Network model: per-sender
- Scheduler: raft-heartbeat-delay
- Heartbeat interval: 5 logical ticks
- Election timeouts: deterministic staggered timeouts
  - node 1: 20
  - node 2: 22
  - node 3: 24
  - node 4: 26
  - node 5: 28
- Delay budget K: 0, 5, 10, 15, 20, 25, 30
- Scheduler targets term-1 leader-1 heartbeats sent to followers.
- The scheduler does not spend delay budget on the leader's self-heartbeat.
- Experiment implementation frozen at commit 7e99d3c.

The scheduler/network configuration is deterministic. Repeating the experiment with different seed values would reproduce the same execution, so one execution is recorded per configuration.

## Primary Outcome

A run is classified as unstable when:

    raft_election_count > 1

The initial successful leader election is included in `raft_election_count`.

Therefore, an unstable run contains at least one additional successful leader election after initial leader establishment.

Secondary observations include:

- timeout count
- final leader
- heartbeat rejections
- raw election count

## Time Models

### EventCoupled

Each scheduler opportunity advances protocol time by one logical tick.

### RoundTick

For N nodes, protocol time advances by one logical tick after N scheduler opportunities.

For N=5:

    5 scheduler opportunities = 1 logical tick

RoundTick should not be interpreted as exact wall-clock time. It prevents each individual scheduler opportunity from directly advancing protocol time.

## Observation Horizons

Two observation horizons are used.

### Scheduler horizon

Both time models receive 400 scheduler opportunities.

For N=5:

- EventCoupled: 400 scheduler steps / 400 logical ticks
- RoundTick: 400 scheduler steps / 80 logical ticks

This tests the effect of the time model under equal scheduler activity.

### Logical-time horizon

Both time models receive 400 logical ticks.

For N=5:

- EventCoupled: 400 scheduler steps / 400 logical ticks
- RoundTick: 2000 scheduler steps / 400 logical ticks

This controls for the possibility that RoundTick appears more stable only because the scheduler-horizon experiment gives it fewer logical ticks.

## Results

At K=0, both EventCoupled and RoundTick remained stable with one successful election and zero timeouts.

Under EventCoupled, every tested nonzero delay budget (K=5 through K=30) produced additional successful leader elections. Election counts ranged from 2 to 3 and timeout counts ranged from 16 to 64.

Under RoundTick, every tested K value remained at one successful election and zero timeouts.

The RoundTick result remained unchanged under the 400-logical-tick horizon, despite requiring 2000 scheduler opportunities.

EventCoupled produced identical results under scheduler and logical-time horizons because one scheduler opportunity corresponds to one logical tick in that model.

## Interpretation

The result is consistent with the time-semantics mechanism observed in the Multi-Paxos experiments.

Under EventCoupled, scheduler activity used to delay or reorder communication also advances timeout-related protocol time. Thus an adversary defined in terms of message scheduling can indirectly influence timeout progression.

Under RoundTick, individual scheduler opportunities do not each advance protocol time. In this experiment, the same bounded heartbeat-delay scheduler did not cause additional successful elections, including when RoundTick was observed for the same 400 logical ticks.

The experiment does not establish that all nonzero K values cause instability under EventCoupled; it establishes the result only for the tested grid K={5,10,15,20,25,30}.

The experiment also does not claim that RoundTick is an exact model of wall-clock execution. It isolates one methodological difference: whether scheduler-event progression directly determines timeout progression.

## Data

See:

    docs/time-semantics/raft-validation.csv
