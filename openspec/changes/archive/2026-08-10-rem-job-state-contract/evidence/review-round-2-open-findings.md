# Review round 2 disposition

Date: 2026-08-09

Authoritative review report: `/home/lh/.local/state/codex-reviews/2026-08-09/1786282686-127212-773d3df3c19c-plan.md`

Round two returned five findings, the same count as round one, so the change was stood down. On 2026-08-09 the owner released PRD ruling R6 and directed the change to reconcile the four open findings before a new independent review. The original findings remain below with their resolution evidence.

## Resolved findings

### 1. The eighteenth acceptance row does not exist in the released PRD

**Status:** resolved — critical

The change artifacts introduce QCG-18 for `run_rem_noop.sh` and require eighteen acceptance records, while the released PRD defines QCG-1 through QCG-17 and seventeen acceptance rows. The change cannot close against two different ledgers.

**Would close when:** the owner either amends and releases the source PRD with an eighteenth acceptance row and reruns its lint, or directs the change to remove QCG-18 and place the noop-runner proof under an existing released acceptance row. The PRD and OpenSpec counts, identifiers, verifier, and evidence paths must then agree exactly.

**Resolution:** Released ruling R6 keeps seventeen gates, removes QCG-18, and folds both runners' ten-code routing proof into QCG-12. The proposal, design, specification, tasks, verifier names, and evidence paths now use QCG-1 through QCG-17 only.

### 2. The thirteen existing error codes are not frozen to specific classes

**Status:** resolved — high

The artifacts require uniqueness and complete coverage but do not bind each of the thirteen existing REM error codes to one exact outcome class. Different implementations could assign a real error to different exits while still satisfying the current structural checks.

**Would close when:** the specification contains the complete error-code-to-class-and-exit mapping, including `usage_error`, and targeted tests compare every declared tuple byte-for-byte, prove every member is reachable, and fail when any error is moved, omitted, duplicated, or added without a mapping.

**Resolution:** `specs/rem-job-state-contract/spec.md` now carries the exact exit, class, and error-code table from PRD section 5.5, including `usage_error`. Section 2 tests are required to reject moved, omitted, duplicated, or unmapped members.

### 3. The hard-cut publication premise lacks external-consumer evidence

**Status:** resolved — high

The repository-wide caller search and per-hit classification identify `run_rem.sh` and `run_rem_noop.sh` as numeric exit-code consumers and show that `deploy-mem-claw.sh:649-656` consumes JSON rather than the numeric contract. That closes the repository caller inventory, but it does not prove that the old behavior was never delivered outside the checked repositories. The proposal still uses that unproven publication premise to reject compatibility work.

**Would close when:** release tags, package or binary publication records, and deployment manifests are checked and preserved as evidence showing that no external consumer received the old exit contract, or the owner makes an explicit compatibility decision for any discovered external consumer.

**Resolution:** `evidence/release-contract-probe.md` records the remote tag inventory and ancestry proof. The newest published tag is `v0.1.7`; commit `5130fee`, which introduced the REM commands, is later and is contained by no release tag. Therefore no published binary or crate contained the old REM exit behavior.

### 4. Final trace coverage does not match the universal guarantee

**Status:** resolved — high

The trace requirement says every final outcome record obeys the state-or-absence invariant, but the acceptance covers only success, job failure, unfamiliar state, invalid response, and pre-connection failure. Invalid usage, timeout, unknown job, local configuration or trace failure, and unclassified failure remain uncovered across the CLI and two runners.

**Would close when:** acceptance exercises all ten outcome classes at every applicable CLI and runner write boundary and includes negative controls for a missing final row, both state fields null, and both state fields non-null. Any physically unwritable trace path must be named as an explicit, separately verified exception rather than silently omitted.

**Resolution:** The universal final-outcome and state-absence guarantee was not in the released PRD and has been removed. REQ-18, design, tasks, and QCG-14 now require only the PRD-authorized unrecognised-state tuple in the CLI and `run_rem.sh` traces.

## Refuted finding

### The authoritative state vocabulary might contain a fifth normal state

**Status:** refuted — not open

The owner dumped the sidecar's own state declaration. Its vocabulary is exactly `queued`, `running`, `done`, and `failed`; the job store is typed from the same source, and no fifth state is emitted anywhere. Therefore the proposed mode-independent handling of any other non-empty value as version mismatch does not misclassify a current normal sidecar state.

No closure work is required for this finding. It must not be carried forward as open unless new source evidence introduces a fifth emitted state.
