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