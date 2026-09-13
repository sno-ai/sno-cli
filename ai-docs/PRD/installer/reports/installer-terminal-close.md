# Installer local execution report

PRD body SHA-256: 95cf60b9da9a29aa6a4b9208d44a3d8c273752b87b99bccd00994e1b8390cf99
Verification: local fixture verification passed; external acceptance incomplete
Acceptance: blocked
Required work remaining: 2 external acceptance rows
Owner approval: execution authorized; final delivery not approved

## Result and scope

The installer implements assemble, update, doctor and remove with release verification, strict shared requirements parsing, ownership-preserving transactions and crash recovery, reminder configuration and bounded execution, scheduler ownership, and explicit state purge. Existing station diagnostics remain available.

Execution ran on **gpt1**, repository **/home/lh/code/sno-cli**, branch **feat/assemble-installer**. The captured baseline commit is **e65b04d840d3009f1f15af9972dd6fb22395f578**. [The final scope record](installer-final-scope.json) compares file hashes against that baseline, including untracked files. No out-of-scope file changed and openspec is unchanged. The PRD was narrowly revised to version 1.4 to permit removal of the obsolete root-doctor retirement assertion; other legacy assertions remain. Dependencies and README changes are restricted to the named surfaces.

## Verification evidence

On gpt1 in this repository, [the final installer test run](installer-fixture-tests.log) passed 38 tests with zero failures and one explicitly ignored external-data test. One passing entry is the subprocess helper, leaving 37 default behavioral cases. The ignored mixed-artifact case was then run explicitly and passed; [the direct rerun log](installer-mixed-utility-direct.log) records actual local heartbeat and quota archive paths and hashes. That case uses synthetic Reach and skill payloads and does not prove a published release.

The two affected legacy CLI cases passed. Formatting, clippy with warnings denied, release build, release-binary help, and source/test diff whitespace checks passed. The full staged diff reported whitespace in verbatim review context, review output and raw test logs; those evidence bytes are retained, so an all-files whitespace pass is not claimed. Logs: [format](installer-format.log), [clippy](installer-clippy.log), [build](installer-release-build.log), [help](installer-release-help.txt). No whole-repository test suite ran. No remote CI or published deployment is claimed.

Canonical prd-proof receipts record nonempty passing verifiers for QCG-1/3/4/5/6/7/8/10. [The acceptance sidecar](../sidecar/sno-assemble-PRD.acceptance.json) links each exact command and receipt. Tests cover hostile archives, ownership conflicts, rollback, restrictive permissions, interrupted committed removal, large payload recovery, hook errors and child termination, read-only trust, doctor failures, timer recovery and actual execution of the generated scheduler command under its declared environment. Native scheduler triggering is not proved.

[Review dispositions](installer-review-closure.md) retain the authoritative original reports and document the three admitted source repairs and three test repairs. One source finding conflicts with the settled named-skip contract and remains Possible FP. No repeated review loop or clean second verdict is claimed.

## Remaining external evidence

- **QCG-2 remains false/null.** The real four-unit gate-produced `scripts/fixtures/s-category-requirements.json` is absent in /home/lh/code/sno-station-skills at this final check. Supply that fixture and its matching staged skill bytes, including promotion stamps, so the real gate and installer can prove the same declarations and hashes. The exact shared contract is available and verified at SHA-256 `5c29a218bd7c3d43003fad6f0e3939914a3f82ede18eb60f2e5f31a3b9a11b92`; synthetic fixtures cannot replace staged proof.
- **QCG-9 remains false/null.** Actual pinned published core and skills releases and the specified real harness/seat exchange, discovery, trust, prompt injection, utility execution and causal wrong-version journey remain unverified. [The readiness check](upstream-readiness.json) records unauthenticated GitHub API 404 responses; those responses do not establish whether releases exist. Local utility archives only prove the explicitly mixed-artifact test.

The delivery lint rejects exactly those two nonpassing rows; see [gate output](installer-delivery-gate.log). The done command is also attempted with this truthful blocked report and its refusal is retained in installer-done-gate.log. The project is not marked done. This is the fixture-reachable execution result requested in the dispatch, not final acceptance or publication.
