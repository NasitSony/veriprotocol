Research questions

Scheduler-sensitive leader election

Heartbeat scheduling

Re-election

Membership change

Future experiments

I think you've reached the real research question

I would even rename the scheduler category.

Instead of:

CriticalDelay

I'd think in terms of:

Goal-Oriented Scheduler

Examples:

Paxos Retry Scheduler — maximize ballot retries.
Raft Election Scheduler — maximize leader elections.
Raft Commit Delay Scheduler — maximize commit latency.
Membership Instability Scheduler — maximize configuration convergence time.

Each one is bounded by the same delay budget, but they optimize different protocol outcomes.