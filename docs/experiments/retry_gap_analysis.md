# Retry Gap Analysis (Corrected Implementation)

Protocol:
- Basic Paxos with partial timeout
- Prepare >= promised_ballot fix applied

Random scheduler:
- 200 seeds

Top-10 retry seeds:
...

Retry motifs:

Prepare NACKs: 80
AcceptRequest NACKs: 94

Gap analysis:

gap=1 : 134
gap>=2: 40

Observation:

Most retry-causing collisions occur when the incoming ballot is exactly
one behind the promised ballot.

Hypothesis:

Repeated gap=1 collisions, rather than deep stale messages,
drive retry cascades.