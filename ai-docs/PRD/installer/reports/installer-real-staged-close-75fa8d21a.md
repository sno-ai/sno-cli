# Replacement Reach candidate consumer acceptance

QCG-2 passed again on **gpt1** against the replacement frozen archive. The parent canonical run passed **157 explicit checks**, with zero failed checks, using fresh isolated HOME directories. No installed tree from the previous candidate was reused. QCG-9 remains false; no publication/live acceptance is claimed.

- Source commit: `75fa8d21a76909ce3ebdadfa45df31e3feef6adb`.
- Frozen archive: `/home/lh/code/sno-station-core/apps/reach/dist/candidates/75fa8d21a76909ce3ebdadfa45df31e3feef6adb/reach-2.0.tar.gz`.
- Archive SHA-256: `47edd32a93366bfac2d650c261869599e8f8730fc5aab5d9ddfaf918d63a21b5`.
- Version: 2.0. Platform: local Linux x86-64 candidate only.
- Skills/utility artifacts, shared fixture and contract remain the same pinned bytes.

[Input/provenance](installer-real-staged-inputs-75fa8d21a.json), [independent archive verification](installer-reach-candidate-75fa8d21a-identity.json), [complete verifier output](installer-real-staged-verifier-75fa8d21a.json), [canonical log](installer-real-staged-canonical-75fa8d21a.log), and [current canonical proof](../sidecar/sno-assemble-PRD.QCG-2.proof.json) bind this run to the new digest. The final run's first HOME was `/mnt/ramdisk/tmp/sno-real-staged-verifier-mrmg98yl/duplicate-entry/home`; all `22` test homes were new and the temporary parent was removed on exit.

The same verified checks exercised actual installed program commands, gate/installer declaration agreement, numeric and capability rules, whole-tree rejection snapshots, shared/custom roots, real file/stamp mapping, both utility doctor failures, and same-release owned-link repair. All original archive and gate hashes remained unchanged. The verifier code changed only its Reach source-commit and digest literals. No product code changed. No second review was run; the prior reviewed assertions and their fixes were reused unchanged.

The previous candidate remains historical: [archived prior canonical proof](installer-QCG-2-proof-5eca401ea.json), [prior acceptance snapshot](installer-acceptance-5eca401ea.json), [prior verifier output](installer-real-staged-verifier.json), and [prior close description](installer-real-staged-close.md). The current canonical proof supersedes the previous one; the old successful run was not inherited by this candidate.

Canonical command:

```sh
python3 /home/lh/.agents/skills/prd-creator/scripts/prd-proof.py ai-docs/PRD/installer/sno-assemble-PRD.md QCG-2 -- python3 tests/fixtures/assemble/verify-real-staged.py --inputs ai-docs/PRD/installer/reports/installer-real-staged-inputs-75fa8d21a.json --gate /home/lh/code/sno-station-skills/scripts/validate-requirements.py --output ai-docs/PRD/installer/reports/installer-real-staged-verifier-75fa8d21a.json
```

Nine of ten acceptance rows pass. Actual published releases, live seat/harness loading and trust, reader delivery, prompt injection and final text-only publication remain outside this result. Candidate runtime copies remain disclosed. Same-release repair is not a version-upgrade proof. No promotion or push occurred.
