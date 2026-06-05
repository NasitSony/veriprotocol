# Next Improvements

## Option 4: Messages Sent Until Decision
- Add messages_sent_until_decision metric
- Record when all nodes have decided
- Compare with decision_delivery_count

## Option 2: Large Random Experiments
- Run 100 random schedules
- Record min/max/avg
- Save results in experiments.md

## Option 3: Seeded Random Scheduler
- CLI: cargo run -- random 10 42
- Use StdRng
- Make experiments reproducible