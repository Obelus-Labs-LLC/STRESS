# STRESS Validation Methodology

## Purpose

Validate that SRI scores correlate with real-world system resilience by testing against systems with known resilience characteristics.

## Methodology

### 1. Known-Resilient System: Redis Sentinel

**Setup**: 3-node Redis Sentinel cluster (1 master, 2 replicas, 3 sentinels)

**Expected scores**:

| Proxy | Expected Range | Rationale |
|-------|---------------|-----------|
| ARR | 0.8 - 1.0 | Sentinel automatically promotes replica on master failure |
| IST | 0.8 - 1.0 | Sentinel continues serving reads during network partition |
| GDS | 0.5 - 0.8 | Write availability degrades during failover window |
| CFR | 0.3 - 0.6 | Master failure affects connected write clients |
| REC | 0.5 - 0.8 | Failover has overhead; throughput drops temporarily |

**STRESS workload mapping**:
- W3-A (distributed coordination): Leader election maps to Sentinel failover
- SP-1: Kill master process (SIGKILL) at Poisson-distributed intervals
- SP-5: Network partition sentinel from master (iptables DROP)

### 2. Known-Fragile System: Single-Process SQLite

**Setup**: Single SQLite process with WAL mode, no replication

**Expected scores**:

| Proxy | Expected Range | Rationale |
|-------|---------------|-----------|
| ARR | 0.0 - 0.2 | No autonomous recovery from process kill |
| IST | 0.0 - 0.2 | Cannot survive isolation (single process, no peers) |
| GDS | 0.3 - 0.6 | Degrades rapidly under increasing stress |
| CFR | 0.0 - 0.2 | Single component = total cascade on failure |
| REC | 0.2 - 0.5 | No redundancy to maintain efficiency |

### 3. Correlation Analysis Protocol

After running STRESS against both systems under identical stress profiles (SP-1, SP-2):

1. **Compute SRI** for each system
2. **Inject real faults** using system-level tools:
   - `kill -9 <pid>` for process failure
   - `iptables -A INPUT -s <ip> -j DROP` for network partition
   - cgroups memory.max for memory pressure
3. **Measure actual resilience dimensions**:
   - Recovery time (seconds from failure to service restoration)
   - Cascade spread (fraction of components affected)
   - Survival duration during isolation
   - Completion rate under stress
4. **Compute Spearman rank correlation** between each SRI proxy score and its real-world counterpart

### 4. Acceptance Criteria

| Criterion | Threshold |
|-----------|-----------|
| Rank correlation per proxy | >= 0.7 |
| SRI difference (Redis - SQLite) | >= 20 points |
| Proxy ordering | Redis ARR > SQLite ARR, Redis IST > SQLite IST |
| Directional consistency | Higher stress profile -> lower SRI for both systems |

### 5. Limitations

- STRESS measures simulated workload behavior, not real application behavior
- Correlation validates the measurement model, not predictive power for arbitrary systems
- Small sample size (2 systems) limits statistical confidence; expand to 10+ for publication
- Real-world failure modes may not map cleanly to STRESS stress parameters
- Environmental coupling (thermal, radiation) cannot be physically reproduced without specialized hardware

## Future Work

1. Expand validation corpus to 10+ systems spanning different resilience classes (etcd, PostgreSQL, CockroachDB, Kubernetes pods, bare processes)
2. Automate validation pipeline with containerized test environments
3. Publish reference SRI scores for common systems as calibration baselines
4. Investigate whether SRI ranking is stable across stress profiles (SP-1 vs SP-2)
5. Cross-validate with chaos engineering platforms (LitmusChaos, Chaos Mesh) fault injection results
