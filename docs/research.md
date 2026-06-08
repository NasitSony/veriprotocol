## Fairness Model

VeriProtocol distinguishes between eventual fairness and bounded fairness.

Eventual fairness requires that every message is eventually delivered. This is useful for asynchronous protocols where late delivery may still contribute to progress.

Bounded fairness requires that every message is delivered within K scheduler opportunities. This is more relevant for timeout-based or partially synchronous protocols, where a message delivered after a timeout may no longer help the current view.

## Timeout Interaction

Let T be the timeout threshold and K be the scheduler fairness bound.

- If K <= T, delayed messages still arrive before timeout.
- If K > T, messages are eventually delivered but may arrive too late for the current view.

This creates a distinction between slow delivery and liveness disruption.


Vary K
Vary T
Measure:
- decision success
- view changes
- messages-to-decide
- rounds-to-decide

