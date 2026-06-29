- Decouple protocol-specific message types from framework messages.

1. Message abstraction
2. PBFTProtocol
3. HotStuffProtocol
4. MVBAProtocol
5. Scheduler Vulnerability Index



## Timeout Threshold T

Current timeout behavior injects timeout messages immediately.

Next model:

A timeout becomes enabled only after T scheduler delivery opportunities.

This allows experiments comparing scheduler fairness bound K against timeout threshold T.

Expected relationship:

- If K <= T, delayed proposals should arrive before timeout.
- If K > T, proposals may arrive eventually but too late for the current view.
- This can cause view churn without permanent message loss.

### Ballot-Density Scheduler

Current deterministic schedulers intentionally delay lower-ballot
messages but create at most three concurrent ballots.

Future work:

Design a scheduler that explicitly maximizes the number of simultaneously
active ballot generations while preserving bounded delay.

Hypothesis:

Increasing concurrent ballot overlap will reproduce the heavy retry
cascades currently observed only under rare random schedules.