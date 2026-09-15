---
name: "sno-assemble"
title: "sno assemble: install, update, check and remove the station"
version: "1.5"
prd_status: "released"
project_status: "not_started"
updated: "2026-09-14"
owner: "larry"
pipeline: "small"
---

# PRD — `sno assemble`: install, update, check and remove the station with one CLI

Four top-level verbs in the `sno` CLI put a machine on the team and keep it current:
`sno assemble` installs core programs once per user and shipped skill text into each eligible agent harness
present on this machine; `sno update` brings these to the latest published version; `sno doctor`
checks everything installed (and absorbs today's `sno station doctor`); `sno remove` takes it
all out. The inputs are release artifacts governed by contracts from two other repositories; a released PRD does not mean its program or artifact has shipped: the Reach naming
contract and `apps/reach/VERSION` (`sno-station-core`, PRD `ai-doc/ACTIVE/PRD/reach/reach-PRD.md`,
released 2026-09-13) and each shipped skill's `PROMOTED.json` plus `requires:` block
(`sno-station-skills`, PRD `ai-doc/ACTIVE/PRD/cat-S/s-category-publish-PRD.md`, revision 1.4 draft). Utility program artifacts and the shared requirements data are incoming dependencies, not already published facts.

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
| 7 | 2026-09-13 | larry | `sno assemble` is the one official name. `sno install` is an alias: it runs the same code path, is not listed in `--help`, and its first output line says `assembling …` so the user learns the official word. `setup` is reserved: not an alias, not a verb here; the owner may use it elsewhere. ("assemble 是正名，install as 别名. Setup，我再想想，可以留着在其他地方用。") |

Decisions under standing authority:

| # | Date | Decision | Why no escalation |
|---|---|---|---|
| S1 | 2026-09-13 | `assemble` and `update` share one install path. Unchanged pieces report `current`; a replacement reports `updated`; an absent piece reports `installed`. | One path, no drift between install and update. |
| S2 | 2026-09-13 | Automatic update is a user-level scheduled timer that runs `sno update --quiet` once a day, switched by `sno update --auto on|off`; on Linux it is a `systemd --user` timer, on macOS a `launchd` agent; no daemon of our own. | The two platforms already ship a scheduler; writing one is scope creep. |
| S3 | 2026-09-13 | Recognise Claude Code, Codex, Hermes and OpenClaw instances. Resolve the instances selected by the invocation’s environment and user configuration, including their active skill roots, before writing; do not scan unrelated user accounts or guess other profiles. `~/.agents/skills` is a shared destination, not a fifth harness; it is used only for identified consumers that actually load it. | The measured capability map records shared symlinks and a custom OpenClaw state root that excludes default home roots. |
| S4 | 2026-09-13 | The Codex hook trust hash cannot be written by the installer; after wiring a Codex hook the installer prints the one action the user must take (open Codex, press `t` on the new hook). | Measured 2026-09-11: Codex refuses to run an untrusted hook and the trust record is written only by its own UI. |
| S5 | 2026-09-13 | Sources are pinned release artifacts, never a checkout at HEAD: Reach, heartbeat and subscription-quota-check from core releases; text and requirements data from the exact `final-skills.tar.gz` plus `.sha256` assets on a tagged skills release. Resolve the tag to its commit before writing; record asset URLs and SHA-256 values. Production release acceptance requires actual upstream artifacts. | Local fixtures prove installer behavior, not publication or artifact compatibility. |

## 2. The problem, measured
<!-- prd:problem-measured -->

Measured 2026-09-12/13 on the owner's workstation and in the three repositories.

- At the reviewed local baseline, `src/cli.rs` already retires `starport`; root commands are `account`, `station`, and external dispatch. There is no installer yet. The earlier 2026-09-12 help output predates that change. Preserve the existing retirement and dispatch behavior.
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
| F1 | `sno` is a Rust clap CLI; root commands are `account`, `station`, plus external dispatch. `starport` is already retired; `doctor` must leave the retired-name list when its root verb is added. | `src/cli.rs`, local probe record | 2026-09-13 |
| F2 | `sno station doctor` prints badge lines for identity, buffer, consent, last ship, lockfile; `--json` gives a structured object. | `src/doctor.rs` 38–70 | 2026-09-13 |
| F3 | Reach installs at `~/.local/lib/sno-reach/releases/<version>/` with `bin/sno-reach`, `lib/`, `vendor/`, `spec/`, `guide/`, `VERSION`, `NOTICE`, `LICENSE`; `current` is the stable path; `sno reach --version` prints `VERSION`; `sno reach doctor --as` checks a seat; config `~/.config/sno-reach/agents.json` maps kind → `{"acpx_agent": name}`; `sno reach remind --as $SNO_REACH_ADDR` is the hook command. | Reach PRD REQ-1, 2, 14, 16, section 5.3 | 2026-09-13 |
| F4 | Category-S revision 1.4 defines `<category>/<unit>/PROMOTED.json` beside `skill/SKILL.md` and supporting text/self-tests. `requires` has exactly programs and harness lists; identifiers and version rules come from `scripts/requirements-contract.json`, shared with the gate. The contract file is an incoming artifact until published. | S publication DEC-3, REQ-8, sections 5.3/5.7 | 2026-09-13 |
| F5 | Claude Code reads `~/.claude/settings.json` `hooks.UserPromptSubmit[]`; Codex reads `~/.codex/hooks.json` `hooks.UserPromptSubmit[]` and records trust per hook in `~/.codex/config.toml` `[hooks.state."…:user_prompt_submit:<i>:0"] trusted_hash`; the hook command shape is `text=$(timeout 5 <cmd>) || exit 0; [ -n "$text" ] || exit 0; jq -nc --arg text "$text" '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:$text}}'`. | workshop `skills/tpm/skills/tpm/references/progress-reporting-hooks.md`; measured hook files 2026-09-11 | 2026-09-11 |
| F6 | ACP adapters are npm packages run through `npx -y @agentclientprotocol/claude-agent-acp@<v>` and `@agentclientprotocol/codex-acp@<v>`, configured in `~/.acpx/config.json` `agents.<name>.argv`; `acpx` itself is a Node CLI on PATH. | measured `~/.acpx/config.json` on the proof VM and workstation, 2026-09-11 | 2026-09-11 |
| F7 | Default skill roots are `~/.claude/skills`, `~/.codex/skills`, `~/.hermes/skills`, and `<OpenClaw stateDir>/skills`. The measured Codex root is a symlink to `~/.agents/skills`; Hermes shared loading requires configuration; custom `OPENCLAW_STATE_DIR` can exclude home roots. These defaults do not establish active-instance discovery. | capability map, perspective 5; local probe record | 2026-09-13 |
| F8 | The CLI ships through crates.io and GitHub Releases with SHA-256 checksums and shell/PowerShell installers. | `README.md` "Install" | 2026-09-13 |

The facts above distinguish specified contracts from implemented behavior. The local probe
`reports/PROBE-RESULTS-sno-assemble-peer-review.md` found no `apps/reach` directory and no
promoted S units in the checked-out upstream repositories. It did not inspect remote releases.
The capability map marks Hermes pre-turn context injection supported (`pre_llm_call`).
Category-S revision 1.4 S2 withdrew the skill-runtime exception after correcting the
source measurements. Both utility programs come from core; category S publishes text and
self-tests only. This supersedes the version-1.2 installer assumption that the skills
release carries public commands. Owner ruling 4 limits Reach entry points to `sno-reach`;
core utility commands are separate programs, not extra Reach verbs.

## 4. Actors and handoff boundaries
<!-- prd:actors-handoffs -->

| Actor | Touches this how | Succeeds when |
|---|---|---|
| A person at a terminal | Runs `sno assemble` once, `sno update` (or the timer) later, `sno doctor` when something is off, `sno remove` to leave. | One command each; every line of output says what was done to which path. |
| The Reach program (from core releases) | Is downloaded, verified, unpacked into a release directory, and linked as `current`; `sno-reach` linked onto PATH. | `sno reach --version` equals the pinned version. |
| A shipped skill (from the skills repository) | Its requirements are resolved by program name and actual harness capability before its nested text payload is placed. | The harness discovers `<unit>/SKILL.md`, references and the copied stamp; no runtime is expected in the skill release. |
| A core utility program | Its own release name, version, checksum, entry point and resources are verified and installed once per user. | Install/update/doctor/remove track its owned files and links independently of Reach and skill text. |
| Each harness (Claude Code, Codex, Hermes, OpenClaw) | Gets skill text, and for Claude Code and Codex the progress-reminder hook entry. | The hook entry is present exactly once; Codex's trust step is printed. |
| The scheduler (systemd user timer / launchd) | Runs `sno update --quiet` daily when `--auto on`. | `sno doctor` reports the timer active. |

Boundary: this PRD does not build Reach or the skills; it installs what those repositories
publish. It does not install ACP adapters or `acpx` (npm packages the user owns); it detects
them and tells the user what is missing.

## 5. The design
<!-- prd:design -->

### 5.1 Settled decisions

- `[DEC-1]` Four root verbs (ruling 3): `assemble`, `update`, `remove`, `doctor`. `sno station doctor` stays as an alias printing only the station section of `sno doctor`. `starport` leaves the root command list and joins `RETIRED_ROOT_COMMANDS` (ruling 6).
- `[DEC-2]` `~/.config/sno/assemble.json` records the last committed installation: artifact identities, each owned file hash or symlink target, canonical destination and consuming harness instances, program records keyed by name with version/checksum/owned paths, the shared requirements contract version/hash, exact owned hook entries, generated `agents.json`, and timer configuration. A durable sibling `assemble.pending.json` records an in-progress mutation and rollback material before any installed target changes. Every mutating verb obtains the same exclusive lock; a competing run exits 4 immediately. `doctor` reads committed and pending state without repairing it.
- `[DEC-3]` Resolve Reach, heartbeat and subscription-quota-check independently from core release manifests; record each program name, actual version, artifact URL, checksum, layout and entry point. Reach uses `reach-<VERSION>-<os>-<arch>.tar.gz` and the same name plus `.sha256`, with literal `std::env::consts::OS` and `std::env::consts::ARCH` values. Supported published pairs are linux-x86_64 and macos-aarch64; linux-aarch64, macos-x86_64 and other unsupported architectures on a supported OS exit 2 naming the pair before any download. The current release has no artifacts for the two excluded pairs. Private release API downloads use `GH_TOKEN`; credentials are sent only to api.github.com. Native Windows remains refused; WSL follows its Linux pair without special handling. (2026-09-14 patch: owner-approved platform-qualified Reach artifacts replace the unqualified pair; no mapping table.) Utility names/versions/layouts come from their accepted core manifests, not this document’s guesses. `--reach-version` pins Reach; `--skills-version` pins text and requirements data; utility versions resolve to their latest eligible core release satisfying the shared declarations. Exclude draft/prerelease and unrelated package tags. A failed fetch/checksum for a selected artifact exits 3. An unavailable declared program yields the named skip in DEC-4; missing runtime artifacts still block complete release acceptance. Isolated consumer proof uses the internal release-source fixture. The explicitly authorized first-install bootstrap may use `--skills-archive PATH` through the normal production command, while core assets still come from published releases. Record the local archive digest as a bootstrap identity; this option conflicts with `--skills-version` and cannot satisfy final published-release acceptance. See REQ-1/2.
- `[DEC-4]` Gate and installer consume the same versioned `scripts/requirements-contract.json`, pin its exact SHA-256, and use its literal IDs rather than generating slugs from row labels. Resolve each declared program by its own name against its own installed verified artifact after program installation; an absent or too-old program skips the dependent skill, naming program, required and actual version. Validate declarations before any install changes: missing requires, wrong types, duplicate keys/entries, unknown fields/IDs, invalid need or malformed version are source errors (exit 3), not empty requirements. For known capability IDs, unsupported/partial/unverified required → skipped, preferred → degraded, optional → no restriction. Hermes supports the measured pre-turn slot; reader-to-agent delivery is unsupported until its own live proof exists. Canonicalise root aliases; a shared destination requires every identified consumer to meet required slots, and any missing preferred slot degrades with that consumer named. Unresolved consumers/roots are skipped with a reason. See REQ-2 and section 5.6.
- `[DEC-5]` Merge the reminder into each present Claude/Codex instance’s resolved hook configuration. Match the complete canonical reminder entry, not a substring; a similar or conflicting user entry is not adopted or deleted. Preserve unrelated entry bytes and user configuration. Record only entries actually created by this installer. Print the Codex trust action until trust is established; never write trust hashes. With no `SNO_REACH_ADDR`, the hook returns no context without calling Reach. With an address, bound execution to five seconds; errors are visible in diagnostics/stderr without blocking the user prompt. Validate wrapper dependencies on each supported platform; do not assume GNU `timeout` exists on macOS. See REQ-4.
- `[DEC-6]` Write `agents.json` only when absent, using the resolved harness kinds and the published adapter mapping verified at release preflight; record ownership if created. Validate existing config and leave it unowned and unchanged. ACP adapters and `acpx` are detected, not installed. A listed kind whose adapter cannot run is `acp:<kind> missing` with its exact setup command. Do not infer an adapter name merely from a harness executable name.
- `[DEC-7]` Never replace an unowned existing destination, even if its bytes happen to match. Refuse the conflicting install with exit 4 before mutation. Existing shared parent directories and configuration containers remain user-owned. Before update/remove, compare owned files, symlinks and hook entries with the manifest; refuse changed content rather than overwrite/delete it. An absent owned target may be restored by update or counted already removed by remove; absence is not a user-modified-content conflict. Delete owned files and links only, then remove owned directories only when empty. Preserve new user files inside installed directories. Keep Reach mail state unless explicit `--purge-state`. Recover pending work before update/remove; a pending record is not “nothing assembled”. See REQ-8.
- `[DEC-8]` Every verb has `--json`; human output is `<verb> <target> <result> [detail]`, with result installed/updated/current/degraded/skipped/removed/missing/ok/warn/fail/disabled. JSON names the same target, result, detail and fix action. `doctor` preserves existing station checks, including `warn` and their nonzero exit behavior. Timer intentionally off is `disabled`, not missing. A configured hook is not proof of execution: report configuration, trust and runtime readiness separately.

### 5.2 Requirements

- `[REQ-1]` `sno assemble` SHALL fetch and verify all selected artifacts in temporary staging before modifying installed targets, unpack validated Reach content to `~/.local/lib/sno-reach/releases/<version>/`, point `current`, and link `~/.local/bin/sno-reach`. Refuse checksum mismatch with exit 3 and no installed-state changes. Reject archive traversal and links escaping the staged payload. Print `assemble reach installed <version>`; an unchanged repeat SHALL print `current` and leave installed files and manifest bytes/mtimes unchanged (temporary staging/lock activity excluded).
- `[REQ-2]` Install core utility artifacts through REQ-1’s staged checksum, ownership and transaction path, preserving each manifest’s declared layout/resources/modes and public entry point. Track install/update/doctor/remove separately by program name; no utility runtime is taken from a skills release. Validate the pinned shared requirements contract and all four units’ declarations, install available selected core programs, then resolve dependencies per DEC-4 before text placement. Copy the contents of each S `<category>/<unit>/skill/` into the eligible harness `<unit>/`, with PROMOTED.json beside SKILL.md; preserve references and self-tests. Validate family/overlay declarations under the same publisher contract. Emit one installed/degraded/skipped result per unit/consumer. Missing or insufficient program/capability never silently becomes supported. QCG-2 provides the shared consumer proof used by S QCG-2/QCG-8.
- `[REQ-3]` `sno assemble` SHALL write `~/.config/sno-reach/agents.json` when absent with the kinds whose harness is present, validate it when present, and SHALL NOT overwrite a present file.
- `[REQ-4]` Merge the reminder once into each present Claude/Codex instance’s resolved hook file per DEC-5. Preserve unrelated entry bytes; never create a missing harness merely to install a hook. Print the trust action for an untrusted Codex entry. `doctor` SHALL distinguish configured, untrusted, missing dependency, missing seat address and runtime-unverified conditions; it SHALL NOT report reminder execution `ok` from entry presence or seat-message delivery alone. Actual prompt-triggered context injection is proved by QCG-9.
- `[REQ-5]` Before changing installed files, links, config entries or timers, write and durably flush the pending transaction, including prior values and backups. Publish the new manifest atomically only after target verification. On handled failure, restore the prior committed state; after interruption, the next mutating command completes rollback before doing new work. Rollback is idempotent. Retain pending state and return 4 if recovery cannot complete; `doctor` reports `fail` with the retry command. If manifest commit already succeeded, recovery completes cleanup rather than undoing that committed generation. The pending record and manifest carry a transaction identity so restart can distinguish an uncommitted operation from committed cleanup. No update/remove may trust a stale manifest while pending work exists. Do not roll back unrelated user edits made after the snapshot; report the conflicting path and retain recovery data.
- `[REQ-6]` `sno update` SHALL use REQ-1..5 with the selected latest or pinned releases, tracking each core program by its own name/version/checksum, and print `<verb> <target> updated <old> -> <new>` or `current`. `--auto on` installs a daily user timer invoking the resolved absolute `sno` executable with the same user/config roots and an explicit usable PATH; `--auto off` removes the owned timer without fetching releases. Timer changes obey the same lock/recovery rules. Scheduler enable failure is reported, never recorded as active. Automatic updates and manual mutations cannot overlap. `--quiet` suppresses unchanged successes, not failures, recovery needs or trust actions.
- `[REQ-7]` `sno doctor` SHALL report `reach`, `skills`, `hooks`, `acp`, `timer`, `station` with fix actions for actionable conditions and the same structure in JSON. The skills section includes named core utility dependency checks against their own installed artifacts; Reach’s version never substitutes for another program’s version. Exit nonzero for any fail, required missing dependency, untrusted required hook, pending recovery, or nonzero station result. Optional timer off is `disabled`; absence of seat context/runtime execution evidence is `warn`, not a fabricated pass. Preserve station’s existing checks and warn semantics; assembling Reach does not bootstrap telemetry identity, buffer or shipment history. A machine with these conditions need not be all green after installation.
- `[REQ-8]` `sno remove` SHALL recover pending work, then remove verified owned Reach files/links, skill text payloads and each core utility’s owned files/command links, generated unchanged `agents.json`, owned hook entries and timer; preserve unowned or changed content per DEC-7. Never remove whole shared skill/config roots. Preflight ownership conflicts before deleting, journal removal for retry, and delete the manifest only after owned items are removed. Preserve state unless `--purge-state`, which deletes only the explicit Reach state root without following links. Print one removed result per item. An interrupted removal first restores the last committed installation through REQ-5, then retries removal; its rollback copies remain available until manifest deletion is committed. Explicit state purge occurs after managed removal commits, so its irreversible deletion is not part of rollback; a retained purge record lets a retry finish only that requested state-root deletion.
- `[REQ-9]` Exit 0 on success, 2 on usage, 3 on source/checksum/layout errors, and 4 on filesystem, ownership, lock, configuration, timer or recovery errors. `doctor` uses exit 1 for unhealthy results. Stage and atomically replace individual files; durable transaction recovery supplies cross-file consistency. No checksum/source failure may alter installed state.
- `[REQ-10]` `sno station doctor` SHALL keep working and print only the `station` section.
- `[REQ-11]` `sno starport` SHALL be refused as a retired root command with the usage message `'starport' is not a top-level command; run 'sno --help'` and exit 2, and SHALL not appear in `sno --help`.

### 5.3 Disposition table

| Input or failure class | Fate |
|---|---|
| First run, all sources reachable | Reach installed, skills placed, agents.json written, hooks merged, manifest written |
| Re-run, nothing newer | Every unchanged eligible piece `current`; no installed-state changes; unmet prerequisites remain visible |
| Checksum mismatch | Exit 3 before any installed-state change, naming the file |
| Network unavailable | Exit 3 naming the URL; manifest untouched |
| Harness absent | `skipped: harness not present` per skill |
| Skill with a `required` slot the harness lacks | `skipped` naming the slot (REQ-2) |
| Skill `preferred` slot missing | `degraded` naming the slot; installed |
| Skill needs Reach ≥ 2.0, installed lower | `skipped` naming both versions |
| `agents.json` present and valid | Left alone, `current` |
| `agents.json` present and malformed | `doctor fail` naming the file; `assemble` exits 4 |
| Hook entry already present | Exact managed entry: current; exact unowned entry: preserved and recorded as external, never owned; conflicting entry: exit 4 (REQ-4) |
| Hook file unwritable | Exit 4; restore prior installed state; if rollback cannot finish, retain pending recovery data and report fail |
| Codex hook newly written | Trust action printed |
| ACP adapter missing for a listed kind | `doctor missing` with the npx line; `assemble` proceeds |
| `remove` with a manifest | Verified owned items removed; unrelated content preserved |
| `remove` without a manifest | Recover any pending install first; when no committed installation remains, exit 2: "nothing assembled on this machine" |
| Timer unsupported (no systemd/launchd) | `--auto on` exits 2 naming the platform |
| Unowned destination conflict | Exit 4 before installed-state changes; preserve original content |
| Owned file changed by user | Update/remove exits 4 naming the conflict; preserves content and records |
| User file added inside an installed directory | Remove owned files only; leave the user file and nonempty directory |
| Pending transaction after crash | Doctor fails; next mutation rolls back or completes committed cleanup first |
| Custom harness root or aliases | Resolve active roots; write once per canonical destination; report actual consumers |
| Timer intentionally off | Doctor timer disabled, not failure |
| Hook configured but untrusted | Doctor reports trust missing and exits nonzero |
| Hook without seat context | No Reach call; doctor runtime warning, not execution ok |

### 5.4 Dead designs

| What | Why killed | Who | When |
|---|---|---|---|
| One verb `sno assemble` with `--check`/`--remove`/`--auto` | Owner chose four words | larry | 2026-09-13 |
| Everything under `sno station …` | Owner chose top-level | larry | 2026-09-13 |
| Installing from a git checkout at HEAD | Unpinned, unverifiable | this PRD (S5) | 2026-09-13 |
| A daemon of our own for auto-update | Platform schedulers exist | this PRD (S2) | 2026-09-13 |

DO NOT REOPEN without a named new fact.

### 5.5 Ordered steps

- `[STP-1]` Add root verbs, manifest/pending transaction types, common lock and exit handling; preserve existing starport retirement (REQ-5, REQ-9..11). Include only module declarations and dependency lock changes needed by these modules. Prerequisite: none.
- `[STP-2]` Implement staged Reach installation and recovery (REQ-1, REQ-5, REQ-6, REQ-8). Prerequisite: STP-1; contract-shaped synthetic fixtures may drive local tests, explicitly labelled synthetic. Real release artifacts are required only for production acceptance, never fabricated as upstream evidence.
- `[STP-3]` Implement the shared requirements parser/hash pin, program-by-name artifact lifecycle, active-root/alias resolution, and nested skill text placement (REQ-2). Prerequisite: STP-1 and the S section-5.7 contract; use one shared declaration fixture for all four actual S units plus gate-produced staged bytes. No independently derived capability slugs or skill-contained runtime commands.
- `[STP-4]` `agents.json` and hooks (REQ-3, REQ-4). Prerequisite: STP-1.
- `[STP-5]` `doctor` and the timer (REQ-6 `--auto`, REQ-7). Prerequisite: STP-2..4.
- `[STP-6]` `remove` (REQ-8). Prerequisite: STP-5.
- `[STP-7]` Validate actual release artifacts, then perform clean-account acceptance (section 8). Prerequisite: STP-6 and the upstream release readiness evidence in section 7.

### 5.6 Shared requirements contract

For REQ-2, the shared file's exact program IDs are `reach`, `heartbeat`, and
`subscription-quota-check`. Its initial capability IDs are `4.shell`, `4.file-read-write`,
`4.background-processes`, `2.pre-turn-context-injection`, and `3.reader-to-agent-delivery`.
Version strings contain two or three dot-separated nonnegative decimal integers; compare
numerically, treating an omitted patch as zero. Whitespace-only values, prerelease aliases
and comparison expressions are invalid. Empty lists express no requirements, subject to
the four unit-specific dependencies. Unsupported and unrecognised are different: an unknown
ID is a source error; a known but unproved capability takes DEC-4's skip/degrade rule.

The shared fixture contains the actual pinned declarations of all four units: reach needs
Reach and shell, with reminder injection preferred; handoff needs Reach, shell and file
read/write; heartbeat needs its core program, shell, background execution and verified
reader-to-agent delivery; quota needs its own core program plus heartbeat and the mandatory
background/reader capabilities for its wait branch. Reach minimum is 2.0 with the required handoff contract (REQ-2); utility minimum versions come from accepted core manifests. No placeholder
version or invented supported cell can close acceptance. Gate and actual installer parser
run the same fixture and report its hash, contract version/hash and matching dispositions.
This is the consumer evidence for S QCG-2 and QCG-8, not a second test-only parser. (REQ-2)

## 6. Non-goals and fences
<!-- prd:non-goals -->

- NOT building: Reach or utility runtimes; skills; an ACP adapter installer; a Windows path; a GUI.
- NOT changing: `sno account`, `sno station telemetry|audit|rem-*`; the external dispatch.
- NOT optimizing: download speed; parallel harness writes.
- NOT supporting: installing from a working tree; per-skill cherry-picking beyond `--reach-version`/`--skills-version`; categories other than S in this PRD (the same code path takes them later without change).
- Deferred with the owner: none — the last open item (`sno starport`) was ruled on 2026-09-13.

Non-reopen: rulings 1–5; dead designs 5.4.

## 7. Test strategy and end-to-end proof
<!-- prd:test-strategy -->

Changed guarantees that may receive tests: the four verbs and exit codes (REQ-9, 10), the
Reach install/current/checksum path (REQ-1), the `requires:` decision (REQ-2), `agents.json`
and hook idempotence (REQ-3, 4), transaction recovery (REQ-5), update/timer (REQ-6), doctor (REQ-7),
remove (REQ-8). Unchanged: existing `account`/`station` commands and their tests.

Test scope: add or change only tests that can fail because of this change; reuse existing proof when it already covers the behavior.

- Unit: required — Rust tests for the `requires:` decision table, manifest read/write, hook merge idempotence, checksum refusal.
- Integration: required — the CLI against a local fixture release directory served from a file URL and a fixture skills tree, in a temp `HOME`.
- End-to-end: required — below.
- Eval/simulation: not applicable — no model output is produced.

E2E proof: required — a clean user account with Claude Code and Codex present ends up, after one `sno assemble`, able to spawn a Reach seat and complete a work item, and after `sno remove` has no installer-owned payload left; preserved user content and Reach state are explicitly excluded.

External systems: Reach and both utility release artifacts from `sno-station-core`, gate-produced staged skills bytes, a tagged `sno-station-skills` release with its shared requirements contract, Claude Code and Codex installed, Hermes and a custom-root OpenClaw instance for focused skill discovery, ACPX with the Codex adapter, required harness logins — Chapter 0 required

Chapter 0: e2e-environment-preflight — record actual artifact URLs, tags/commits, checksums, unpacked layouts, host, platform, harness versions, active config/skill roots, discovery evidence, wrapper dependencies, ACPX/adapter configuration, logins and Codex hook trust. Use the target user’s real PATH and scheduler environment. Record prerequisite station state without silently creating it. Every `[E2E]` receipt names this baseline.

Upstream readiness is a release gate, not an assumption: inspect each selected GitHub release and download the exact assets; compare Reach VERSION and layout with the naming contract, and inspect all four promoted S text units and their requirements contract; inspect the separate core utility artifacts and entry points. Record commands and raw outputs. A local synthetic fixture cannot satisfy this gate. If artifacts are absent, local implementation and fixture checks can proceed, but production acceptance and delivery remain blocked on the named upstream publication. Do not build a nonexistent upstream program or claim the PRD being released means its artifact exists.

Two evidence stages are required and recorded separately:

1. **Staged consumer proof (QCG-2):** the S gate produces disposable staged release bytes,
   actual stamps and the shared declaration fixture, recording source/gate/contract/payload
   hashes. Feed these bytes through the isolated installer's release-source fixture and
   actual parser. This can run before a production skills tag exists and is consumed by
   S QCG-2/QCG-8. Synthetic unit-test data alone cannot satisfy this stage.
2. **Published-release acceptance (QCG-9):** after S publication, install actual pinned core
   and skills release artifacts and repeat the complete user journey. Record actual release
   identities, hashes and contract hash; compare the published text bytes with the staged
   evidence under S's stamp/timestamp rules. Staged success does not replace final acceptance,
   and a final release does not erase the need for the staged consumer proof.

E2E test author: current agent using `test-writer` — the seat journey is the Reach PRD's own; this PRD adds the install and remove layers around it.

## 8. Acceptance
<!-- prd:acceptance -->

- `[QCG-1]` With a synthetic Reach 2.0 archive/checksum supplied through the internal fixture source provider and an empty isolated user environment, assemble installs the stated layout; the installed executable prints 2.0. A repeat leaves installed files and manifest unchanged. Build-host release selection chooses only its platform-qualified Reach name and matching checksum, while utility names remain unchanged; a faked unsupported pair exits 2 naming the pair before downloads. (2026-09-14 patch: affected platform selection/refusal proof.) Altered checksum or an archive entry escaping staging exits 3 with no installed changes. · proves: REQ-1, REQ-9
- `[QCG-2]` Staged consumer proof: use gate-produced nested payloads and the one shared fixture with real declarations from all four S units. The actual installer parser and S gate pin the same contract version/hash and fixture hash. Validate the same good and bad declarations, including unknown IDs, duplicate keys, wrong types, invalid need/version and overlay declarations. Verify numeric version comparison per program: missing/too-old heartbeat skips its dependent units even when Reach is current; quota resolves both its own program and heartbeat. Known partial/unverified required slots skip; preferred degrade; optional do not restrict. Claude and Hermes support the real pre-turn slot; a distinct unsupported fixture cell tests degradation without falsifying Hermes. A reader log alone never proves delivery. Verify the exact skill/ contents-to-unit mapping and stamps. Core utilities install once from their own verified artifacts; their installed read/help commands work, update changes their owned targets, and doctor detects a missing utility. Aliased roots are written once; custom roots use active discovery. Supply this consumer receipt to S QCG-2/QCG-8. · proves: REQ-2, REQ-6, REQ-7
- `[QCG-3]` With no `agents.json`, `sno assemble` writes one listing the present harness kinds; with a valid one present its bytes are unchanged after a run; with a malformed one `sno assemble` exits 4 naming the file and `sno doctor` prints `acp agents.json fail`. · proves: REQ-3, REQ-7
- `[QCG-4]` For each present Claude/Codex instance, an unrelated hook entry remains byte-identical and one reminder is inserted. Repeat adds nothing. An exact pre-existing external reminder remains unowned; a similar conflicting user entry causes refusal. Missing harnesses get no config. An untrusted Codex entry prints the trust action and doctor does not report execution ok. Without a seat address the wrapper produces no context and does not call Reach; failed addressed execution is bounded and leaves visible diagnostic output. Real injection is QCG-9. · proves: REQ-4, REQ-7
- `[QCG-5]` Inject a write failure after Reach activation but before hook commit, both on first install and update. Handled failure restores previous installed files, links and manifest. Kill a separate fixture run at that boundary: doctor fails on pending recovery; retry assemble or remove recovers before acting. First-install remove leaves no orphan payload even if it then returns 2. Interrupt after manifest commit: recovery keeps the committed generation and cleans backups. A forced recovery error preserves the pending record and exits 4. Start a concurrent mutation: it exits 4 without changing files. · proves: REQ-5, REQ-8, REQ-9
- `[QCG-6]` Publish a newer fixture release: update prints `update reach updated <old> -> <new>` and current for unchanged skills. Auto-on records a working user timer; run its actual scheduled command with scheduler PATH/config to verify the intended user installation is selected. Auto-off works offline, removes the owned timer and reports disabled. Enabling failure cannot produce active manifest state. · proves: REQ-6, REQ-7
- `[QCG-7]` After assembly, doctor emits all six sections and accurately reports their distinct states: optional timer disabled, untrusted hook missing, missing seat/runtime evidence warn, and unchanged station warnings where prerequisites were not initialised. Assert nonzero on required missing/trust failures and existing station warnings. In a separately prepared healthy station fixture with trusted hook config and required dependencies present, doctor exits 0 while still identifying runtime-unverified hook evidence as warn. Deleting current produces reach fail; JSON carries the same checks; station doctor preserves its original section and exit behavior. · proves: REQ-7, REQ-10
- `[QCG-8]` Before install, plant an unowned same-name skill directory and command link: assemble refuses without altering either. In a successful independent install, add an unrelated file inside an owned skill directory and beside it; remove preserves both, removes only recorded unchanged files/links/entries, keeps Reach state, and deletes the manifest last. Generated unchanged agents.json is removed; pre-existing config remains. Modify an owned file before update/remove: refusal preserves that file and the manifest. Interrupted removal resumes safely. A second completed remove exits 2; explicit purge removes only Reach state. · proves: REQ-8, REQ-9
- `[QCG-9]` `[E2E]` Published-release acceptance: with actual core/skills releases and Chapter 0 green, begin with both seat identities absent. Run assemble; verify installed versions and the doctor reach section (unrelated station warnings do not block this journey). Run `sno reach init --as tpm.e2e@<host> --name Sender` and `sno reach init --as executor.e2e@<host> --name Receiver`, then `sno reach spawn codex --as <address>` for both initialized seats; require `sno reach doctor --as <address>` to pass for each before send. No preflight or fixture may create their identity/registration. Following the installed skill and core section-7 exchange, the sender sends a nonce-bearing question; the receiver accepts then completes on the same original card. Observe terminal completion with one bounded `sno reach wait --as tpm.e2e@<host> --from executor.e2e@<host> --reply-to <question-id> --timeout 300`; inspect accepted-before-completed through export/state and thread IDs. Wait does not select accepted status. Preserve core’s reply-ring and sender-acknowledgment checks. Actual Claude/Codex discovery and an addressed real prompt prove reminder injection after explicit Codex trust. Execute each installed core utility’s harmless read/help entry without workshop paths. Point current at a different version: the same journey fails its version/Reach check before init/spawn; restoring it passes. Remove leaves no owned program commands, hooks or text, preserving user state. Focused actual discovery also covers Hermes and custom-root OpenClaw; unavailable proof is blocked, not inferred from copies. Record both evidence stages independently. · proves: REQ-1, REQ-2, REQ-4, REQ-8
- `[QCG-10]` With the CLI built, `sno starport` exits 2 printing `'starport' is not a top-level command; run 'sno --help'`, and `sno --help` does not contain the word `starport`. · proves: REQ-11

### Execution-contract repair — 2026-09-13

STP-1 makes root doctor a real command. The existing `tests/cli.rs` retirement list still
asserts it is rejected; the change boundary now explicitly permits removal of that one
obsolete assertion. Other existing command tests are preserved.

### Revision 1.3 — cross-repository closure, 2026-09-13

Align with category S 1.4 and Reach 1.3: utilities come from core, declarations use one
shared versioned/hash-pinned contract, both clean-state seats are initialized before spawn,
and staged consumer proof precedes separate published-release acceptance. This repairs the
newly changed sibling handoff, not a claim that its incoming artifacts have been delivered.
The former skill-contained utility source is withdrawn, not an alternative.

### Revision 1.2 record — 2026-09-13

Owner authorised all repairs from the peer review. This patch closes the eight document
findings by defining public-command installation, correct Hermes capability handling,
transaction recovery, file ownership, upstream release readiness, active-instance placement,
hook execution evidence, and truthful doctor states. It does not claim implementation or
external acceptance has run. Historical sealed head and review reports remain unchanged;
the current body and updated companion walk are the execution contract.

## 9. Residuals and leads
<!-- prd:residuals -->

Audience: the next agent.

- Capability IDs come from the shared contract, not independent row slugging. Unverified
  cells, including reader-to-agent delivery, remain unsupported until their own proof exists.
- ACP adapters remain the user's npm packages; a later PRD may let `assemble` install them.
- Categories J, T, H, R reuse REQ-2's path when their skills ship; no code change expected.

## 10. Execution notes
<!-- prd:execution-notes -->

Order: STP-1 → STP-2/3/4 (parallel) → STP-5 → STP-6 → STP-7. Fixtures: synthetic data only for narrow unit cases; gate-produced staged bytes and the shared four-unit declarations for consumer proof; actual published artifacts for final acceptance. The two evidence stages do not substitute for each other.

### Direct Execution Contract

| Field | Content |
|---|---|
| Change boundary | Baseline: the `sno-cli` working tree at the commit this PRD is released on, recorded by `git rev-parse HEAD` and `git status --porcelain` into `ai-docs/PRD/installer/sidecar/baseline.txt` before the first edit. Allowed paths: `src/cli.rs`, `src/doctor.rs`, new `src/assemble.rs`, `src/manifest.rs`, `src/harness_slots.rs`, `src/lib.rs` (module declarations only), `Cargo.lock` (required dependency resolution only), `tests/assemble*.rs`, `tests/fixtures/assemble/**`, `tests/cli.rs` (remove only the obsolete root-doctor retirement assertion; preserve other legacy checks), `README.md` (Commands section), `Cargo.toml` (dependencies only), `ai-docs/PRD/installer/sidecar/**` and `reports/**` under this PRD directory (baseline and proof records only). Non-goals: section 6. Final comparison: `git status --porcelain` (tracked, staged, untracked) diffed against the baseline; any path outside the list is reported, not absorbed. `openspec/**` is excluded and must be identical to baseline (`git diff --stat -- openspec/` empty; no untracked files there). |
| Implementation order | STP-1 first (verbs parse, exit codes, `--json`), then STP-2, STP-3, STP-4 in any order, then STP-5, STP-6, then STP-7. This is a new feature, not a correction: no causal RED is owed before product edits; each REQ's integration test is written red against the fixture before its code lands. |
| Causal test | Not applicable as a pre-edit RED (new feature, no defect being fixed); the causal proof is each `[QCG]` row's negative half (checksum altered → exit 3; hook write failure → exit 4 and recovery; `current` deleted → doctor fail), recorded under `ai-docs/PRD/installer/sidecar/<stem>.QCG-n.proof.json` by `prd-proof.py`; the E2E causal proof is the same QCG-9 journey with the wrong current link, then restoration. |
| Integration proof | `cargo test --test assemble` against gate-produced staged release bytes through the internal file-URL source fixture and the shared four-unit declarations in a temp `HOME`: real CLI entry, real files written, read back by the test from the manifest and the filesystem (QCG-1..8). |
| User journey | One change-affected primary flow: assemble → public init/spawn of both seats → verified seat exchange → remove, mapped to the end-to-end acceptance row QCG-9; its planted defect (`current` linked at the previous release) makes the same journey red and restoration green. Update and recovery use the same owned installation; their affected proof is QCG-5/6/8, plus the real scheduler invocation and focused harness discovery required above. |
| Negative proof | Checksum mismatch exits 3 with no installed-state changes (QCG-1); hook failure exits 4 with prior committed state restored or explicit pending recovery (QCG-5); malformed `agents.json` exits 4 (QCG-3); second `remove` exits 2 (QCG-8); `doctor` fails on a deleted `current` (QCG-7). |
| Completion | `cargo build --release` exit 0; `cargo test --test assemble` exit 0 with the row count printed; `cargo clippy -- -D warnings` exit 0; separate staged-consumer and published-release receipts; each QCG row run through `prd-proof.py` with receipts under `ai-docs/PRD/installer/sidecar/`; `git diff --stat -- openspec/` empty and no untracked files under `openspec/`; the final `git status --porcelain` compared with `baseline.txt`. Any nonzero or unexpected result blocks review and success. |
| Stop conditions | If upstream artifacts are absent, finish local fixture implementation and checks, record exact missing release evidence, and mark real-release acceptance blocked; never fabricate a release or invoke make in an absent source tree. Missing login/trust is an owner action; complete independent checks and request only that action. Recovery/conflict errors retain their durable records and identify the path to repair. An out-of-boundary implementation need is reported and resolved against the settled scope before editing; no automatic scope expansion. No blocked acceptance row can be called passed or delivery complete. |
