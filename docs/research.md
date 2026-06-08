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

We are working on VeriProtocol, a Rust scheduler-aware consensus simulation framework.

Current status:
- Clean framework with Scheduler trait and Protocol trait.
- Existing protocols:
  1. SimpleConsensusProtocol: Proposal -> Vote -> Commit -> Decision
  2. TwoPhaseProtocol: Proposal -> Vote -> Decision
  3. TimeoutProtocol skeleton: leader-based Proposal -> Vote -> Decision, with view/leader/timeout fields added.
- Existing schedulers:
  FIFO, Random, DelayNode, DelayProposal, DelayVote, DelayCommit, BoundedDelay, ProbabilisticDelay, QuorumBlocking.
- Metrics:
  messages_sent
  messages_delivered
  decision_delivery_count / should rename to messages_delivered_until_decision
  messages_sent_until_decision
  undelivered_messages_at_decision
  decisions
- Docs created:
  architecture.md
  experiments.md
  findings.md
  results.csv
  future_work.md
  fairness_timeout_model.md

Important recent result:
TimeoutProtocol skeleton now runs:
Messages Sent: 32
Messages Delivered: 28
Decision Delivery Count: 28
Messages Sent Until Decision: 32
Undelivered Messages At Decision: 4
Decisions: 4

Current next task:
Make TimeoutProtocol more realistic by changing initial proposal generation so only the current leader broadcasts Proposal. Right now all nodes still send Proposal, but TimeoutProtocol only accepts proposal if msg.from == node.leader.

Need help with:
1. Find where initial Proposal messages are broadcast in Simulation.
2. Add protocol-aware initialization, or a simple leader-only proposal path for TimeoutProtocol.
3. Keep SimpleConsensusProtocol and TwoPhaseProtocol behavior unchanged.
4. Run:
   cargo check
   cargo run -- fifo 1 42 timeout
   cargo run -- fifo 1 42 simple
   cargo run -- fifo 1 42 two-phase
5. Commit message:
   Add leader-only proposal initialization for timeout protocol

Research direction:
We are moving toward fair-adversary timeout experiments. Need to distinguish:
- eventual fairness: every message eventually delivered
- K-bounded fairness: every message delivered within K scheduler opportunities
Timeout relation:
- if K <= T, message arrives before timeout
- if K > T, message eventually arrives but may be too late for current view

Later goal:
Build fair scheduler + timeout/view-change model where a scheduler can cause leader churn or non-termination-like behavior without dropping messages forever.