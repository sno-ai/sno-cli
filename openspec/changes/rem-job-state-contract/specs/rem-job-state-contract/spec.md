## ADDED Requirements

### Requirement: REQ-1 Single outcome declaration
The tool SHALL carry one declaration listing every outcome class reachable by the `station rem-*` family. Each entry SHALL contain exactly one class name, exactly one process exit code, and an `error_codes` list, and raise and exit sites SHALL resolve both machine-readable error codes and process exits through that declaration. Success and unclassified failure SHALL have empty `error_codes` lists, invalid usage SHALL contain exactly `usage_error`, and every other named failure class SHALL contain at least one error code.

#### Scenario: Raise and exit sites derive both codes
- **WHEN** a REM operation resolves a known failure
- **THEN** its error code belongs to exactly one declaration entry and its process exit code is obtained from that entry

### Requirement: REQ-2 Unique outcome identifiers
No two entries SHALL share a class name or process exit code, no machine-readable error code SHALL occur in more than one entry's `error_codes` list, and the tool's test suite SHALL fail each kind of collision independently.

#### Scenario: Duplicate class identity is rejected
- **WHEN** two declaration entries share a class name or process exit code
- **THEN** the class-identity test fails before the build can ship

#### Scenario: Duplicate error membership is rejected
- **WHEN** one machine-readable error code appears in two entries' `error_codes` lists
- **THEN** the tool's test suite fails before the build can ship

### Requirement: REQ-3 Documentation matches the declaration
`README.md` SHALL document the `station rem-*` exit codes in a table generated from or checked against the outcome declaration.

#### Scenario: Documentation drift is rejected
- **WHEN** the README exit-code table differs from the declaration
- **THEN** a repository test fails

### Requirement: REQ-4 Family-wide classification with non-REM stability
The outcome declaration SHALL govern every command in the `sno station rem-*` family. Commands outside that family SHALL retain their existing exit behavior, and a generic runtime error without a named REM outcome SHALL continue to exit `1`.

#### Scenario: Shared sidecar failure is classified only for REM commands
- **WHEN** `rem-start` and `rem-status` encounter a stopped sidecar and unrelated commands encounter generic runtime failures
- **THEN** both REM commands exit `7` and the unrelated commands continue to exit `1`

### Requirement: REQ-5 Ten exact outcome classes
The REM command family SHALL use exactly these outcome classes and exit codes: success `0`, unclassified failure `1`, invalid usage `2`, job failed `3`, wait deadline passed `4`, state vocabulary mismatch `5`, malformed or truncated response `6`, unreachable, undiscoverable, unauthenticated, or client-failed sidecar `7`, local environment, profile, or trace failure `8`, and unknown job identifier `9`.

#### Scenario: Each observable outcome has its assigned exit
- **WHEN** each of the ten REM outcomes is produced independently
- **THEN** the command exits with that outcome's assigned code without interchange

### Requirement: REQ-6 Every raisable REM error is mapped
Every machine-readable error code raisable by a REM path SHALL map to exactly one declared outcome class, and the test suite SHALL fail when a raisable code is unmapped.

#### Scenario: New unmapped raise site fails the build
- **WHEN** a REM path gains a raisable error code absent from the declaration
- **THEN** the exhaustive mapping test fails

### Requirement: REQ-7 Unclassified has no mapped member
No machine-readable error code SHALL map to the unclassified outcome class; that class SHALL be reached only when a code is absent from the map.

#### Scenario: Explicit unclassified mapping is rejected
- **WHEN** a declaration entry maps an error code to the unclassified class
- **THEN** the tool's test suite fails

### Requirement: REQ-8 Exit one remains the fallback
Exit code `1` SHALL remain reserved for unclassified failure and SHALL NOT be assigned to a named error-code mapping.

#### Scenario: Unknown failure uses the reserved fallback
- **WHEN** the REM command receives a failure for which no outcome mapping exists
- **THEN** it exits `1` and no named outcome class claims that code

### Requirement: REQ-9 Non-waiting active jobs succeed
A non-waiting `rem-status` call SHALL print and exit `0` for `queued`, `running`, or `done`; preserve the sidecar error and exit `3` for `failed`; and print the raw state and exit `5` for every other non-empty state.

#### Scenario: Active job poll is not a failure
- **WHEN** `rem-status` without waiting observes a queued or running job
- **THEN** stdout contains the observed state and the command exits `0`

#### Scenario: Non-waiting unfamiliar state fails closed
- **WHEN** `rem-status` without waiting observes a non-empty state other than queued, running, done, or failed
- **THEN** stdout contains the raw state, the tool emits `rem_state_unrecognised`, and the command exits `5`

### Requirement: REQ-10 Unrecognised state is printed before the error
An unrecognised-state error SHALL retain the raw state string, and `rem-status` SHALL print that state to stdout before emitting the error in both waiting and non-waiting modes, including when stdout is captured by command substitution.

#### Scenario: Captured stdout retains the raw state
- **WHEN** a caller captures stdout while a status call encounters an unfamiliar non-empty state
- **THEN** the raw state is present in captured stdout before the error is emitted

### Requirement: REQ-11 Version skew has a dedicated code pair
A well-formed response with any unfamiliar non-empty state SHALL raise `rem_state_unrecognised` and exit `5` immediately in both waiting and non-waiting modes; the sidecar protocol exposes no independent terminal-state field, and that error and exit pair SHALL be used for no other outcome.

#### Scenario: Waiting unfamiliar state is classified
- **WHEN** a waiting status call receives a valid job record with an unfamiliar non-empty state
- **THEN** the tool emits `rem_state_unrecognised` and exits `5` without another poll

#### Scenario: Non-waiting unfamiliar state is classified
- **WHEN** a non-waiting status call receives a valid job record with an unfamiliar non-empty state
- **THEN** the tool emits `rem_state_unrecognised` and exits `5`

### Requirement: REQ-12 Version-skew message identifies the mismatch
The unrecognised-state message SHALL contain the job identifier, the state string byte-for-byte, and a sentence stating that the sidecar reported a state this build of the tool does not know.

#### Scenario: Operator can diagnose component skew from one message
- **WHEN** an unfamiliar non-empty state is reported
- **THEN** the emitted message identifies the job, preserves the state, and explains the state-vocabulary mismatch

### Requirement: REQ-13 Invalid response is narrowly defined
`sidecar_response_invalid` SHALL be raised only when a response cannot be parsed into a job record or when its state field is absent or empty, in either wait mode. It SHALL NOT be raised for a well-formed response with an unfamiliar non-empty state.

#### Scenario: Malformed and unfamiliar responses remain distinct
- **WHEN** invalid JSON, an empty state, and a well-formed unfamiliar state are evaluated
- **THEN** only invalid JSON and the empty state produce `sidecar_response_invalid`

### Requirement: REQ-14 Failed job preserves sidecar detail
A `rem_job_failed` message SHALL include the sidecar's error string when the sidecar supplies one.

#### Scenario: Sidecar failure detail reaches the caller
- **WHEN** the sidecar marks a job failed and supplies an error string
- **THEN** the CLI failure message contains that string

### Requirement: REQ-15 One enumerated caller routing table
Both `run_rem.sh` and `run_rem_noop.sh` SHALL apply the same enumerated exit-code routing table to `rem-start` and `rem-status`, with each listed code mapping to exactly one outcome. Routing decisions SHALL use the exit code and SHALL NOT inspect message text.

#### Scenario: Both runner calls use identical numeric routing
- **WHEN** either runner receives the same listed exit code from `rem-start` and `rem-status` with different message text
- **THEN** it applies the same routing outcome to both commands

### Requirement: REQ-16 Unknown exit codes fail closed
An exit code absent from either runner's routing table SHALL make that runner exit non-zero, fail the persona, and log the unmatched code.

#### Scenario: Future tool code cannot silently pass
- **WHEN** the tool returns an exit code not present in either runner's table
- **THEN** that runner fails the persona and its log names the unmatched code

### Requirement: REQ-17 Version skew fails with a useful diagnosis
Exit code `5` SHALL fail the persona in both runners, and the applicable runner log SHALL name the raw state and the version mismatch rather than describing the response as invalid.

#### Scenario: Unrecognised state is a loud failure
- **WHEN** the CLI exits `5` for an unfamiliar state
- **THEN** the applicable runner fails the persona and logs the state and state-vocabulary mismatch

### Requirement: REQ-18 Both traces record the routing tuple
Every final outcome record written to an available CLI or runner trace SHALL contain `raw_state`, `state_unavailable_reason`, `outcome_class`, and `exit_code`. Exactly one of `raw_state` and `state_unavailable_reason` SHALL be non-null: a component that obtained a decoded job state SHALL record it byte-for-byte, while a component without a state SHALL set `raw_state` to null and set `state_unavailable_reason` to the machine-readable error code or `job-state-not-returned` for successful `rem-start`. A trace sink that cannot be opened or written cannot record its own `rem_trace_error` and is exempt from this trace-record guarantee.

#### Scenario: Routing can be reconstructed across state availability
- **WHEN** success, job failure, unfamiliar state, invalid response, and pre-connection failure are each exercised with writable tracing
- **THEN** every CLI and runner final outcome record contains the outcome class and exit code, records the raw state when observed, and otherwise records a non-empty state-unavailable reason

### Requirement: REQ-19 JSON transport remains separate from routing
Both runners SHALL continue passing `--json` wherever each already does and SHALL NOT parse human-readable message text for a routing decision.

#### Scenario: Message replacement does not alter routing
- **WHEN** the CLI returns the same exit code with unrelated message text to either runner
- **THEN** that runner makes the identical decision and retains its existing `--json` invocation

### Requirement: REQ-20 Runner-owned exits are disjoint
Both `run_rem.sh` and `run_rem_noop.sh` SHALL exit `20` for their own usage error and `21` for their own rejected operation type. Neither SHALL originate any code from `0` through `9`; such codes SHALL only be propagated from the tool.

#### Scenario: Runner failures cannot be confused with tool outcomes
- **WHEN** either runner is invoked without an argument or with an operation type it rejects
- **THEN** it exits `20` or `21` respectively, while no runner-owned path emits `0` through `9`

### Requirement: REQ-21 Operation names and validation exits have separate owners
This change SHALL NOT alter which operation names `run_rem.sh` accepts or its unknown-operation message; those belong to `rem-operation-switches`. This change SHALL alter the rejected-operation exit statement in that same validation block from `2` to `21`. The sibling's name and message edits and this change's exit-number edit SHALL remain independently landable in either order.

#### Scenario: Either change can land first
- **WHEN** only this change is present or only the sibling operation-name change is present
- **THEN** the accepted-name set and message come from the present name owner, the rejected-operation exit comes from the present exit owner, and the relevant verification passes against the same store
