Timing-model artifact in Multi-Paxos liveness evaluation

Holding protocol, workload, adversarial scheduler, network model, and seeds fixed, changing only timeout-time semantics eliminates the previously observed post-recovery leadership churn. Under per-sender EventCoupled timing, N=5/K=11 produced extra valid leader elections in 15/20 runs (75%). Under RoundTick, the same configuration produced 0/20. Additional RoundTick experiments across N=3/5/7 and multiple K values produced no extra valid elections across 260 runs.

Instability definition: a run is unstable when recovery is followed by at least one additional valid leader election/ballot advance. Timeout log events alone are not counted as instability. This distinction matters because RoundTick can produce transient timeout observations at nodes that still hold stale leader state without causing another valid election.

Mechanism: EventCoupled advances follower failure-detector age with scheduler activity. In the matched N=5/K=11/seed=42 execution, leader 2 is elected at scheduler step/logical tick 86, but scheduler activity advances logical time rapidly enough for candidate 3 to reach heartbeat_age=20 and trigger another election at step 109. Under RoundTick, leader 2 is elected at step 140/logical tick 28 and reaches stable recovery at step 191, around logical tick 38, without another valid election.

Critically, RoundTick performs more scheduler work before stable recovery (191 vs. 176 steps) while producing less elapsed protocol time and no spurious election. This isolates the problem: EventCoupled conflates scheduler activity with elapsed timeout time.

Conclusion: the earlier claim that AcceptRequest-targeted delay inherently causes Multi-Paxos leadership instability is not supported. The observed churn depends on event-coupled timeout semantics. Adversarial consensus simulations that advance failure detectors with scheduler-event counts can therefore manufacture apparent liveness failures. Timeout progression should be decoupled from individual scheduler activity.

Limitation: RoundTick is not claimed to reproduce physical wall-clock time exactly. It is a discrete logical-time abstraction designed to remove the direct coupling between individual scheduler events and timeout progression.