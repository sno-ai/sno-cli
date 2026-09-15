# Staged consumer partial receipt

On gpt1 in /home/lh/code/sno-cli, the existing test driver was rebuilt against CLI commit f06908b400bbd1f2ebc190128010c6367153111b. It invokes the actual public CLI parser and installer; only release resolution and retrieval use local files. No source or test code changed.

[Identity evidence](installer-staged-identity.json) verifies the supplied archive, complete gate artifact identity, fixture, contract, all four SKILL.md files and all four pre-stamp unit payloads. The actual runs and exact driver/library hashes, commands, isolated environments, artifacts and observed filesystem state are retained in [the machine receipt](installer-staged-consumer.json). Compilation output is retained separately, including the initial test-harness dependency-search-path error and its correction.

| Input | Actual result | Observed installation state |
|---|---|---|
| Exact real staged skills archive plus real local heartbeat/quota 1.0 archives; no Reach archive | Exit 3: selected release has no Reach program | No manifest, pending transaction, or installed program roots |
| Disposable copy with the reach skill hash changed in the shared fixture; same utilities | Exit 3: fixture skill hash/declaration mismatch: reach | No manifest, pending transaction, or installed program roots |

The original staged archive, fixture and contract hashes remained unchanged. The full artifact hash is 9889490cf88d8576a50efcd8ce1709da100eb2a23a8e8333398c2569d3eb3a36 and the fixture hash is 17eb722714548f71492d89d40ba09764bd2a24053c1c67f6ef70039dc3785d69.

This proves actual validation and refusal against the gate-produced bytes. It does not prove positive installation, all declaration variants, skill mapping, utility execution/update or the complete staged-consumer criterion. QCG-2 remains false/null; no success receipt was fabricated. A real Reach archive is required next. QCG-9 still requires published releases and live acceptance. Candidate skill payloads retain utility runtime copies disclosed by their producer and are not final text-only publication acceptance. No promotion, push or upstream write occurred.
