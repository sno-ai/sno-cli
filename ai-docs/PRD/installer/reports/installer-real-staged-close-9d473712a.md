# Fourth Reach candidate consumer acceptance

QCG-2 passed again on **gpt1** against source `9d473712a8d6d7ce1f6e931e1d445f7d6392012e`.
Frozen archive: `/home/lh/code/sno-station-core/apps/reach/dist/candidates/9d473712a8d6d7ce1f6e931e1d445f7d6392012e/reach-2.0.tar.gz`.
SHA-256: `00c882584fc05519a0b7aa2ceb2f8d2b16a4a5eb6bcfb72e75af1c1864a5bc22`.

The same verifier passed **157 explicit checks**, zero failures, first under the test owner and then through the parent's canonical prd-proof run. The final run used 22 newly created HOME directories, disjoint from all three earlier candidate runs. Example: `/mnt/ramdisk/tmp/sno-real-staged-verifier-9eo_i3rz/duplicate-entry/home`. All temporary homes were removed on exit. No old installation was overlaid.

## Current evidence

- [Pinned inputs](installer-real-staged-inputs-9d473712a.json)
- [Archive/checksum/source verification](installer-reach-candidate-9d473712a-identity.json)
- [Complete verifier output](installer-real-staged-verifier-9d473712a.json)
- [Canonical run log](installer-real-staged-canonical-9d473712a.log)
- [Current canonical proof](../sidecar/sno-assemble-PRD.QCG-2.proof.json)

The verified behavior is unchanged: actual installed commands, real text/stamp mapping, gate/parser declaration parity, whole-tree rejection snapshots, numeric/capability rules, shared/custom roots and both utility doctor/owned-link repair paths. The skills/utility/fixture/contract inputs remain unchanged. Test code changed only two fixed Reach source/hash literals. No product edit, redesign, new review or unrelated suite ran.

```sh
python3 /home/lh/.agents/skills/prd-creator/scripts/prd-proof.py ai-docs/PRD/installer/sno-assemble-PRD.md QCG-2 -- python3 tests/fixtures/assemble/verify-real-staged.py --inputs ai-docs/PRD/installer/reports/installer-real-staged-inputs-9d473712a.json --gate /home/lh/code/sno-station-skills/scripts/validate-requirements.py --output ai-docs/PRD/installer/reports/installer-real-staged-verifier-9d473712a.json
```

## History and limits

The third candidate's [canonical proof](installer-QCG-2-proof-5340e2a11.json), [acceptance snapshot](installer-acceptance-5340e2a11.json), [full run](installer-real-staged-verifier-5340e2a11.json) and [close](installer-real-staged-close-5340e2a11.md) remain historical. The first two candidates' evidence is also preserved. No prior pass was inherited for new bytes.

Nine of ten installer acceptance rows pass. QCG-9 stays false and terminal done is refused. This is a local Linux x86-64 candidate consumer result, not proof of idle tmux submission, native bootstrap, live loading/trust, reader delivery, prompt injection, published releases or final text-only runtime-copy cleanup. Same-release owned-link repair is not version-upgrade proof. No promotion, source overlay, publication or push occurred.
