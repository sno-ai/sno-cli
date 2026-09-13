> Historical version-1.2 repair record. Its skill-contained public-command source was superseded by S revision 1.4. The current installer contract is version 1.3; see `sno-assemble-contract-closure.md`.

# Installer PRD peer-review repairs

Artifact: `../sno-assemble-PRD.md`, version 1.2.
PRD SHA-256: `98fa5c9c56b06ee3432b41c9b3cdd8ce8575e12b9ae03b2bd6fe085b0646ce23`.
Scope: owner-authorised repairs to the eight findings from `sno-assemble-peer-review.md`. Documentation only. The original authoritative review is preserved unchanged.

## Finding dispositions

| Original finding | Severity | Cause and repair | Current contract and proof |
|---|---|---|---|
| Skill executable commands have no install step | Critical | The installer consumed text but omitted the upstream public-bin handoff. Both public commands now retain relative resources, executable modes and ownership through install/update/doctor/remove. | REQ-2/6/8; QCG-2/9 |
| Hermes acceptance contradicts capability evidence | Critical | A supported pre-turn capability was treated as absent. Production Hermes now uses the measured support; unsupported cases use a separate explicit fixture cell. | DEC-4; QCG-2 |
| Failed mutations leave an inaccurate manifest and cannot be removed | Critical | A final manifest write alone cannot make multiple filesystem changes atomic. A durable pending transaction, common mutation lock, rollback data and commit identity now govern recovery. Removal and explicit state purge have distinct commit boundaries. | REQ-5/8/9; QCG-5/8 |
| Existing skills can be overwritten and deleted as owned directories | Critical | Path presence was confused with ownership. Unowned collisions refuse; owned changed content is preserved; deletion is per recorded file/link/entry and only empty owned directories are removed. | DEC-7; REQ-8; QCG-8 |
| The fallback build cannot run when upstream is absent | High | A released upstream PRD was confused with available implementation. Synthetic fixtures permit local development only; real release assets, hashes and layouts gate production acceptance. Missing artifacts are an explicit delivery dependency. | DEC-3; STP-2/7; section 7; execution stop conditions |
| Fixed skill paths may not be read by active harnesses | High | Directory existence does not establish the selected instance’s discovery roots. Resolve invocation-selected instances, canonicalise aliases, evaluate shared consumers and prove actual discovery, including custom OpenClaw roots. | S3; DEC-4; REQ-2; QCG-2/9 |
| Hook entry presence does not prove reminder execution | High | Config, trust, seat identity and live context injection were conflated. They are now separate checks; missing address is an explicit no-op, addressed errors stay observable, and actual prompts prove injection. | DEC-5; REQ-4/7; QCG-4/9 |
| Six all-green sections do not follow from installation | High | Optional timer state and existing station warnings were hidden by an overstrong acceptance claim. Disabled, missing, warn and fail remain distinct; station behavior is preserved. The Reach journey checks Reach readiness rather than requiring unrelated telemetry initialization. | DEC-8; REQ-7; QCG-6/7/9 |

All eight findings are addressed in the document. This is not a claim that their product tests passed.

## Companion consistency

- Updated the current walk: corrected Hermes, recovery, ownership, public commands, target discovery, hook trust, release readiness and doctor outcomes.
- Marked the old version-1.1 owner report historical with a pointer here, so its rejected claims cannot be reused as current guidance.
- Preserved the sealed head and original review as historical records.
- Kept all ten acceptance flags false. No product implementation, runtime acceptance or release is claimed.
- Included narrowly required module declarations, dependency lock resolution and proof output paths in the future implementation boundary; this revision changed no source files.

## Verification

Host: `gpt1`. Evidence boundary: local documentation in `/home/lh/code/sno-cli`.

- `prd-lint.py`: `prd-lint: clean` (exit 0).
- `git diff --check`: exit 0.
- Identifier check: 11 unique requirement definitions; 10 unique acceptance definitions; exact acceptance-sidecar correspondence; all acceptance flags false.
- Changed tracked paths are confined to `ai-docs/PRD/installer/`.
- No product tests, live harness calls, remote publication or deployment were performed.

The peer-review document workflow does not run a confirmation-review loop. Closure here uses direct repair, the affected contract walk and deterministic validation; no second independent approval is claimed.

## Remaining delivery prerequisites

Implementation still has to obtain actual upstream artifacts and run the specified fixture, scheduler and live-harness checks. Their current absence does not leave an undefined document instruction: section 7 and the stop conditions define the dependency and forbid substituting synthetic evidence. Project status remains `not_started`.
