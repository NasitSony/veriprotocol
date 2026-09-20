#!/bin/bash

echo "seed,fifo_recovery,promise_recovery,slowdown"

for seed in $(seq 1 20)
do
    fifo=$(cargo run --quiet -- fifo 1 "$seed" stable-multi-paxos 5 20 4 0.5 2>/dev/null \
        | grep "Multi-paxos Recovery Completed Step:" \
        | sed -E 's/.*Some\(([0-9]+)\).*/\1/')

    promise=$(cargo run --quiet -- mp-promise-delay 1 "$seed" stable-multi-paxos 5 20 4 0.5 2>/dev/null \
        | grep "Multi-paxos Recovery Completed Step:" \
        | sed -E 's/.*Some\(([0-9]+)\).*/\1/')

    if [[ "$fifo" =~ ^[0-9]+$ && "$promise" =~ ^[0-9]+$ ]]; then
        slowdown=$((promise - fifo))
    else
        slowdown="NA"
    fi

    echo "$seed,$fifo,$promise,$slowdown"
done