---
name: "sno-assemble"
title: "sno assemble: install, update, check and remove the station"
version: "1.1"
prd_status: "released"
project_status: "not_started"
updated: "2026-09-13"
owner: "larry"
pipeline: "small"
---

# PRD — `sno assemble`: install, update, check and remove the station with one CLI

Four top-level verbs in the `sno` CLI put a machine on the team and keep it current:
`sno assemble` installs the Reach program and the shipped skill text into every agent harness
present on this machine; `sno update` brings both to the latest published version; `sno doctor`
checks everything installed (and absorbs today's `sno station doctor`); `sno remove` takes it
all out. The inputs are published contracts from two other repositories: the Reach naming
contract and `apps/reach/VERSION` (`sno-station-core`, PRD `ai-doc/ACTIVE/PRD/reach/reach-PRD.md`,
released 2026-09-13) and each shipped skill's `PROMOTED.json` plus `requires:` block
(`sno-station-skills`, PRD `ai-doc/ACTIVE/PRD/cat-S/s-category-publish-PRD.md`, released 2026-09-13).

## 1. Owner rulings and settled decisions
<!-- prd:owner-rulings -->

| # | Date | Decider | Ruling |
|---|---|---|---|
| 1 | 2026-09-12 | larry | Every install and update goes through the `sno` CLI, one tool for all of it ("千军万马汇成一个"). |
| 2 | 2026-09-10 | larry | Onboarding is the `sno assemble` command; a skill's requirements declaration is what `sno assemble` reads to decide install / degrade / skip. |
| 3 | 2026-09-13 | larry | Four top-level verbs: `sno assemble`, `sno update`, `sno remove`, `sno doctor`; `sno doctor` absorbs `sno station doctor` as one section. ("2".) |
| 4 | 2026-09-12 | larry | Public names are the Reach naming contract; the installer writes `~/.config/sno-reach/agents.json`, installs to `~/.local/lib/sno-reach/`, and puts only `sno-reach` on PATH. |
| 5 | 2026-09-10 | larry | Two public homes only; the installer reads from `sno-station-core` releases and `sno-station-skills`, nothing else. |
| 6 | 2026-09-13 | larry | `sno starport` is dead and is removed in this PRD; no CLI verb will ever map to it. ("砍了。") |

Decisions under standing authority:

| # | Date | Decision | Why no escalation |
|---|---|---|---|
| S1 | 2026-09-13 | `sno update` is `sno assemble` re-run against the latest published versions; the two share one code path and differ only in whether an already-current piece is reported as `current` or `updated`. | One path, no drift between install and update. |
| S2 | 2026-09-13 | Automatic update is a user-level scheduled timer that runs `sno update --quiet` once a day, switched by `sno update --auto on|off`; on Linux it is a `systemd --user` timer, on macOS a `launchd` agent; no daemon of our own. | The two platforms already ship a scheduler; writing one is scope creep. |
| S3 | 2026-09-13 | The harnesses recognised are Claude Code, Codex, Hermes, OpenClaw, and the shared `~/.agents/skills` directory; a harness is "present" when its skill directory exists or its executable is on PATH. Others are skipped with a `skip` line. | Matches the harness capability map's five columns minus dsh (no skill directory measured). |
| S4 | 2026-09-13 | The Codex hook trust hash cannot be written by the installer; after wiring a Codex hook the installer prints the one action the user must take (open Codex, press `t` on the new hook). | Measured 2026-09-11: Codex refuses to run an untrusted hook and the trust record is written only by its own UI. |
| S5 | 2026-09-13 | Sources are pinned release artifacts, never a git checkout at HEAD: `sno-station-core` GitHub Releases for the Reach tarball (`reach-<version>.tar.gz` + `.sha256`), `sno-station-skills` at a tagged release for skill text. | The CLI's own README already distributes by tagged releases with SHA-256; the same rule for what it installs. |

## 2. The problem, measured
<!-- prd:problem-measured -->

Measured 2026-09-12/13 on the owner's workstation and in the three repositories.

- `sno` 0.1.7 has no install verb: `sno --help` lists `account`, `station`, `starport`, and
  `starport` answers "no starport verbs are released yet" (`src/cli.rs` line 173). External
  dispatch exists and works (`src/cli.rs` 283–305; probe 2026-09-12: a stub `sno-reach` got its
  arguments and its exit code 42 came back).
- Putting the communication stack on one machine by hand on 2026-09-11 took nine manual steps
  (workshop report `ai-doc/ACTIVE/PRD/acp-v1-communication/reports/acp-v1-communication-terminal-close-2026-09-10.md`):
  install the runtime, copy the wrapper scripts into two harness skill directories, write the
  ACP agent config, write the routing config, link two ACP adapters onto PATH, merge a hook into
  two harness settings files, press `t` in Codex to trust the hook, run one probe. Two of the
  nine were discovered only when a seat failed to start.
- Per-harness copies drift: the workshop deployer reported the TPM skill `STALE` in two harness
  directories on 2026-09-11 until redeployed.
- Five harness skill directories exist on the workstation (`~/.claude/skills` 74 entries,
  `~/.codex/skills` 82, `~/.agents/skills` 82, `~/.hermes/skills` 1, `~/.openclaw/skills` 2);
  two hook files (`~/.claude/settings.json`, `~/.codex/hooks.json`) each already carry one
  progress-reminder entry written by hand.
- `ai-docs/PRD/installer/` was empty before this document.

Falsified: "cargo install and a README are enough" — the nine steps above are not cargo's.

## 3. Verified facts the executor must not re-derive
<!-- prd:verified-facts -->

| # | Fact | Source | Date |
|---|---|---|---|
| F1 | `sno` is a Rust clap CLI; root commands `account`, `station`, `starport`, plus `external_subcommand` dispatching to `sno-<name>`; `RETIRED_ROOT_COMMANDS` refuses retired names. | `src/cli.rs` 36–52, 283–305 | 2026-09-13 |
| F2 | `sno station doctor` prints badge lines for identity, buffer, consent, last ship, lockfile; `--json` gives a structured object. | `src/doctor.rs` 38–70 | 2026-09-13 |
| F3 | Reach installs at `~/.local/lib/sno-reach/releases/<version>/` with `bin/sno-reach`, `lib/`, `vendor/`, `spec/`, `guide/`, `VERSION`, `NOTICE`, `LICENSE`; `current` is the stable path; `sno reach --version` prints `VERSION`; `sno reach doctor --as` checks a seat; config `~/.config/sno-reach/agents.json` maps kind → `{"acpx_agent": name}`; `sno reach remind --as $SNO_REACH_ADDR` is the hook command. | Reach PRD REQ-1, 2, 14, 16, section 5.3 | 2026-09-13 |
| F4 | A promoted skill directory carries `PROMOTED.json` (`unit category tier split source selftests_run promoted_at`) and a `SKILL.md` whose frontmatter has `requires: {programs: [{name, min_version}], harness: [{slot, need}]}` with `need` ∈ required/preferred/optional; slot names are `<perspective>.<row-slug>` from the capability map. | S publish PRD DEC-3, F2 | 2026-09-13 |
| F5 | Claude Code reads `~/.claude/settings.json` `hooks.UserPromptSubmit[]`; Codex reads `~/.codex/hooks.json` `hooks.UserPromptSubmit[]` and records trust per hook in `~/.codex/config.toml` `[hooks.state."…:user_prompt_submit:<i>:0"] trusted_hash`; the hook command shape is `text=$(timeout 5 <cmd>) || exit 0; [ -n "$text" ] || exit 0; jq -nc --arg text "$text" '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:$text}}'`. | workshop `skills/tpm/skills/tpm/references/progress-reporting-hooks.md`; measured hook files 2026-09-11 | 2026-09-11 |
| F6 | ACP adapters are npm packages run through `npx -y @agentclientprotocol/claude-agent-acp@<v>` and `@agentclientprotocol/codex-acp@<v>`, configured in `~/.acpx/config.json` `agents.<name>.argv`; `acpx` itself is a Node CLI on PATH. | measured `~/.acpx/config.json` on the proof VM and workstation, 2026-09-11 | 2026-09-11 |
| F7 | Skill directories per harness: `~/.claude/skills/<name>/SKILL.md`, `~/.codex/skills/<name>/SKILL.md`, `~/.agents/skills/<name>/` (shared), `~/.hermes/skills/`, `~/.openclaw/skills/`. | measured 2026-09-13 | 2026-09-13 |
| F8 | The CLI ships through crates.io and GitHub Releases with SHA-256 checksums and shell/PowerShell installers. | `README.md` "Install" | 2026-09-13 |

## 4. Actors and handoff boundaries
<!-- prd:actors-handoffs -->

| Actor | Touches this how | Succeeds when |
|---|---|---|
| A person at a terminal | Runs `sno assemble` once, `sno update` (or the timer) later, `sno doctor` when something is off, `sno remove` to leave. | One command each; every line of output says what was done to which path. |
| The Reach program (from core releases) | Is downloaded, verified, unpacked into a release directory, and linked as `current`; `sno-reach` linked onto PATH. | `sno reach --version` equals the pinned version. |
| A shipped skill (from the skills repository) | Its text is copied into each present harness's skill directory when its `requires:` are met; degraded or skipped otherwise. | `PROMOTED.json` is copied beside it as the installed stamp. |
| Each harness (Claude Code, Codex, Hermes, OpenClaw) | Gets skill text, and for Claude Code and Codex the progress-reminder hook entry. | The hook entry is present exactly once; Codex's trust step is printed. |
| The scheduler (systemd user timer / launchd) | Runs `sno update --quiet` daily when `--auto on`. | `sno doctor` reports the timer active. |

Boundary: this PRD does not build Reach or the skills; it installs what those repositories
publish. It does not install ACP adapters or `acpx` (npm packages the user owns); it detects
them and tells the user what is missing.

## 5. The design
<!-- prd:design -->

### 5.1 Settled decisions

- `[DEC-1]` Four root verbs (ruling 3): `assemble`, `update`, `remove`, `doctor`. `sno station doctor` stays as an alias printing only the station section of `sno doctor`. `starport` leaves the root command list and joins `RETIRED_ROOT_COMMANDS` (ruling 6).
- `[DEC-2]` One manifest, `~/.config/sno/assemble.json`, records what is installed: Reach version and path, each skill's unit/version/`PROMOTED.json` and the harnesses it landed in, hook entries written, timer state. `update`, `doctor` and `remove` read it instead of guessing from the filesystem (REQ-8 depends on it).
- `[DEC-3]` Sources (S5): Reach from `sno-station-core` GitHub Releases (`reach-<version>.tar.gz` and its `.sha256`), skills from a tagged `sno-station-skills` release; `--reach-version` and `--skills-version` pin; the default is the latest tag of each (REQ-1, REQ-6).
- `[DEC-4]` Install decision per skill per harness from `requires:` (F4): a `required` slot the harness lacks → `skip` (REQ-2); a `preferred` slot missing → `degraded` (installed, one line says why); `programs` with `min_version` higher than installed Reach → `skip` naming the version. The harness→slot table is the capability map's row cells rendered as yes/no per harness, shipped inside the CLI as data.
- `[DEC-5]` Hooks: the installer merges the progress-reminder entry (F5) into Claude's and Codex's hook files idempotently (present once, keyed by the command's `sno reach remind` substring), leaves other entries untouched, and prints S4's Codex trust action (REQ-4).
- `[DEC-6]` `agents.json` is written only when absent; when present it is validated and left alone. ACP adapters and `acpx` are detected, not installed: a listed kind whose adapter is not runnable is reported by `doctor` as `acp:<kind> missing — run: <npx line>`.
- `[DEC-7]` `remove` deletes exactly what the manifest lists and nothing else, then the manifest; it keeps `~/.local/state/sno-reach/` (mail is user data) unless `--purge-state`.
- `[DEC-8]` Every verb has `--json`; every line of the human output is `<verb> <target> <result>` where result ∈ installed/updated/current/degraded/skipped/removed/missing/ok/fail.

### 5.2 Requirements

- `[REQ-1]` `sno assemble` SHALL download the pinned Reach release and its checksum, refuse on mismatch before writing anything, unpack to `~/.local/lib/sno-reach/releases/<version>/`, point `current`, link `~/.local/bin/sno-reach`, and print `assemble reach installed <version>`; a second run with nothing newer SHALL print `current` and change no file.
- `[REQ-2]` `sno assemble` SHALL copy each shipped category-S skill into every present harness whose `requires:` are met per DEC-4, printing one line per skill per harness with `installed`, `degraded`, or `skipped` and the reason for the last two.
- `[REQ-3]` `sno assemble` SHALL write `~/.config/sno-reach/agents.json` when absent with the kinds whose harness is present, validate it when present, and SHALL NOT overwrite a present file.
- `[REQ-4]` `sno assemble` SHALL merge the progress-reminder hook into `~/.claude/settings.json` and `~/.codex/hooks.json` exactly once each, preserving every other entry byte-for-byte, and after writing the Codex entry SHALL print the trust action.
- `[REQ-5]` `sno assemble` SHALL write the manifest (DEC-2) last, so a manifest without a matching filesystem state cannot exist after a crash mid-run; a run interrupted before the manifest SHALL leave the previous manifest untouched.
- `[REQ-6]` `sno update` SHALL perform REQ-1..5 against the latest tags, print `updated`/`current` per piece, and with `--auto on` SHALL install a daily user timer running `sno update --quiet`; `--auto off` SHALL remove it; `--quiet` SHALL print only changed pieces.
- `[REQ-7]` `sno doctor` SHALL print sections `reach`, `skills`, `hooks`, `acp`, `timer`, `station`, each check as `ok`/`fail`/`missing` with the fix command on the same line, and exit non-zero when any `fail`; `sno doctor --json` SHALL emit the same as one object.
- `[REQ-8]` `sno remove` SHALL delete exactly the manifest's entries (release directories, `current`, the PATH link, skill directories it installed, the hook entries it wrote, the timer), keep `~/.local/state/sno-reach/` unless `--purge-state`, print one `removed` line per item, and delete the manifest last.
- `[REQ-9]` Every verb SHALL exit 0 on success, 2 on usage, 3 when a source cannot be fetched or fails its checksum, 4 when a harness file cannot be written, and SHALL never leave a half-written file (write to a temp path and rename).
- `[REQ-10]` `sno station doctor` SHALL keep working and print only the `station` section.
- `[REQ-11]` `sno starport` SHALL be refused as a retired root command with the usage message `'starport' is not a top-level command; run 'sno --help'` and exit 2, and SHALL not appear in `sno --help`.

### 5.3 Disposition table

| Input or failure class | Fate |
|---|---|
| First run, all sources reachable | Reach installed, skills placed, agents.json written, hooks merged, manifest written |
| Re-run, nothing newer | Every piece `current`; no file changes |
| Checksum mismatch | Exit 3 before any write, naming the file |
| Network unavailable | Exit 3 naming the URL; manifest untouched |
| Harness absent | `skipped: harness not present` per skill |
| Skill with a `required` slot the harness lacks | `skipped` naming the slot (REQ-2) |
| Skill `preferred` slot missing | `degraded` naming the slot; installed |
| Skill needs Reach ≥ 2.0, installed lower | `skipped` naming both versions |
| `agents.json` present and valid | Left alone, `current` |
| `agents.json` present and malformed | `doctor fail` naming the file; `assemble` exits 4 |
| Hook entry already present | Not duplicated; `current` |
| Hook file unwritable | Exit 4 naming the file; earlier pieces stay installed; manifest not written |
| Codex hook newly written | Trust action printed |
| ACP adapter missing for a listed kind | `doctor missing` with the npx line; `assemble` proceeds |
| `remove` with a manifest | Exactly the listed items removed |
| `remove` without a manifest | Exit 2: "nothing assembled on this machine" |
| Timer unsupported (no systemd/launchd) | `--auto on` exits 2 naming the platform |

### 5.4 Dead designs

| What | Why killed | Who | When |
|---|---|---|---|
| One verb `sno assemble` with `--check`/`--remove`/`--auto` | Owner chose four words | larry | 2026-09-13 |
| Everything under `sno station …` | Owner chose top-level | larry | 2026-09-13 |
| Installing from a git checkout at HEAD | Unpinned, unverifiable | this PRD (S5) | 2026-09-13 |
| A daemon of our own for auto-update | Platform schedulers exist | this PRD (S2) | 2026-09-13 |

DO NOT REOPEN without a named new fact.

### 5.5 Ordered steps

- `[STP-1]` Add the four root verbs to `src/cli.rs` with `--json`, the manifest type, and the exit codes; remove `starport` and retire its name (REQ-9, REQ-10, REQ-11). Prerequisite: none.
- `[STP-2]` Reach install/update/remove path (REQ-1, REQ-6 core, REQ-8 Reach part). Prerequisite: STP-1; a Reach release tarball to test against (the core PRD's `make install` output packed as `reach-<version>.tar.gz` is acceptable as the fixture).
- `[STP-3]` Skill placement with the `requires:` decision and the harness slot table (REQ-2). Prerequisite: STP-1; one promoted skill directory as fixture.
- `[STP-4]` `agents.json` and hooks (REQ-3, REQ-4). Prerequisite: STP-1.
- `[STP-5]` `doctor` and the timer (REQ-6 `--auto`, REQ-7). Prerequisite: STP-2..4.
- `[STP-6]` `remove` (REQ-8). Prerequisite: STP-5.
- `[STP-7]` End-to-end on a clean user account (section 8). Prerequisite: STP-6.

## 6. Non-goals and fences
<!-- prd:non-goals -->

- NOT building: Reach; skills; an ACP adapter installer; a Windows path; a GUI.
- NOT changing: `sno account`, `sno station telemetry|audit|rem-*`; the external dispatch.
- NOT optimizing: download speed; parallel harness writes.
- NOT supporting: installing from a working tree; per-skill cherry-picking beyond `--reach-version`/`--skills-version`; categories other than S in this PRD (the same code path takes them later without change).
- Deferred with the owner: none — the last open item (`sno starport`) was ruled on 2026-09-13.

Non-reopen: rulings 1–5; dead designs 5.4.

## 7. Test strategy and end-to-end proof
<!-- prd:test-strategy -->

Changed guarantees that may receive tests: the four verbs and exit codes (REQ-9, 10), the
Reach install/current/checksum path (REQ-1), the `requires:` decision (REQ-2), `agents.json`
and hook idempotence (REQ-3, 4), manifest-last (REQ-5), update/timer (REQ-6), doctor (REQ-7),
remove (REQ-8). Unchanged: existing `account`/`station` commands and their tests.

Test scope: add or change only tests that can fail because of this change; reuse existing proof when it already covers the behavior.

- Unit: required — Rust tests for the `requires:` decision table, manifest read/write, hook merge idempotence, checksum refusal.
- Integration: required — the CLI against a local fixture release directory served from a file URL and a fixture skills tree, in a temp `HOME`.
- End-to-end: required — below.
- Eval/simulation: not applicable — no model output is produced.

E2E proof: required — a clean user account with Claude Code and Codex present ends up, after one `sno assemble`, able to spawn a Reach seat and complete a work item, and after `sno remove` has none of it.

External systems: a Reach release artifact from `sno-station-core`, a tagged `sno-station-skills` release, Claude Code and Codex installed, ACPX with the Codex adapter, a Codex login — Chapter 0 required

Chapter 0: e2e-environment-preflight — the preflight settles the two release artifacts, the two harnesses' versions, ACPX and the adapter, and the login on the proof machine; its artifact is one baseline file naming each, stamped into every `[E2E]` receipt.

E2E test author: current agent using `test-writer` — the seat journey is the Reach PRD's own; this PRD adds the install and remove layers around it.

## 8. Acceptance
<!-- prd:acceptance -->

- `[QCG-1]` With a fixture Reach release and checksum on a file URL and an empty temp `HOME`, `sno assemble --reach-version 2.0` prints `assemble reach installed 2.0`, `~/.local/lib/sno-reach/current/VERSION` reads `2.0`, `~/.local/bin/sno-reach --version` prints `2.0`; a second run prints `current` and no file under `~/.local/lib/sno-reach` changes mtime; with one byte of the checksum altered the first run exits 3 and writes nothing. · proves: REQ-1, REQ-9
- `[QCG-2]` With a fixture skills tree holding `reach` (requires program reach ≥ 2.0, harness `2.pre-turn-context-injection` preferred) and a temp `HOME` with `~/.claude/skills` and `~/.hermes/skills` present, `sno assemble` prints `installed` for Claude, `degraded` naming the slot for Hermes, and `skipped: harness not present` for Codex; with Reach pinned at `1.9`, both print `skipped` naming `2.0` and `1.9`. · proves: REQ-2
- `[QCG-3]` With no `agents.json`, `sno assemble` writes one listing the present harness kinds; with a valid one present its bytes are unchanged after a run; with a malformed one `sno assemble` exits 4 naming the file and `sno doctor` prints `acp agents.json fail`. · proves: REQ-3, REQ-7
- `[QCG-4]` With `~/.claude/settings.json` and `~/.codex/hooks.json` each holding one unrelated hook, `sno assemble` leaves those entries byte-identical, adds the reminder entry once to each, prints the Codex trust action, and a second run adds nothing and prints `hooks current`. · proves: REQ-4
- `[QCG-5]` With the hook file made read-only mid-run, `sno assemble` exits 4 naming it, Reach stays installed, and `~/.config/sno/assemble.json` is absent (first run) or unchanged from before (second run). · proves: REQ-5, REQ-9
- `[QCG-6]` With a newer fixture release published, `sno update` prints `updated reach <old> -> <new>` and `current` for unchanged skills; `sno update --auto on` creates the user timer and `sno doctor` shows `timer ok`; `--auto off` removes it and `doctor` shows `timer missing`. · proves: REQ-6, REQ-7
- `[QCG-7]` With everything assembled, `sno doctor` exits 0 and prints the six sections all `ok`; with `current` deleted it exits non-zero printing `reach current fail — run: sno update`; `sno doctor --json` parses and carries the same sections; `sno station doctor` prints only the station section. · proves: REQ-7, REQ-10
- `[QCG-8]` With everything assembled and one unrelated file placed in `~/.claude/skills/`, `sno remove` prints one `removed` line per manifest item, leaves the unrelated file and `~/.local/state/sno-reach/`, deletes the manifest last, and a second `sno remove` exits 2 with "nothing assembled on this machine"; with `--purge-state` the state root is gone. · proves: REQ-8
- `[QCG-9]` `[E2E]` On a clean user account with Claude Code and Codex installed and Chapter 0 green, `sno assemble` followed by `sno reach spawn codex --as executor.e2e@<host>` and one `sno reach send` ends with an `accepted` and a `completed` card, using only what `assemble` placed; then `sno remove` leaves `sno reach` not found on PATH; planted defect: the installer linking `current` at the previous release makes `sno reach --version` disagree with the manifest and `sno doctor` go `fail`, and the spawn journey does not run. · proves: REQ-1, REQ-2, REQ-4, REQ-8
- `[QCG-10]` With the CLI built, `sno starport` exits 2 printing `'starport' is not a top-level command; run 'sno --help'`, and `sno --help` does not contain the word `starport`. · proves: REQ-11

## 9. Residuals and leads
<!-- prd:residuals -->

Audience: the next agent.

- The harness→slot table (DEC-4) is data copied from the capability map; fourteen cells there
  are unverified and must be probed before a skill may `require` them.
- ACP adapters remain the user's npm packages; a later PRD may let `assemble` install them.
- Categories J, T, H, R reuse REQ-2's path when their skills ship; no code change expected.

## 10. Execution notes
<!-- prd:execution-notes -->

Order: STP-1 → STP-2/3/4 (parallel) → STP-5 → STP-6 → STP-7. Fixtures: a Reach tarball built
from the core PRD's `make install` output; one promoted skill directory from the skills PRD.

### Direct Execution Contract

| Field | Content |
|---|---|
| Change boundary | Baseline: the `sno-cli` working tree at the commit this PRD is released on, recorded by `git rev-parse HEAD` and `git status --porcelain` into `ai-docs/PRD/installer/sidecar/baseline.txt` before the first edit. Allowed paths: `src/cli.rs`, `src/doctor.rs`, new `src/assemble.rs`, `src/manifest.rs`, `src/harness_slots.rs`, `tests/assemble*.rs`, `tests/fixtures/assemble/**`, `README.md` (Commands section), `Cargo.toml` (dependencies only). Non-goals: section 6. Final comparison: `git status --porcelain` (tracked, staged, untracked) diffed against the baseline; any path outside the list is reported, not absorbed. `openspec/**` is excluded and must be identical to baseline (`git diff --stat -- openspec/` empty; no untracked files there). |
| Implementation order | STP-1 first (verbs parse, exit codes, `--json`), then STP-2, STP-3, STP-4 in any order, then STP-5, STP-6, then STP-7. This is a new feature, not a correction: no causal RED is owed before product edits; each REQ's integration test is written red against the fixture before its code lands. |
| Causal test | Not applicable as a pre-edit RED (new feature, no defect being fixed); the causal proof is each `[QCG]` row's negative half (checksum altered → exit 3; hook file read-only → exit 4; `current` deleted → doctor fail), recorded under `ai-docs/PRD/installer/sidecar/<stem>.QCG-n.proof.json` by `prd-proof.py`. |
| Integration proof | `cargo test --test assemble` against the fixture release (file URL) and fixture skills tree in a temp `HOME`: real CLI entry, real files written, read back by the test from the manifest and the filesystem (QCG-1..8). |
| User journey | One change-affected primary flow: assemble → seat journey → remove, mapped to the end-to-end acceptance row QCG-9; its planted defect (`current` linked at the previous release) makes the same journey red and restoration green. No other primary flow is affected. |
| Negative proof | Checksum mismatch exits 3 with nothing written (QCG-1); unwritable hook file exits 4 with the manifest untouched (QCG-5); malformed `agents.json` exits 4 (QCG-3); second `remove` exits 2 (QCG-8); `doctor` fails on a deleted `current` (QCG-7). |
| Completion | `cargo build --release` exit 0; `cargo test --test assemble` exit 0 with the row count printed; `cargo clippy -- -D warnings` exit 0; each QCG row run through `prd-proof.py` with receipts under `ai-docs/PRD/installer/sidecar/`; `git diff --stat -- openspec/` empty and no untracked files under `openspec/`; the final `git status --porcelain` compared with `baseline.txt`. Any nonzero or unexpected result blocks review and success. |
| Stop conditions | If a fixture cannot be built (no Reach release yet): build the tarball from the core PRD's `make install` output and record its sha256 in the fixture directory; do not park. If a proof cannot run (no Codex login on the proof machine): run QCG-1..8 fully, record QCG-9 as blocked with the exact missing precondition, and ask the owner for the login only — that is an owner-only access. If a fix cannot land inside the boundary: report the path, diagnose against the settled intent, repair the contract row (not the boundary) and continue. Never park the task. |
