# Third Reach candidate consumer acceptance

QCG-2 passed again on **gpt1** using source `5340e2a11b80f68b33066fade7196d9868e0e16f` and the frozen archive at `/home/lh/code/sno-station-core/apps/reach/dist/candidates/5340e2a11b80f68b33066fade7196d9868e0e16f/reach-2.0.tar.gz`.
Archive SHA-256: `f2b95f4574f503d2b6802d990210334eabf2675bd201ac2c4220f13ec9e84e1f`.

The final canonical run passed **157 explicit checks**, zero failures. All 22 isolated HOME paths were newly created and differ from both earlier candidate runs; the temporary directories were removed after execution. Example canonical HOME: `/mnt/ramdisk/tmp/sno-real-staged-verifier-wfflhqwh/duplicate-entry/home`. No previous installation was overlaid. The skills, utilities, contract and fixture remain the previously pinned real inputs.

## Evidence

- [New input identities](installer-real-staged-inputs-5340e2a11.json)
- [Independent archive verification](installer-reach-candidate-5340e2a11-identity.json)
- [Full verifier output](installer-real-staged-verifier-5340e2a11.json)
- [Canonical run log](installer-real-staged-canonical-5340e2a11.log)
- [Current canonical proof](../sidecar/sno-assemble-PRD.QCG-2.proof.json)

The same existing verifier checked actual installed commands, real file/stamp mapping, gate/installer declaration agreement, numeric and capability rules, shared/custom roots, whole-tree refusal snapshots, both utility doctor failures and same-release owned-link restoration. The test owner ran it first; the parent then ran it through prd-proof. The verifier changed only its two fixed Reach source/hash literals. No product edits, new review or unrelated test runs occurred.

```sh
python3 /home/lh/.agents/skills/prd-creator/scripts/prd-proof.py ai-docs/PRD/installer/sno-assemble-PRD.md QCG-2 -- python3 tests/fixtures/assemble/verify-real-staged.py --inputs ai-docs/PRD/installer/reports/installer-real-staged-inputs-5340e2a11.json --gate /home/lh/code/sno-station-skills/scripts/validate-requirements.py --output ai-docs/PRD/installer/reports/installer-real-staged-verifier-5340e2a11.json
```

## History and boundary

The previous canonical proof is preserved in [the 75fa8d21a archive](installer-QCG-2-proof-75fa8d21a.json), with [its acceptance snapshot](installer-acceptance-75fa8d21a.json), [full previous run](installer-real-staged-verifier-75fa8d21a.json), and [previous close](installer-real-staged-close-75fa8d21a.md). The first candidate records also remain unchanged. Earlier passes are historical evidence, not inherited acceptance of these new bytes.

Nine of ten installer rows pass. QCG-9 remains false; the terminal gate still refuses final completion. This is a local Linux x86-64 candidate only. It does not prove the native bootstrap/idle repair, live harness loading/trust, reader delivery, prompt injection, published releases or final text-only runtime-copy cleanup. Those boundaries remain separate. Same-release repair is not version-upgrade proof. No promotion, source overlay, publication or push occurred.
