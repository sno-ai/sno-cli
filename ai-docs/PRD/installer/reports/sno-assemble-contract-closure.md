# Installer cross-repository contract closure

Current PRD: `../sno-assemble-PRD.md`, version 1.3.
SHA-256: `a560dfeb6bc523a3d416b6ec41366bdf08f2a8939b463ef35175d8dfe56bc1d2`.
Authority: owner-authorised document repairs; mailbox closure request `<closure-sno-cli-1789274700@gpt1>` specifies the four agreed alignment items. Sources: S publication revision 1.4 (draft) and Reach revision 1.3 (released).

| Item | Changed rows | Closure |
|---|---|---|
| 1. Core utility artifacts | S5, F4, actor table, DEC-2/3, REQ-2/6/7/8, STP-3, QCG-2/9 | Utility source is core; each program has its own artifact name/version/checksum/layout/entry point and lifecycle ownership. No skill-contained utility runtime is expected. Existing Reach namespace remains fixed. |
| 2. One requirements contract | DEC-4, REQ-2, section 5.6, STP-3, QCG-2 | Gate and actual installer pin the same version/hash and literal IDs. Each program is checked against its own artifact. Unknown/malformed declarations fail; known missing/partial/unverified capability follows required/preferred/optional dispositions. One shared actual four-unit fixture supports S QCG-2/QCG-8. Numeric version rules and all unit dependency sets are explicit. |
| 3. Clean-account exchange | QCG-9; Direct Execution Contract User journey | Both seats start absent and are initialized through public init, then spawned/registered and checked before send. No hidden preflight identity setup. One bounded filtered wait reads terminal completion; export/state proves acceptance and the thread. Core reply-ring/acknowledgment behavior is retained. |
| 4. Two evidence stages | Section 7, QCG-2/9, section 10 fixtures, Direct Execution Contract Integration proof/Completion | Gate-produced staged bytes prove the isolated real consumer before publication; actual pinned releases prove the final journey afterward. Separate receipts and identities; neither stage substitutes for the other. |

Companions: updated the affected current walk rows; marked version-1.2 repair notes historical where their utility-source assumption was superseded. Sealed head and authoritative reviews remain unchanged.

Validation on local documents in `/home/lh/code/sno-cli`: targeted `prd-lint.py` clean; `git diff --check` exit 0; ten acceptance IDs match their sidecar and all remain false. No source or test code changes, runtime acceptance, external publication, or independent re-review was performed.

All four requested document items are complete. The actual shared contract file, core utility manifests and staged/published artifacts are still incoming implementation dependencies; this closure does not invent their versions/hashes or claim their delivery.
