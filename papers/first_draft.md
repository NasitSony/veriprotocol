# VeriProtocol: Scheduler-Aware Evaluation of Consensus Protocols

## Abstract

Consensus protocol evaluations typically focus on protocol design, fault tolerance, and network conditions. However, message scheduling behavior can significantly influence protocol progress, timeout recovery, and useful message delivery.

This paper presents VeriProtocol, a scheduler-aware consensus evaluation framework for studying the interaction between protocol behavior and adversarial message scheduling. VeriProtocol supports multiple protocol implementations, pluggable schedulers, timeout-driven recovery, view changes, and stale message analysis.

Experiments demonstrate that scheduler behavior significantly affects protocol execution. In a 4-node timeout-based protocol, different schedulers require different timeout thresholds to avoid unnecessary view changes. For example, FIFO scheduling requires a timeout threshold of only 4 logical steps, while DelayLeader and TimeoutFirst schedulers require thresholds of 13 and 16 respectively.

The results suggest that timeout configuration cannot be evaluated independently of scheduler behavior and motivate scheduler-aware evaluation as an additional dimension of consensus protocol analysis.

## 1. Introduction

Consensus protocols are typically evaluated under assumptions about network delay, failures, and throughput. While these evaluations provide insight into protocol performance, they often treat message scheduling as an implementation detail rather than a first-class experimental variable.

This work explores scheduler-aware consensus evaluation. We investigate how different message scheduling strategies influence protocol behavior, timeout recovery, view changes, and useful progress.

We make the following contributions:

1. VeriProtocol, a scheduler-aware consensus evaluation framework.
2. A taxonomy of adversarial schedulers.
3. Experimental analysis of scheduler-specific timeout sensitivity.
4. An observation that eventual message delivery does not necessarily imply useful delivery.

## 2. Motivation

A message may be delivered after a timeout has already triggered a view change. Such messages are eventually delivered but may no longer contribute to protocol progress.

This distinction motivates the concept of useful delivery and highlights the importance of scheduler-aware evaluation.

## 3. Architecture

* Protocol abstraction
* Scheduler abstraction
* Network model
* Timeout model
* View-change mechanism
* Metrics subsystem

## 4. Scheduler Taxonomy

* FIFO Scheduler
* Random Scheduler
* DelayLeader Scheduler
* DelayProposal Scheduler
* DelayVote Scheduler
* DelayCommit Scheduler
* TimeoutFirst Scheduler
* QuorumBlocking Scheduler

## 5. Experimental Results

### Scheduler-Specific Timeout Sensitivity

| Scheduler    | Critical Timeout T* |
| ------------ | ------------------: |
| FIFO         |                   4 |
| DelayLeader  |                  13 |
| TimeoutFirst |                  16 |

These results indicate that timeout configuration depends strongly on scheduler behavior.

### Eventual Delivery vs Useful Delivery

Experiments show that delayed leader proposals may be eventually delivered after replicas have already advanced to a newer view. Such messages become stale and are ignored despite successful delivery.

## 6. Discussion

The results suggest that scheduler behavior should be treated as a first-class experimental variable when evaluating consensus protocols.

## 7. Future Work

* Fair adversarial schedulers
* K-bounded fairness
* PBFT and HotStuff implementations
* Scheduler Vulnerability Index
* Scheduler-aware liveness analysis
