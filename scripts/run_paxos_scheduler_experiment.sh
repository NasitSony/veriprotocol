#!/bin/bash
set -e

OUT="results/paxos_scheduler_experiment.csv"

echo "scheduler,seed,k,messages_sent,scheduler_steps,paxos_retries,nack_messages,max_ballot_seen,safety_violation" > "$OUT"

# FIFO baseline
cargo run --quiet -- fifo 1 0 paxos 5 6 5 | awk -v scheduler="fifo" -v seed=0 -v k=0 '
/Messages Sent:/ {sent=$3}
/Scheduler Steps:/ {steps=$3}
/Paxos Retries:/ {retries=$3}
/Nack Messages:/ {nacks=$3}
/Max Ballot Seen:/ {ballot=$4}
/Safety Violation:/ {safety=$3}
END {print scheduler "," seed "," k "," sent "," steps "," retries "," nacks "," ballot "," safety}
' >> "$OUT"

# Random seed sweep
for seed in $(seq 1 200); do
  cargo run --quiet -- random 1 "$seed" paxos 5 6 5 | awk -v scheduler="random" -v seed="$seed" -v k=0 '
  /Messages Sent:/ {sent=$3}
  /Scheduler Steps:/ {steps=$3}
  /Paxos Retries:/ {retries=$3}
  /Nack Messages:/ {nacks=$3}
  /Max Ballot Seen:/ {ballot=$4}
  /Safety Violation:/ {safety=$3}
  END {print scheduler "," seed "," k "," sent "," steps "," retries "," nacks "," ballot "," safety}
  ' >> "$OUT"
done

# Critical-delay budget sweep
for k in $(seq 0 20); do
  cargo run --quiet -- critical-delay 1 42 paxos 5 "$k" 5 | awk -v scheduler="critical-delay" -v seed=42 -v k="$k" '
  /Messages Sent:/ {sent=$3}
  /Scheduler Steps:/ {steps=$3}
  /Paxos Retries:/ {retries=$3}
  /Nack Messages:/ {nacks=$3}
  /Max Ballot Seen:/ {ballot=$4}
  /Safety Violation:/ {safety=$3}
  END {print scheduler "," seed "," k "," sent "," steps "," retries "," nacks "," ballot "," safety}
  ' >> "$OUT"
done

echo "Wrote $OUT"
