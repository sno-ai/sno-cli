# Real staged installer acceptance

Status: QCG-2 passed through the canonical verifier. QCG-9 remains false. This is local candidate acceptance on **gpt1**, repository **/home/lh/code/sno-cli**, not publication or live harness acceptance.

## Identity and actual effects

Reach source commit: `5eca401ea26763050c6abcc82cc68f74dc8ad48f`.
Frozen Reach archive SHA-256: `dc4f84e6b15ebabbd767a6eccdd0d87186d7ae636142aa0e20540fd727102939`.
Skills source commit: `8474baf96195bbf262adc261bb2c51cd74d7a631`.
Skills archive SHA-256: `857e2f29f11bcce208823adc0b1a6bac33db417e09581e59069d898ec4b94627`.
Whole staged artifact identity: `9889490cf88d8576a50efcd8ce1709da100eb2a23a8e8333398c2569d3eb3a36`.
Shared fixture SHA-256: `17eb722714548f71492d89d40ba09764bd2a24053c1c67f6ef70039dc3785d69`.
Shared contract SHA-256: `5c29a218bd7c3d43003fad6f0e3939914a3f82ede18eb60f2e5f31a3b9a11b92`.

[Reach provenance](installer-reach-candidate-identity.json), [staged identities](installer-staged-identity.json), and [the full final verifier output](installer-real-staged-verifier.json) retain all archive paths, utility hashes, gate source hash/commit, exact commands, environments and observed results. The gate file hash is checked again after the run. The original inputs remain unchanged.

On gpt1 the final verifier passed **157 checks**, with zero failed checks. These are explicit assertions, not 157 independent test cases. It invokes the existing real CLI/installer driver against pinned real local archives. Thirty skill files and promotion stamps are compared across one shared Claude/Codex destination and custom Hermes/OpenClaw destinations. Configured directories prove installer discovery and placement only; no live harness loading is inferred.

| QCG-2 obligation | Final proof |
|---|---|
| Same real declarations, fixture and contract | All four original gate validations pass; actual installer records exact fixture/contract digests. Archive and per-unit identities match the handoff. |
| Good/bad declaration agreement | Ten copied invalid cases check field-specific gate and installer refusal: unknown program/slot, invalid need/version, wrong version/list type, duplicate entry/key, missing requires and invalid overlay. Valid overlay passes both and its actual bytes are installed. |
| Per-program numeric and dependency rules | Missing actual heartbeat and too-high copied requirements produce the named missing/too-old skip while Reach remains current. Quota tests each declared program independently. Equivalent numeric forms and unsatisfied numeric minima are checked. No fake program version/archive is presented as real. |
| Required/preferred/optional capabilities | Explicit copied reader declarations skip/degrade/install as required. Original reader-dependent utility skills skip. Hermes pre-turn support remains supported; custom OpenClaw provides the distinct unsupported preferred case. |
| Nested placement, stamps and roots | Actual nested skill contents and stamps match resolved destinations. Claude/Codex aliases produce one shared consumer destination. Hermes/OpenClaw custom roots are used; default roots remain absent. |
| Actual programs, doctor and update | The Reach command is bound to its owned isolated release and executed by absolute path. Both actual utility help commands succeed. Each utility has an ok doctor row before its link is removed, a fail row afterward, and a same-release update restores its owned link and executable help. This proves changed owned targets, not a second release version. |
| Refusal before placement | Invalid cases compare the entire isolated HOME tree, including directories, bytes, modes and symlink targets. Only the exact empty regular installer lock file is excluded. |

## Review and canonical receipt

The [one authoritative review](installer-real-staged-review.md) found two consequential test gaps: host PATH could satisfy a missing Reach entry, and negative assertions checked too few paths. Both were admitted and repaired as described above. The original report is preserved. The final verifier also removes an unused gate import and verifies the gate source remains unchanged. No second review or independent clean verdict is claimed.

The test owner ran the corrected verifier, then the parent ran the same command once through prd-proof. [Canonical receipt](../sidecar/sno-assemble-PRD.QCG-2.proof.json), [canonical run log](installer-real-staged-canonical.log), and [acceptance sidecar](../sidecar/sno-assemble-PRD.acceptance.json) record the actual successful command. Only the verifier tool changed the acceptance row.

```sh
python3 /home/lh/.agents/skills/prd-creator/scripts/prd-proof.py ai-docs/PRD/installer/sno-assemble-PRD.md QCG-2 -- python3 tests/fixtures/assemble/verify-real-staged.py --inputs ai-docs/PRD/installer/reports/installer-real-staged-positive.json --gate /home/lh/code/sno-station-skills/scripts/validate-requirements.py --output ai-docs/PRD/installer/reports/installer-real-staged-verifier.json
```

## Remaining boundary

Nine of ten acceptance rows now pass. QCG-9 still requires actual published releases and the named live seat, discovery, trust and prompt-injection journey. The candidate is Linux x86-64 only. Utility runtime copies remain in the producer's candidate skill payloads; final text-only publication is not proved. Reader-to-agent delivery is still unverified. No promotion, push, platform port, native scheduler trigger or live session was performed by this continuation.

Only the new test-owned verifier, its evidence and the canonical acceptance row changed. Production source remains unchanged. Historical negative-only and missing-input receipts are preserved as earlier observations, superseded for current readiness by this report.
