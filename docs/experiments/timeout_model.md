A timeout fires when proposer has not received quorum response by T scheduler steps.

Timeout creates a new ballot.

Stale Promise messages are ignored.

Stale NACKs are ignored if nack.ballot != current_ballot.

Stale AcceptRequest may produce NACK, but stale NACK does not cause retry.