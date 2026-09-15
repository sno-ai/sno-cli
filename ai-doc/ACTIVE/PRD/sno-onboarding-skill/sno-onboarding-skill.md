---
name: "sno-onboarding-skill"
title: "Sno onboarding: the Sno skill and the sno onboarding command (one binary, two doors)"
version: "0.1"
prd_status: "draft"
project_status: "not_started"
updated: "2026-09-13"
owner: "larry"
pipeline: "mid"
---

# PRD — Sno onboarding: the Sno skill + `sno onboarding`

> **Why this PRD exists (owner 2026-09-13):** the launch entry point for Sno Station is not a package command. It is the user saying **"Sno onboarding"** inside any agent conversation, and the agent running the setup through the `sno` CLI. That ruling is from 2026-08-22 (`sno-strategy/sno/ai-doc/01-product-strategy-and-roadmap/sno-cli-strategy.md` §一: one binary, two doors; install ships with a Sno skill; skill is a first-class citizen). Nothing of it exists in code today (checked 2026-09-13: no skill in this repo, no `onboarding` subcommand, no unit in `sno-skills`, `M-onboarding-and-rollout` in `sno-station-skills` holds a README only). Launch is 2026-09-18; this is on the critical path.
> **Home:** this repo, because every line of logic lives in the binary and the skill ships inside it. `sno-station-skills/ai-doc/ACTIVE/PRD/cat-M/` carries a pointer, not a copy.
> **Strategy inputs, read before implementing:** `sno-cli-strategy.md` §一 (wake word, not personified), §一·三 (authorization: outside red lines, default allow), §二 4 (models judge, code decides); `sno-station-core/ai-doc/ACTIVE/git-launch/sno-station-launch-plan.md` §0 and §3.3 (what launch day must prove); `sno-strategy/.../00-foundation/harness-capability-map.md` (per-harness slots the skill declares).

## 1. Owner rulings and settled decisions
<!-- prd:owner-rulings -->

| # Ruling | Decided on | Decider | Rationale |
|---|---|---|---|
| [DEC-1] One binary, two doors: humans type `sno …`; agents use the Sno skill, which calls the same binary. No logic lives in the skill. | 2026-08-22 | Larry | Function implemented once; each harness needs only a thin skill file. |
| [DEC-2] "Sno" is a wake word, never a persona. Trigger phrases: "Sno onboarding", "Sno setup", "set up Sno Station" and synonyms. | 2026-08-22 | Larry | Same word in terminal, chat and voice; muscle memory from the first touch. |
| [DEC-3] **Thin CLI, fat skill (owner 2026-09-13, overrides the earlier "CLI asks, model relays").** The skill owns the conversation and the judgement: it infers what the user wants from context, asks about preferences in plain language, decides which harnesses to onboard — **including installing a harness the user does not have yet: default is not to install, but the skill must ask** — and plans the sequence. The CLI owns execution only: a set of idempotent primitives, each deterministic, each emitting JSON (`sno onboarding detect`, `install-sidecar`, `install-harness <name>`, `install-skin --harness <name>`, `mailbox-init`, `skills-install`, `verify`, `status`). Agent shells have no TTY, so the CLI never prompts; every choice arrives as an argument. Owner: "一切让大模型多做判断,不用拍脑门。" | 2026-09-13 | Larry | Judgement (what the user wants) is the model's; execution (what actually happens on disk) is code's — same principle, opposite cut from the first draft. |
| [DEC-4] Wave-1 scope of onboarding: detect the harnesses present (Claude Code, Codex, OpenClaw); **ask which to onboard and whether to install a missing one (default no)**; install or verify the memory sidecar; install the memory skin for each chosen harness; write each harness's hook / MCP configuration; initialise the mailbox and the callsign; install the skill library into each harness; run a round-trip proof; print the first act. | 2026-09-13 | Larry (launch plan §0; amended same day) | These are the launch-day pillars 1–3. |
| [DEC-5] Idempotent and re-entrant: running "Sno onboarding" twice is safe; each step reports `already done`, `done`, `skipped (why)` or `failed (why, fix)`. `sno doctor` is extended to check every piece onboarding installs. | 2026-09-13 | proposed | Re-runs are the normal repair path; a second-brain agent may run it on behalf of the first. |
| [DEC-6] Authorization inside onboarding follows the CLI red-line model: writing config files, installing skins and skills, starting the sidecar are default-allow; anything that deletes, overwrites a non-Sno file, or sends data off the machine asks first. | 2026-08-27 | Larry | Questions whose answer is always "yes" are noise. |
| [DEC-7] The skill is embedded in the binary and materialised by `sno skill install` into each harness's skill directory, with one overlay per harness (frontmatter / invocation differences only). Bootstrap for a machine with no `sno`: the README's one-line installer; the skill's first instruction is "if `sno` is not on PATH, run the installer, then continue". | 2026-09-13 | proposed | Kills the chicken-and-egg without a second distribution channel; registries (skills.sh, agentskills) list the same skill for discovery. |
| [DEC-8] Out of scope for this PRD: `sno assemble` (Duo declaration file), harness routing, Hosting, account/login flows. | 2026-09-13 | Larry (launch plan) | Launch day needs onboarding, not assembly. |

## 2. The problem, measured
<!-- prd:problem-measured -->

- The current human path is per-harness and per-package: OpenClaw users run `npx @snoai/mem-claw`; Claude Code and Codex users have no path at all (skins not built). There is no single first touch, and the first touch a user would see today exposes an internal package name, against the one-name rule (Sno Station only).
- The strategy has ruled onboarding as the first of three wake-word battlegrounds (`sno-cli-strategy.md` §一·二: first touch, highest-frequency pain, highest-value moment). Launch-day proof (`launch plan` §3.3) is written in terms of this command: "say Sno onboarding on each of three harnesses; memory round trip; mailbox round trip".
- Every harness in wave 1 reads `SKILL.md` and has a shell tool (harness-integration research, 2026-09-02), so a skill-driven install is zero-plugin-code on all three.

## 3. Scope

### 3.1 `sno onboarding` (new subcommand group) — primitives only, no conversation

```
sno onboarding detect --json                       # harnesses installed / running, paths, versions; sno pieces present
sno onboarding install-sidecar --json              # install or verify + start the memory sidecar; health
sno onboarding install-harness <name> --json       # install a harness the user asked for (claude-code | codex | openclaw)
sno onboarding install-skin --harness <name> --json  # memory skin + hook / MCP config for that harness
sno onboarding mailbox-init [--callsign <h>=<cs>] --json
sno onboarding skills-install [--harness <name>] --json   # skill library + the Sno skill, per-harness overlay
sno onboarding verify --from <h> --to <h> --json   # memory round trip + mailbox round trip between two harnesses
sno onboarding status --json                       # what is installed, what is not, what changed since last run
sno onboarding run --harness a,b [--install-missing] --yes   # human shortcut: the default sequence with defaults
```

Every primitive: idempotent (`already done` / `done` / `skipped` / `failed` + `fix`), never prompts, validates its arguments, refuses anything on the red-line list unless `--confirm` is present (DEC-6). No primitive knows about the others; sequencing is the skill's job (and `run` for humans).

### 3.2 The Sno skill — fat: the conversation, the judgement, the plan

- `SKILL.md` with frontmatter `name: sno`, description carrying the trigger phrases (DEC-2). Body teaches the model to:
  1. **Read the situation first** (`detect`), then infer intent from what the user said and what is on the machine ("you have Claude Code and Codex; Codex is running; no Sno pieces yet — I'll set up both, shared memory and a mailbox between them. Want OpenClaw too? It isn't installed; I can install it or skip.").
  2. **Ask preferences in plain words**, few questions, defaults stated; never install a missing harness without asking; never answer for the human.
  3. **Plan the sequence** from the primitives, run them one by one, narrate each JSON result in one line, stop and explain on `failed` with the CLI's `fix`.
  4. Finish with the first act (roster, callsigns, one runnable example) and offer `verify`.
- Per-harness overlays: Claude Code (`.claude/skills/sno/`), Codex (`~/.codex/skills/sno/` + `agents/openai.yaml`), OpenClaw (`~/.agents/skills/sno/`). Body identical; only frontmatter / invocation metadata differs.
- `sno skill install [--harness <name>]` materialises the embedded skill; `sno skill status` reports drift.
- Requirements declaration (for the M category / capability map): shell, file write, skill directory; hook or MCP is a per-harness slot, not a hard requirement.

### 3.3 Not in scope

`sno assemble`, routing, Hosting login, account creation, any UI.

## 4. Acceptance (three-layer proof, PM to expand)

1. **Unit:** every primitive runs to `done` on a fixture machine with all three harnesses; every `--json` output validates against one schema; a second run yields `already done` for every primitive; `install-harness` on a present harness is a no-op; a red-line action without `--confirm` is refused.
2. **Integration (launch plan §3.3):** on a clean profile for each harness, the human types "Sno onboarding" in the agent; the agent runs the machine; memory round trip Claude Code ↔ Codex; mailbox round trip; OpenClaw `/memory status` sees the same store. Transcript kept as the launch-day receipt.
3. **Proof it ran:** `sno onboarding status` after the run lists every installed piece with its path and version; `sno doctor` green.

Gates: README first screen shows only `sno` and "Sno onboarding" (no npm package names); the skill installs on all three harnesses with the same body; no step requires a plugin.

## 5. Open questions for the owner

- Skin package names for Claude Code and Codex (internal; the user never types them).
- Fallback wording if a skin is not green on 2026-09-16 (launch plan §7 d).
