# Working Title

## 1. Problem and Motivation

Adversarial schedulers are useful for evaluating the liveness behavior of distributed consensus protocols by systematically delaying, reordering, or prioritizing protocol messages. Such simulations, however, require not only a model of message delivery but also a model of time. This distinction becomes important for timeout-driven protocols: if protocol time advances with scheduler activity, the number of simulated message-delivery events can inadvertently determine when failure detectors expire.

We encountered this issue while evaluating Multi-Paxos under bounded adversarial message scheduling. Under an event-coupled time model, targeted delay of AcceptRequest traffic appeared to induce substantial post-recovery leadership instability. The effect persisted after replacing a global network queue with per-sender scheduling, suggesting that global serialization was not its primary cause.

However, when timeout progression was decoupled from individual scheduler events, the apparent instability disappeared. This led us to ask:

## 2. Research Question

To what extent can the time semantics of an adversarial consensus simulator alter conclusions about protocol liveness?

## 3. Simulator and Time Models

VeriProtocol is a deterministic consensus simulation framework in which protocol messages are placed into a simulated network and delivered according to a configurable scheduler. The scheduler can perturb execution by delaying or selecting protocol messages, while seeded executions provide reproducible schedules for controlled comparisons.

This study considers stable Multi-Paxos with leader failure and recovery. Followers maintain heartbeat-based failure detectors and initiate a new election when their heartbeat age reaches an election-timeout threshold. During the investigation, we identified two possible interpretations of simulated time.

### EventCoupled. Each scheduler step advances protocol time. Consequently, message delivery, adversarial delay decisions, and other scheduler activity also advance heartbeat and timeout state. Under this model, increased scheduler activity can cause failure-detector time to advance even when the activity does not represent an equivalent amount of elapsed real time.

### RoundTick. Protocol time is decoupled from individual scheduler events. For a configuration with N nodes, one logical tick occurs after N scheduler opportunities. Failure-detector and heartbeat timers advance only on these logical ticks. RoundTick is not intended as an exact model of physical wall-clock time; rather, it provides a controlled abstraction in which timeout progression is no longer directly proportional to individual scheduler-event count.

This distinction allows us to hold the protocol, scheduler, network model, workload, and random seed fixed while changing the semantics by which scheduler progress is translated into protocol time.

## 4. Experimental Setup

We evaluate a Multi-Paxos recovery scenario in which the initial leader fails and the remaining replicas must elect a replacement while preserving previously accepted values. The adversarial scheduler targets Multi-Paxos AcceptRequest traffic using a bounded delay budget K. Safety is checked throughout each execution, and recovery is considered stable when a heartbeat from the current highest-ballot post-failure leader is successfully delivered.

We initially evaluated a global network queue, then introduced a per-sender network model to test whether observed instability resulted from serialization of otherwise independent communication paths. At N=5,K=11, instability decreased only modestly from 85% under the global queue to 75% under per-sender scheduling, indicating that global serialization alone did not explain the effect.

We subsequently compared EventCoupled and RoundTick using the per-sender network model. Unless otherwise noted, each configuration was evaluated across 20 deterministic seeds. We define leadership instability as at least one additional valid leader election or ballot advance after the expected post-failure recovery election. Transient timeout observations that do not produce an additional valid election are not classified as instability.

The primary matched comparison uses N=5,K=11, with identical protocol configuration, workload, scheduler, network model, and seeds. We additionally evaluate RoundTick across multiple delay budgets at N=5 and representative configurations at N=3 and N=7.

## 5. Results

### 5.1 Time semantics qualitatively change the observed stability result

We first compare the two time models at N=5, K=11, using the per-sender network model and the same 20 deterministic seeds. Under EventCoupled timing, 15 of 20 executions (75%) exhibited leadership instability after the expected recovery election. The average number of view changes was 2.85, with a maximum of four.

Under RoundTick timing, none of the 20 executions exhibited an additional valid leader election. The instability rate therefore changed from 75% to 0%, while the average number of view changes decreased from 2.85 to 1.00. All executions preserved the expected Multi-Paxos safety property.

| Time model | Runs | Unstable | Instability | Avg. views | Max views | Avg. recovery tick | Avg. stable tick |

> |---|---:|---:|---:|---:|---:|---:|---:|
> | EventCoupled | 20 | 15 | 75% | 2.85 | 4 | 130.90 | 225.40 |
> | RoundTick | 20 | 0 | 0% | 1.00 | 1 | 37.30 | 37.30 |

Because the protocol, scheduler, network model, workload, and seeds are held fixed, this comparison isolates simulated time semantics as the changed experimental dimension. The previously observed leadership churn therefore does not persist when timeout progression is decoupled from individual scheduler events.

### 5.2 The absence of additional elections persists across tested configurations

We next varied the adversarial delay budget and cluster size under RoundTick. At N=5, no additional valid leader elections occurred across the tested delay budgets K∈{0,6,8,10,11,12,14}. Average recovery time increased modestly with the delay budget, from 35.00 logical ticks at K=0 to 38.00 at K=14, showing that adversarial scheduling continued to affect recovery latency even though it no longer produced leadership churn.

We additionally evaluated representative configurations for N=3 and N=7. No additional valid elections occurred in any of these executions. Across the RoundTick validation campaign, 260 executions produced zero additional valid leader elections.

|         N | K values                |    Runs | Unstable |
| --------: | ----------------------- | ------: | -------: |
|         3 | 10, 16, 24              |      60 |        0 |
|         5 | 0, 6, 8, 10, 11, 12, 14 |     140 |        0 |
|         7 | 2, 6, 10                |      60 |        0 |
| **Total** |                         | **260** |    **0** |


The RoundTick result does not imply that adversarial delay has no effect. Recovery latency still changes with K. Rather, the qualitative effect changes: bounded message delay can slow recovery without necessarily inducing additional valid leader elections. This distinction suggests that the earlier leadership-instability result was produced by the interaction between adversarial scheduling and event-coupled timeout progression, rather than by the targeted message delay alone.

## 6. Matched-Trace Mechanism

## 7. Methodological Implications

## 8. Limitations

## 9. Threats to Validity

## 10. Conclusion