# Gap-1 Scheduler

Goal

Engineer repeated gap=1 collisions.

Rule

1. Hold stale ballot b messages.
2. Deliver Prepare(b+1) until quorum.
3. Release stale ballot b.
4. Trigger NACK.
5. Retry.
6. Repeat.

Observation

Initial implementation:
- 34 steps
- 2 retries

Improved implementation:
- 74 steps
- 4 retries

Current implementation:

Execution does not terminate.

Example run:

- scheduler steps >130000
- ballot >6500

The execution continuously produces

Prepare(b+1)
→ stale AcceptRequest(b)
→ gap=1 NACK
→ retry(b+2)

The run was terminated manually.

Important:

The execution cap is an experimental measurement limit,
not evidence that the cascade terminates.

# Gap1 Scheduler Analysis

After correcting Paxos Prepare handling (`>=` instead of `>`), random retry tails remained.  
NACK cause analysis showed that most retry-causing conflicts are gap=1 conflicts: the incoming ballot is exactly one behind the acceptor's promised ballot.

However, experiments with deterministic Gap1 schedulers show that gap=1 conflicts alone are not sufficient.

## Key mechanism

The harmful pattern is bounded two-generation overlap:

```text
Prepare(b+1)
→ acceptors promise b+1
→ stale AcceptRequest(b) is still in the queue
→ AcceptRequest(b) is delivered
→ NACK with gap=1
→ proposer retries to b+2
→ repeat