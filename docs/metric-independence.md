# Metric Independence Analysis

## Question

Are GDS (Graceful Degradation Score) and ARR (Autonomous Recovery Rate) statistically independent? Do they double-count recovery behavior?

## Analysis

### GDS: What it measures

GDS = (1/n) * sum(C_i) where C_i = completion_rate at stress level i.

GDS captures how work completion degrades as stress intensity increases. A GDS of 1.0 means all tasks completed at every stress level. A GDS of 0.5 means half the work was completed on average.

### ARR: What it measures

ARR = Fa / Fr where Fa = autonomously recovered failures, Fr = total recoverable failures.

ARR captures the system's ability to self-heal from recoverable faults. An ARR of 1.0 means every recoverable fault was resolved without external intervention.

### Shared input events

In W1-A, when `should_inject_fault()` returns true:
- A `Failure` event with `FailureClass::AutonomouslyRecovered` is emitted
- The task is NOT completed (`continue` skips work)
- `tasks_completed` is NOT incremented

This means:
- **GDS impact**: The task was skipped, so `completion_rate` decreases. More faults = lower GDS.
- **ARR impact**: The fault was autonomously recovered, so it counts in both Fa and Fr. More faults with recovery = ARR stays high.

### Are they double-counting?

No. The same fault event affects both metrics, but in **different directions** through **different causal pathways**:

| Metric | Effect of fault+recovery | Mechanism |
|--------|--------------------------|-----------|
| GDS | **Decreases** | Faulted task not completed -> lower completion_rate |
| ARR | **Stays high** (or increases) | Fault classified as AutonomouslyRecovered -> Fa/Fr = 1 |

### Independence demonstration

A system with many faults that all recover will have:
- **Low GDS** (many tasks skipped)
- **High ARR** (all faults recovered)

A system with few faults, none recovered, will have:
- **High GDS** (most tasks completed)
- **Low ARR** (no autonomous recovery)

This demonstrates that GDS and ARR can vary independently. They are not redundant.

### Analogy

GDS and ARR are analogous to precision and recall in machine learning:
- Both derive from the same confusion matrix (same predictions/events)
- They measure genuinely different dimensions
- They can trade off against each other
- Combining them (F1 = harmonic mean) is meaningful because they capture complementary information

Similarly, combining GDS and ARR in SRI is meaningful because they capture complementary resilience dimensions: degradation behavior vs recovery capability.

## Conclusion

**No correction needed.** GDS and ARR are not statistically independent (they share fault events as inputs), but they are semantically independent (they measure different properties of the system's response to those events). This is an acceptable and intentional design choice consistent with established composite metric practices.

## Verification Test Cases

**Scenario A**: 100 tasks, 50 faults, all autonomously recovered
- GDS = 0.5 (50% completion)
- ARR = 1.0 (all faults recovered)

**Scenario B**: 100 tasks, 2 faults, neither recovered (both RecoverableNotRecovered)
- GDS = 0.98 (98% completion)
- ARR = 0.0 (no autonomous recovery)

**Scenario C**: 100 tasks, 0 faults
- GDS = 1.0 (100% completion)
- ARR = N/A (no recoverable failures observed -> Fr = 0)

GDS and ARR vary independently across all three scenarios.
