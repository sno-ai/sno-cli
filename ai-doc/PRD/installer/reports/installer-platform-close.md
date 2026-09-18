# Platform-qualified Reach archive selection

Status: requested platform patch completed and affected local proof passed on **gpt1**, repository **/home/lh/code/sno-cli**. Final tier delivery remains blocked on QCG-9 publication/live acceptance and delivery-time freshness of unchanged historical receipts.

## Behavior and contract

PRD version 1.5 patches DEC-3 and QCG-1, adds the dated 2026-09-14 note and updates only those walk inputs. Reach selects `reach-<VERSION>-<os>-<arch>.tar.gz` plus that exact filename followed by `.sha256`, using literal `std::env::consts::OS` and `ARCH` values. Supported pairs are linux-x86_64, linux-aarch64, macos-aarch64 and macos-x86_64. WSL needs no separate path. There is no mapping table or fallback to unqualified Reach archives. Heartbeat and quota archive names remain unchanged.

Unsupported architecture on Linux/macOS returns usage exit 2 naming the pair before any release request. Unsupported OS returns the existing Linux/macOS refusal. The original Windows guard in src/manifest.rs is byte-identical. The actual network implementation remains unchanged; a private fetch callback makes pre-download refusal observable in tests without contacting a service.

This implements a new owner contract. The previous unqualified naming was the prior released requirement; no pre-edit defect RED is claimed. No head seal or intent was rewritten. [Scope evidence](installer-platform-scope.json) verifies both sealed head files and the Windows guard are unchanged and excludes openspec changes.

## Affected proof

On gpt1 the final canonical command ran five platform tests and four existing QCG-1 installation/integrity/recovery cases, all passing with no ignored case. [The canonical QCG-1 receipt](../sidecar/sno-assemble-PRD.QCG-1.proof.json) contains both commands and complete test output; [the recorder log](installer-platform-canonical.log) records its successful update. [Previous QCG-1 proof](installer-QCG-1-before-platform.json) and [prior acceptance snapshot](installer-acceptance-before-platform.json) are retained.

The tests cover all four literal suffixes, the build host's archive name and parsed numeric version, rejection of other-platform and old unqualified names, unchanged utility names, exact checksum matching without decoy fallback, and fake unsupported pairs. A counted external fetch substitute would return valid JSON if called, but its count must remain zero on refusal. Native Windows CI follows a tested refusal branch instead of unwrapping a supported-host result. Native Windows/macOS execution was not run here; [compiler target cfg output](installer-platform-targets.json) only confirms the literal identifiers.

Formatting, clippy --lib with warnings denied, release build and whitespace checks passed. Logs: [format](installer-platform-format.log), [clippy](installer-platform-clippy.log), [release build](installer-platform-release-build.log). Regular PRD lint is [clean](installer-platform-prd-lint.log). No full repository test suite ran. No real remote release retrieval or cross-platform runtime acceptance is inferred from helper tests.

## Bounded review

The [source review](installer-platform-code-review.md) approved the initial platform delta. The [test review](installer-platform-test-review.md) found one proof gap: a successful request before refusal could leave the original exit-code assertions green. The gap was admitted and fixed with the private external-fetch boundary and zero-call assertion, then the five tests and canonical affected proof passed. The original reports remain unchanged. No second review or post-fix independent clean verdict is claimed. Before review, the active agent also repaired new tests that assumed every CI host was supported; this did not change product support policy.

## Delivery boundary

The sidecar retains nine prior passing flags, but only affected QCG-1 has been re-proved against version 1.5. Delivery mode requires every receipt to match the complete final PRD body, so it additionally rejects QCG-2/3/4/5/6/7/8/10 as historical after this narrow revision. The owner-directed mail limits this patch to affected tests and leaves other tier rows as they were; those unrelated verifiers were not rerun or resealed. [Delivery output](installer-platform-delivery-gate.log) and [done refusal](installer-platform-done-gate.log) preserve the exact result. Existing QCG-2 staged candidate evidence remains separate from this GitHub asset-name selection change; its local provider does not claim platform-qualified assets were published. QCG-9 remains false. No publication, push, worktree, live session or unrelated source edit occurred. [The current terminal report](installer-terminal-close.md) uses the new PRD body hash and retains the blocked delivery state.
