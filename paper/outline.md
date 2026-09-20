# Working Title

## 1. Problem and Motivation

Adversarial schedulers are useful for evaluating the liveness behavior of distributed consensus protocols by systematically delaying, reordering, or prioritizing protocol messages. Such simulations, however, require not only a model of message delivery but also a model of time. This distinction becomes important for timeout-driven protocols: if protocol time advances with scheduler activity, the number of simulated message-delivery events can inadvertently determine when failure detectors expire.

We encountered this issue while evaluating Multi-Paxos under bounded adversarial message scheduling. Under an event-coupled time model, targeted delay of AcceptRequest traffic appeared to induce substantial post-recovery leadership instability. The effect persisted after replacing a global network queue with per-sender scheduling, suggesting that global serialization was not its primary cause.

However, when timeout progression was decoupled from individual scheduler events, the apparent instability disappeared. This led us to ask:

## 2. Research Question

To what extent can the time semantics of an adversarial consensus simulator alter conclusions about protocol liveness?
 
## 3. Simulator and Time Models

VeriProtocol is a deterministic consensus simulation framework in which protocol messages are placed into a simulated network and delivered according to a configurable scheduler. The scheduler can perturb execution by delaying or selecting messages, while seeded executions provide reproducible schedules for controlled comparisons.

This study considers stable Multi-Paxos with leader failure and recovery. Followers maintain heartbeat-based failure detectors and initiate a new election when their heartbeat age reaches an election-timeout threshold. During the investigation, we considered two semantics for advancing simulated protocol time.

### EventCoupled

Under EventCoupled semantics, each scheduler step advances protocol time. Consequently, message deliveries, adversarial delay decisions, and other scheduler activity also advance heartbeat and timeout state. Scheduler activity can therefore accelerate failure-detector progression even when the number of scheduler events does not correspond to an equivalent amount of elapsed time.

### RoundTick

Under RoundTick semantics, protocol-time progression is decoupled from individual scheduler events. For a configuration with N nodes, one logical tick occurs after N scheduler opportunities, and heartbeat and failure-detector timers advance only on these logical ticks. RoundTick therefore removes the direct proportionality between scheduler-event count and protocol-time progression.

This distinction matters because wall-clock failure detectors expire according to elapsed time rather than according to the number of messages delivered or scheduling decisions performed. RoundTick is therefore more faithful along the specific dimension studied here: scheduler activity alone does not cause protocol time to advance at the same rate as under EventCoupled semantics. We do not claim that RoundTick exactly reproduces physical wall-clock execution; it remains a deterministic logical-time abstraction that corrects this particular event-time coupling.

The two models allow us to hold the protocol, scheduler, network model, workload, and random seed fixed while changing how scheduler progress is translated into protocol time.

RoundTick does not disable timeout-driven elections. As demonstrated by the heartbeat-withholding control in Section 6, an election can still occur when leader contact remains unavailable until the configured logical timeout threshold is reached.

## 4. Experimental Setup

We evaluate a Multi-Paxos recovery scenario in which the initial leader fails and the remaining replicas must elect a replacement while preserving previously accepted values. The adversarial scheduler targets Multi-Paxos AcceptRequest traffic using a bounded delay budget K. Safety is checked throughout each execution, and recovery is considered stable when a heartbeat from the current highest-ballot post-failure leader is successfully delivered.

We initially evaluated a global network queue, then introduced a per-sender network model to test whether observed instability resulted from serialization of otherwise independent communication paths. At N=5,K=11, instability decreased only modestly from 85% under the global queue to 75% under per-sender scheduling, indicating that global serialization alone did not explain the effect.

We subsequently compared EventCoupled and RoundTick using the per-sender network model. Unless otherwise noted, each configuration was evaluated across 20 deterministic seeds. We define leadership instability as at least one additional valid leader election or ballot advance after the expected post-failure recovery election. Transient timeout observations that do not produce an additional valid election are not classified as instability.

At N=5, K=11, instability decreased only modestly from 85% under the global queue to 75% under per-sender scheduling, indicating that global serialization alone did not explain the effect. However, both experiments retained EventCoupled time semantics; the subsequent time-model comparison showed that this shared assumption was the more consequential source of the observed instability.

## 5. Results

### 5.1 Time semantics qualitatively change the observed stability result

We first compare the two time models at N=5, K=11, using the per-sender network model and the same 20 deterministic seeds. Under EventCoupled timing, 15 of 20 executions (75%) exhibited leadership instability after the expected recovery election. The average number of view changes was 2.85, with a maximum of four.

Under RoundTick timing, none of the 20 executions exhibited an additional valid leader election. The instability rate therefore changed from 75% to 0%, while the average number of view changes decreased from 2.85 to 1.00. All executions preserved the expected Multi-Paxos safety property.

| Time model | Runs | Unstable | Instability | Avg. views | Max views | Avg. recovery tick | Avg. stable tick |
|---|---:|---:|---:|---:|---:|---:|---:|
| EventCoupled | 20 | 15 | 75% | 2.85 | 4 | 130.90 | 225.40 |
| RoundTick | 20 | 0 | 0% | 1.00 | 1 | 37.30 | 37.30 |

Because the protocol, scheduler, network model, workload, and seeds are held fixed, this comparison changes only the simulator's protocol-time semantics. The previously observed leadership churn therefore does not persist when protocol-time progression is decoupled from individual scheduler events.

### 5.2 The absence of additional elections persists across tested configurations

We next varied the adversarial delay budget and cluster size under RoundTick. At N=5, no additional valid leader elections occurred across the tested delay budgets K∈{0,6,8,10,11,12,14}. Average recovery time increased modestly with the delay budget, from 35.00 logical ticks at K=0 to 38.00 at K=14, showing that adversarial scheduling continued to affect recovery latency even though it no longer produced leadership churn.

We additionally evaluated representative configurations for N=3 and N=7. No additional valid elections occurred in any of these executions. Across the RoundTick validation campaign, 260 executions produced zero additional valid leader elections.

| N | K values | Runs | Unstable |
|---:|---|---:|---:|
| 3 | 10, 16, 24 | 60 | 0 |
| 5 | 0, 6, 8, 10, 11, 12, 14 | 140 | 0 |
| 7 | 2, 6, 10 | 60 | 0 |
| **Total** | | **260** | **0** |


The RoundTick result does not imply that adversarial delay has no effect. Recovery latency still changes with K. Rather, the qualitative effect changes: bounded message delay can slow recovery without necessarily inducing additional valid leader elections. This distinction suggests that the earlier leadership-instability result was produced by the interaction between adversarial scheduling and event-coupled timeout progression, rather than by the targeted message delay alone.

## 6. Matched-Trace Mechanism
To understand why the two time models produce different leadership outcomes, we examine a matched execution using seed 42 at N=5 and K=11. The protocol configuration, workload, adversarial scheduler, per-sender network model, and random seed are identical across the two executions; only the semantics of protocol-time progression differ. This comparison allows us to trace how scheduler activity interacts with heartbeat generation and follower timeout state under EventCoupled and RoundTick.

The matched execution highlights that the difference is not simply a reduction in scheduler activity. In the seed-42 comparison, EventCoupled reached stable recovery at scheduler step 176 after an additional election raised the ballot to 3. RoundTick, by contrast, required 191 scheduler steps to reach stable recovery but advanced through only 38 logical ticks and remained at ballot 2. Thus, RoundTick performed more raw scheduler work before stable recovery while avoiding the additional election. This contrast provides direct evidence for the mechanism: under EventCoupled, scheduler activity itself contributes directly to timeout progression, whereas under RoundTick the same activity does not imply an equivalent amount of elapsed protocol time.

The heartbeat-withholding control distinguishes decoupling from suppression. When a ballot-2 heartbeat from the live recovery leader was deliberately withheld from a follower until its timeout threshold was reached under RoundTick, the follower initiated an additional valid election and the execution advanced to ballot 3. RoundTick therefore does not prevent adversarially induced elections. Instead, it requires heartbeat unavailability to persist across sufficient logical time, rather than allowing scheduler activity alone to accelerate timeout expiration.

EventCoupled + scheduler activity
        ↓
timeout state advances with events
        ↓
additional election under EventCoupled timing

RoundTick + scheduler activity
        ↓
insufficient logical timeout duration
        ↓
no additional election

RoundTick + heartbeat withheld
for full logical timeout duration
        ↓
timeout expires
        ↓
valid additional election

## 7. Methodological Implications


Our results show that time semantics are part of the experimental model in adversarial consensus simulation, rather than an implementation detail. When timeout progression is coupled to individual scheduler events, an adversarial scheduler can influence not only which messages are delayed or delivered, but also how quickly protocol failure detectors perceive time to be passing. This creates an additional path through which scheduler activity can affect protocol behavior.

The EventCoupled and RoundTick comparison demonstrates the consequence of this coupling. Under EventCoupled semantics, bounded perturbations of Multi-Paxos traffic produced additional leader elections in 75% of the evaluated executions. After timeout progression was decoupled from individual scheduler events, these additional elections disappeared across the evaluated N and K configurations. The matched-trace analysis further showed that this difference was not caused by RoundTick performing less scheduler work: the RoundTick execution required more scheduler steps before stable recovery while avoiding the additional election.



These observations suggest a methodological requirement for adversarial liveness evaluation: the scheduler's control over message ordering and delay should be separated from the mechanism by which protocol time advances. Otherwise, an adversary intended to manipulate communication may inadvertently gain control over the effective rate of timeout progression as well. Apparent liveness degradation can then reflect the simulator's clock semantics rather than the modeled communication perturbation alone.

More generally, evaluations of timeout-sensitive distributed protocols should report their time semantics explicitly. Results involving leader elections, retries, failure detectors, leases, or other timer-driven behavior may depend on whether simulated time advances with events, rounds, or an independent clock. Reproducible adversarial schedules are therefore not sufficient by themselves; the mapping from scheduler execution to protocol time is also part of the experimental specification.

## 8. Limitations

This study evaluates time semantics within a deterministic simulation of stable Multi-Paxos rather than a deployment using physical clocks and real network delays. RoundTick should therefore not be interpreted as an exact representation of wall-clock execution. It is a controlled abstraction that removes the direct coupling between individual scheduler events and protocol-time progression. Our conclusion is consequently about the sensitivity of simulated liveness behavior to this coupling, rather than a claim that RoundTick reproduces all timing behavior of deployed systems.

The experimental scope is limited to stable Multi-Paxos and a specific leader-failure-and-recovery scenario. Although the underlying concern applies conceptually to other timeout-sensitive protocols, we have not demonstrated the same effect for Raft, PBFT, HotStuff, or other consensus protocols. Generalization beyond the evaluated protocol and scenario therefore remains future work.

Our adversarial scheduler targets a specific class of Multi-Paxos traffic using bounded delay budgets. Other adversaries, including direct heartbeat delay, partitions, correlated delays, or adaptive scheduling strategies, may produce different liveness behavior under RoundTick; our results should therefore not be interpreted as showing that RoundTick prevents adversarially induced leadership changes.

Finally, RoundTick advances logical time at a fixed relationship to scheduler opportunities. This removes the one-event-one-time-unit coupling of EventCoupled, but it remains a discrete simulation policy rather than an independently modeled physical clock. Future work could evaluate alternative decoupled clock models, including explicit timer events or simulated wall-clock advancement, to determine how sensitive the observed results are to the particular decoupling strategy.


## 9. Threats to Validity

One threat concerns the implementation of the two time models. EventCoupled and RoundTick are implemented within the same simulator, so implementation errors could affect the comparison. We mitigate this risk by holding the protocol, scheduler, network model, workload, and random seeds fixed while changing only the rule governing timeout progression. We also instrument scheduler steps and logical ticks separately and use matched-seed traces to inspect the mechanism directly.

A second threat concerns the definition of leadership instability. We classify an execution as unstable only when recovery is followed by an additional valid leader election and ballot advance. Timeout observations from replicas that still hold stale leader state are not counted as instability unless they result in such an election. This distinction prevents transient timeout observations from being interpreted as leadership churn.

A third threat is that RoundTick might suppress timeout behavior generally rather than specifically remove event-time coupling. The heartbeat-withholding control in Section 6 addresses this alternative explanation by demonstrating that timeout-driven additional elections remain reachable under RoundTick.

Finally, the evaluated executions use deterministic seeded schedules and a finite set of cluster sizes and delay budgets. These experiments establish the phenomenon within the evaluated configuration space but do not exhaust all possible schedules or adversarial strategies. We therefore treat the results as evidence of a specific simulator-modeling hazard rather than a universal characterization of consensus liveness.


## 10. Conclusion

This study began with an apparent Multi-Paxos liveness result: bounded adversarial delay of AcceptRequest traffic produced repeated post-failure leader elections. Initial validation showed that the effect persisted after replacing a globally serialized network queue with per-sender scheduling, suggesting that queue serialization alone did not explain the instability.

A more fundamental comparison revealed that the result depended strongly on the simulator's treatment of time. Under EventCoupled semantics, where individual scheduler events advance timeout state, additional leader elections occurred in 75% of the evaluated executions. Under RoundTick, which decouples timeout progression from individual scheduler events, no additional elections occurred across the evaluated cluster sizes and delay budgets.

Matched-seed analysis provided evidence for the mechanism behind this difference. RoundTick could perform more scheduler work before stable recovery while accumulating substantially less logical time, demonstrating that scheduler-event count and protocol-time progression need not advance proportionally.

The resulting lesson is methodological rather than a claim about Multi-Paxos vulnerability. In adversarial consensus simulation, message scheduling and protocol-time progression are distinct experimental dimensions. Coupling them can allow scheduler activity to alter the effective rate of timeout progression and thereby manufacture apparent liveness degradation. Evaluations of timeout-sensitive distributed protocols should therefore make their time semantics explicit and, when studying communication adversaries, separate timeout progression from individual scheduler events.