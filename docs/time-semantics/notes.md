# Time-Semantics Experiment Notes

## 2026-09-18 — EventCoupled Multi-Paxos Grid

Configuration:

- Protocol: stable Multi-Paxos
- Nodes: N=5
- Network model: per-sender
- Time model: EventCoupled
- Runs per configuration: 20
- Seeds: 0–19
- Scheduler: mp-accept-request-delay
- Timeout threshold: 5
- Delay probability: 0.5

## Batch-runner correction

During experiment reproduction, the multi-run harness was found to compute:

`run_seed = seed + i`

but pass the original `seed` to `Simulation::new()`. Therefore repeated runs used the same seed.

The harness was corrected to pass `run_seed`.

Seed propagation was explicitly verified with a 3-run execution:

- Run 1: seed 0
- Run 2: seed 1
- Run 3: seed 2

The Multi-Paxos aggregation was also changed to report:

- unstable runs
- instability rate
- average/max view changes
- max ballot
- stable-recovery observation count
- average stable-recovery tick among runs where stable recovery was observed

Instability is defined for this experiment as:

`view_changes > 1`

because one view change is the expected leader-failure recovery election.

## Reproduction result

After correcting seed propagation, the previously reported
N=5, K=11 EventCoupled result reproduced:

`15/20 unstable (75%)`

Therefore the headline instability-frequency result survived the
batch-runner correction.

See `eventcoupled-grid.csv` for the corrected N=5 sweep.

## Stable-recovery interpretation

`mp_stable_recovery_tick = None` means stable recovery was not observed
within the experiment window. It must not be interpreted as proof that
the protocol never recovers.

Average stable-recovery tick is calculated only over runs where a
stable-recovery tick was observed.