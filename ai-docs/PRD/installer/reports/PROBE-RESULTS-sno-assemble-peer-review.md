# Probe results: installer PRD peer review

Scope: local working copies on the host recorded below; no deployed behavior or remote release availability is inferred from these probes. No product code was edited and no tests were run.

Normal use paths to review: first assemble on an existing agent machine; repeat assemble unchanged; daily update with existing manifest; update fails after writing Reach or skills; doctor then remove; clean-account install followed by Reach spawn/send. Inputs are the published directory/config contracts below, including pre-existing skill directories and hook entries.

## Command

```sh
hostname; git rev-parse HEAD; git status --short
```

```text
gpt1
9d8adc8bd9f6527cedac4a39759b7f1501df9ff1
A  ai-docs/PRD/installer/sidecar/sno-assemble-PRD.acceptance.json
A  ai-docs/PRD/installer/sidecar/sno-assemble-PRD.walk.json
?? ai-doc/ACTIVE/PRD/sno-onboarding-skill/
?? ai-docs/PRD/installer/reports/PROBE-RESULTS-sno-assemble-peer-review.md
?? ai-docs/PRD/installer/sidecar/sno-assemble-PRD.identifier-repair-plan.json
?? ai-docs/PRD/installer/sidecar/sno-assemble-PRD.identifier-repair-plan.md

exit=0
```

## Command

```sh
sed -n '1,58p' src/cli.rs; sed -n '265,315p' src/cli.rs; cat src/lib.rs; cat Cargo.toml
```

```text
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};
use serde_json::{Value, json};

use crate::doctor;
use crate::error::CliError;
use crate::export::{self, ExportFormat};
use crate::service;
use crate::state::{self, ConsentValue};

#[path = "rem.rs"]
mod rem;

const RETIRED_ROOT_COMMANDS: &[&str] =
    &["consent", "observe", "register", "claim", "audit", "doctor", "starport"];

#[derive(Debug, Parser)]
#[command(
    name = "sno",
    version,
    about = "The unified command-line interface for SNO",
    long_about = "The unified command-line interface for SNO\n\nAdditional commands can be installed as external subcommands named sno-<command>."
)]
struct SnoCli {
    #[arg(long, global = true, help = "Emit stable JSON output")]
    json: bool,

    #[command(subcommand)]
    command: Option<RootCommand>,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
    #[command(about = "Manage account and machine identity")]
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },
    #[command(name = "station", about = "Manage this SNO Station")]
    SnoStation {
        #[command(subcommand)]
        command: SnoStationCommand,
    },
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

#[derive(Debug, Subcommand)]
enum AccountCommand {
    #[command(about = "Manage this machine's account identity")]
    Machine {
        #[command(subcommand)]
        command: MachineCommand,
    },
            let output_path = path.or(out);
            let export_format = format.as_deref().map(parse_export_format).transpose()?;
            if json_enabled
                && output_path.is_none()
                && matches!(export_format, Some(ExportFormat::Jsonl | ExportFormat::Csv))
            {
                return Err(CliError::usage(
                    "JSON mode requires an output path for jsonl or csv export",
                ));
            }
            export::run(output_path, export_format, json_enabled)
        }
    }
}

fn dispatch_external(arguments: Vec<OsString>) -> Result<i32, CliError> {
    let (name, child_arguments) = arguments
        .split_first()
        .ok_or_else(|| CliError::usage("missing external subcommand"))?;
    let name = name
        .to_str()
        .ok_or_else(|| CliError::usage("external subcommand name is not valid UTF-8"))?;
    if RETIRED_ROOT_COMMANDS.contains(&name) {
        return Err(CliError::usage(format!(
            "'{name}' is not a top-level command; run 'sno --help'"
        )));
    }
    let executable = format!("sno-{name}");
    match Command::new(&executable).args(child_arguments).status() {
        Ok(status) => Ok(status.code().unwrap_or(1)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(CliError::usage(format!(
            "unknown command '{name}'; external executable '{executable}' was not found on PATH"
        ))),
        Err(error) => Err(CliError::runtime(
            "external_command_failed",
            error.to_string(),
        )),
    }
}

fn print_missing_command(json_enabled: bool) -> i32 {
    if json_enabled {
        let _ = print_json(&json!({ "error": "usage_error", "message": "missing command" }));
    } else {
        let mut command = SnoCli::command();
        let _ = command.print_help();
        println!();
    }
    2
}

mod cli;
mod doctor;
mod error;
mod export;
mod rem_outcome;
mod service;
mod state;

pub use cli::run;
[package]
name = "sno"
version = "0.1.7"
edition = "2024"
rust-version = "1.85"
description = "The unified command-line interface for SNO"
license = "Apache-2.0"
repository = "https://github.com/sno-ai/sno-cli"
homepage = "https://www.sno.ai"
readme = "README.md"
keywords = ["sno", "cli", "telemetry", "observability"]
categories = ["command-line-utilities"]
include = [
  "src/**",
  "Cargo.toml",
  "Cargo.lock",
  "README.md",
  "LICENSE",
]

[[bin]]
name = "sno"
path = "src/main.rs"

[dependencies]
chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }
clap = { version = "4.5", features = ["derive"] }
flate2 = "1.0"
fs2 = "0.4"
hex = "0.4"
rand = "0.9"
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0", features = ["preserve_order"] }
sha2 = "0.10"
tar = "0.4"
url = "2.5"
uuid = { version = "1.11", features = ["v7"] }

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = ["Win32_Storage_FileSystem"] }

[dev-dependencies]
csv = "1.3"
tempfile = "3.14"

# The profile that 'dist' will build with
[profile.dist]
inherits = "release"
lto = "thin"

exit=0
```

## Command

```sh
sed -n '1,100p' src/doctor.rs; sed -n '135,155p' src/doctor.rs; rg -n 'Install|releases|sha256' README.md
```

```text
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use fs2::{FileExt, lock_contended_error};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::cli::print_json;
use crate::error::CliError;
use crate::state::{ConsentValue, Identity, SnoPaths, is_valid_identity};

#[derive(Serialize)]
struct DoctorReport {
    identity: DoctorCheck,
    buffer: DoctorCheck,
    consent: DoctorCheck,
    last_ship: DoctorCheck,
    lockfile: DoctorCheck,
}

#[derive(Serialize)]
struct DoctorCheck {
    name: &'static str,
    status: CheckStatus,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

pub fn run(json_enabled: bool) -> Result<i32, CliError> {
    let paths = SnoPaths::from_environment()?;
    let (buffer, shipped_count) = check_buffer(&paths.buffer_path);
    let report = DoctorReport {
        identity: check_identity(&paths.identity_path),
        buffer,
        consent: check_consent(&paths.consent_path),
        last_ship: check_last_ship(shipped_count),
        lockfile: check_lockfile(&paths.identity_lock_path),
    };
    let has_issue = report.identity.status != CheckStatus::Ok
        || report.buffer.status != CheckStatus::Ok
        || report.consent.status != CheckStatus::Ok
        || report.last_ship.status != CheckStatus::Ok
        || report.lockfile.status != CheckStatus::Ok;
    if json_enabled {
        print_json(&serde_json::to_value(&report)?)?;
    } else {
        for check in [
            &report.identity,
            &report.buffer,
            &report.consent,
            &report.last_ship,
            &report.lockfile,
        ] {
            let badge = match check.status {
                CheckStatus::Ok => "[ok]",
                CheckStatus::Warn => "[warn]",
                CheckStatus::Fail => "[fail]",
            };
            println!("{badge} {}", check.detail);
        }
    }
    Ok(if has_issue { 1 } else { 0 })
}

fn check_identity(path: &Path) -> DoctorCheck {
    let _metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return DoctorCheck {
                name: "identity",
                status: CheckStatus::Warn,
                detail: "identity not bootstrapped - first SDK use or `sno account machine register` will create it automatically".to_owned(),
                path: Some(path.display().to_string()),
            };
        }
        Err(error) => return failed_path("identity", path, error.to_string()),
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return failed_path("identity", path, error.to_string()),
    };
    let parsed = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(parsed) => parsed,
        Err(_) => {
            return DoctorCheck {
                name: "identity",
                status: CheckStatus::Fail,
                detail: format!("identity is not readable JSON at {}", path.display()),
                path: Some(path.display().to_string()),
            };
        }
            };
        }
    }
    DoctorCheck {
        name: "identity",
        status: CheckStatus::Ok,
        detail: "identity present (anonymous machine)".to_owned(),
        path: Some(path.display().to_string()),
    }
}

fn check_buffer(path: &Path) -> (DoctorCheck, i64) {
    if !path.exists() {
        return (
            DoctorCheck {
                name: "buffer",
                status: CheckStatus::Warn,
                detail: format!("buffer not initialized at {}", path.display()),
                path: Some(path.display().to_string()),
            },
            0,
7:## Install
15:Install from crates.io when a Rust toolchain is available:

exit=0
```

## Command

```sh
git -C /home/lh/code/sno-station-core rev-parse HEAD; git -C /home/lh/code/sno-station-skills rev-parse HEAD; ls -ld /home/lh/code/sno-station-core/apps/reach; rg --files /home/lh/code/sno-station-skills/skills; rg -n 'reach|tar.gz|sha256' /home/lh/code/sno-station-core/.github/workflows/*
```

```text
b1f4a845c1a91288da4ad56eabfbcf6280c98227
a263ccd1f08a413dafab788d0ba255b671de2b40
ls: cannot access '/home/lh/code/sno-station-core/apps/reach': No such file or directory
/home/lh/code/sno-station-skills/skills/T-delegation-and-work-orders/README.md
/home/lh/code/sno-station-skills/skills/M-onboarding-and-rollout/README.md
/home/lh/code/sno-station-skills/skills/H-reporting-and-visibility/README.md
/home/lh/code/sno-station-skills/skills/J-peer-review-and-audit/README.md
/home/lh/code/sno-station-skills/skills/S-communication-and-handoff/README.md
/home/lh/code/sno-station-skills/skills/R-recursive-self-improvement/README.md

exit=1
```

## Command

```sh
cat /home/lh/code/sno-station-core/ai-doc/ACTIVE/PRD/reach/reach-naming-contract.md
```

```text
# Reach — the public naming contract (owner-approved 2026-09-12; peer-reviewed once, four repairs applied)

Reach is the program that lets one AI agent reach another on the same team: talk to it now,
leave it a card for later, ring it awake, and know where it sits. It is shipped in
`sno-station-core/apps/reach` and invoked as `sno reach <verb>` (binary `sno-reach` on PATH;
the `sno` CLI already dispatches `sno <x>` to `sno-<x>`).

Audience for every name below: a developer who has never seen this system. The test for each
name: reading it, they know what it means and what it will do. Names are an API; once shipped
they do not change.

## Principle

Verbs are named by the ACT the user wants, never by the transport. Whether a seat is reached
over an ACP session or a tmux pane is decided once, when the seat is created and registered.
No verb that talks to a seat takes a transport flag.

Two acts:
- `call` — talk now and wait for the answer (real-time).
- `send` — leave a card and ring; the seat answers when it processes it (non-real-time).

## Vocabulary

- seat — an agent that can be reached. Has an address, and a registered channel.
- address — `<role>.<name>@<host>`. Each part is lowercase letters, digits and hyphens; role 1–32 characters, name 1–64, host 1–253 (the rule the code enforces today). Role is chosen by the team (executor, lead, owner…); Reach does not define roles.
- card — one message in a seat's inbox (RFC 5322 text). Has a type and, for work, a state.
- ring — waking a seat so it reads its inbox.
- channel — how a seat is reached: `acp` (ACP session), `tmux` (pane), `orca`.

## Commands: `sno reach <verb>`

| Group | Verb | Meaning | Replaces (workshop today) |
|---|---|---|---|
| Seats | `spawn <agent> [--window]` | create a seat; ACP by default, `--window` makes a tmux pane | spawn-acp.sh, seat half of spawn-exec.sh |
| Seats | `register` / `unregister` | this seat says where it is / leaves | mbox register, mailbox-doorbell.sh register |
| Seats | `seats` | list seats, where each is, whether it is live | locate-agent-window.sh, mbox-reachability status |
| Now | `call <seat> "<text>"` | deliver text into the live seat and print what the seat says next, until `--timeout` seconds; `--expect <regex>` names text that must appear; exit 0 the seat answered, 3 the runtime refused, 4 nothing came back in time, 5 the seat could not be read. Same contract on every channel (ACP verifies by a receipt token, tmux by new screen text). Not a request/response RPC. | message-agent-window.sh, message-acp.sh |
| Now | `watch <seat>` | follow a seat's live output read-only | ACP read-only follow |
| Now | `ring <seat>` | ring only, no card | mailbox-doorbell.sh doorbell |
| Later | `send` | deliver a card and ring, one step; `--no-ring` for tooling | mbox send + mailbox.sh send (two layers collapse into one) |
| Later | `reply` | answer a card; `--state accepted|completed|failed` for work | mbox reply |
| Later | `inbox` | cards this seat must act on; `--cc` shows informed copies | peek, informed |
| Later | `wait` | block until a card arrives; `--reply-to <id>` waits for the answer to one card, `--from <address>` for one sender; `--timeout`, `--every`, `--idle` in seconds | wait, standby (folded) |
| Later | `dismiss` / `log` / `state` / `flush` | clear a cc copy / history / work-item states / retry outbox | same names |
| Care | `init` | create this seat's inbox and identity; `--name` sets the display name once | init + claim (folded) |
| Care | `rebind` | move this seat to the current machine | rebind |
| Care | `doctor` / `export` / `lint` | check a seat / export one work item to an mbox file / check a card file | same names |

Removed entry points: `mailbox.sh`, `mailbox-doorbell.sh`, bare `mbox`, `mbox-wake`,
`mbox-reachability`, `mbox-transport`, `mbox-deliver`, `mbox-lint` — all become internal
under the program's lib; only `sno-reach` is on PATH.

## Other user-visible names

| Kind | Decision |
|---|---|
| Program dir | `sno-station-core/apps/reach` |
| Install path | `~/.local/lib/sno-reach/releases/<version>` and `~/.local/lib/sno-reach/current` |
| State root | `~/.local/state/sno-reach/<address>/` (Maildir `new/ cur/ tmp/` plus `reachable.json`, `seat.json`) |
| Config | `~/.config/sno-reach/agents.json` — which ACP adapter serves each agent kind |
| Env vars | `SNO_REACH_ROOT` (state root), `SNO_REACH_ADDR` (this seat's address); every `SNO_MBOX_*` becomes `SNO_REACH_*` |
| Seat files | `reachable.json` (kept), `seat.json` (was card.json) |
| Card headers | `X-Type`, `X-State`, `X-Tag`, `X-No-Reply` kept; `X-Journey` → `X-Work` and `X-Callsign` → `X-Name` (journey and callsign are internal vocabulary; the header carries the display name set by `init --name`) |
| X-Type values | question, decision, answer, info, status, done, cancel |
| X-State values | accepted, running, requires-action, completed, failed, cancelled, refused |
| Channels | `acp`, `tmux`, `orca`; `--window` is the only spelling of "give it a tmux pane" |
| Skill names (category S) | one skill `reach` (when to use which verb; replaces agent-window-locate, agent-window-direct-message, and the mailbox skill text); `agent-cutover` → `handoff`; `heartbeat` and `subscription-quota-check` unchanged |
| Hook | the progress reminder hook is installed by the installer; no user-visible name |

## Flags — one word, one meaning, everywhere

| Flag | Meaning | Replaces |
|---|---|---|
| `--as <address>` | act as this seat | `--as` |
| `--name <display name>` | the seat's display name; given once at `init`, carried automatically afterwards | `--callsign` on every call |
| `--work <id>` | the work item (matches header `X-Work`) | `--journey` |
| `--card <path>` | the card file to answer or dismiss | `--message` |
| `--state <accepted\|completed\|failed>` | the work state a reply carries | `--state` |
| `--reason <text>` | why a card is dismissed or a work item failed/cancelled | `--reason` |
| `--reply-to <message-id>` | wait for the answer to this card | `--reply-to` |
| `--from <address>` | only cards from this sender | `--from` |
| `--timeout <seconds>` / `--every <seconds>` / `--idle <seconds>` | wait at most / check every / stop after this long with nothing new | `--timeout-secs` / `--interval-secs` / `--idle-secs` |
| `--channel <acp\|tmux\|orca>` / `--handle <locator>` | where this seat is reached | same |
| `--window` / `--cwd <dir>` | give the seat a tmux pane / its working directory | same |
| `--cc` | show informed copies instead of action cards | (was `informed`) |
| `--no-ring` | deliver without waking (tooling only) | (was bare `mbox send`) |
| `--expect <regex>` | text the seat's output must contain for `call` to count as answered | `--expect` |
| `--output <file>` | where `export` writes | `--output` |
| `--json` | machine-readable output on read verbs | new |

Not public: `--migration`, `--machine-id`, `--owns`, `--runtime`, `--supervisor`, `--root`,
`--evict` (internal or set by the installer / the TPM launcher). TPM-only flags (`--journey`,
`--budget-h`, `--dispatch`, `--class`, `--lane`, `--fence`, `--resume`) stay on the TPM's own
launcher, which calls `sno reach spawn` underneath.

## Verified on 2026-09-12
- `sno` dispatches `sno <x>` to `sno-<x>` on PATH with arguments and exit code intact
  (probe: a stub `sno-reach` returning 42 printed its arguments and `sno reach probe --x 1` exited 42).
- Address rule from `mbox-reachability`: `^[a-z][a-z0-9-]{0,31}\.[a-z0-9][a-z0-9-]{0,63}@[a-z0-9][a-z0-9.-]{0,252}$`.
- `call` behaviour measured in `message-acp.sh` (receipt token, `verified` flag) and
  `message-agent-window.sh` (screen delta since cursor, `--expect`, exit 0/3/4/5).

exit=0
```

## Command

```sh
sed -n '1,25p' /home/lh/code/sno-station-core/ai-doc/ACTIVE/PRD/reach/reach-PRD.md; sed -n '190,245p' /home/lh/code/sno-station-core/ai-doc/ACTIVE/PRD/reach/reach-PRD.md
```

```text
---
name: "reach"
title: "Reach: one program for agent-to-agent communication"
version: "1.2"
prd_status: "released"
project_status: "not_started"
updated: "2026-09-13"
owner: "larry"
pipeline: "mid"
---

# PRD — Reach: one program for agent-to-agent communication

Reach is the program that lets one AI agent on a team reach another: talk to it now, leave it
a card for later, ring it awake, and know where it sits. This PRD moves the communication
program out of the private workshop into this repository as `apps/reach`, under one name, one
version and one command, `sno reach <verb>`. The public names are fixed by
`ai-doc/ACTIVE/PRD/reach/reach-naming-contract.md` (owner-approved 2026-09-12); this PRD
does not restate them, it binds them.

Sibling documents: the skill-text publish PRD is `ai-doc/ACTIVE/PRD/cat-S/s-category-publish-PRD.md` in the `sno-station-skills` repository; the install/update PRD is `ai-docs/PRD/installer/sno-assemble-PRD.md` in the `sno-cli` repository. This PRD ships the program those two consume.

## 1. Owner rulings and settled decisions
<!-- prd:owner-rulings -->


Same shape and validation as F6. A kind present is ACP-capable; absent is not. The installer
writes it; `spawn` reads it; nothing else does.

### 5.4 Disposition table

| Input or failure class | Fate |
|---|---|
| Known verb, valid flags | Runs; exit codes per REQ |
| No verb / unknown verb | Usage table, exit 64 |
| Old flag from DEC-4 | Refused, exit 64, one stderr line naming the new flag |
| Old environment variable set (`SNO_MBOX_*`, `SNO_EXECUTOR_ADDR`) | Ignored; `doctor` warns once |
| Old header `X-Journey` or `X-Callsign` on a card | `send`/`reply`/`lint` refuse naming `X-Work` or `X-Name` |
| Old state root present (`~/.local/state/agent-mbox`) | Ignored; `doctor` reports it as `legacy state present, not read` |
| Address not matching F2 | Refused naming the rule |
| `send` to an unregistered `To` seat | Refused before any write; prints the register command |
| `send` to a registered seat whose channel is dead | Delivered; ring reports `rang-unverified`/`failed`; retry per today's wake retry; exit 5 semantics as today |
| `call` on `acp` / `tmux` | Same contract; exit 0/3/4/5 |
| `call` on `orca` | Exit 3 with "channel orca does not support call" (no live-prompt path exists) |
| `spawn` with malformed `agents.json` | Refused naming the file; nothing created |
| `spawn` kind not in `agents.json`, no `--window` | tmux seat; stdout says the kind is not ACP-capable |
| `reply --state completed` before `accepted` | Refused; prints the acceptance command |
| Stateless `reply` on an unaccepted work item | Refused; prints the acceptance command |
| Plain `reply` on a question that is not a work item | Delivered |
| Personal binding inside `apps/reach/` | Test run fails naming file:line |
| Wake-attempt store with thousands of terminal files | Under 1 s per verb start |

### 5.5 Dead designs

| What | Why killed | Who | When |
|---|---|---|---|
| Names Switchboard, Front Desk, Intercom, Comms, Ping, Relay, Page, Hail | Owner rejected; Reach chosen | larry | 2026-09-12 |
| `sno reach realtime` / `sno reach async` verb split | Names a transport, contradicts inherited ruling 8 | larry | 2026-09-12 |
| Surface-only rename, keep `sno-agent-mbox` paths | Owner chose full rename | larry | 2026-09-12 |
| Two send entry points (bare + wrapper) with a notice on the bare one | Root cause of the silent no-wake delivery; folded into one `send` | this PRD | 2026-09-12 |
| Program as skill scripts copied per harness | Measured drift | larry | 2026-09-12 |

DO NOT REOPEN any row above without a named new fact.

### 5.6 Ordered build steps

- `[STP-1]` Create `apps/reach/` from the workshop runtime (`bin/`, `lib/`, `vendor/`, `spec/`, `skill/agent-mailbox.md` → `guide/`, `t/` → `tests/apps/reach/`, `VERSION` set to `2.0`, `NOTICE`, `Makefile`), renaming binary, paths, environment variables, headers and flags per DEC-2..4, DEC-8..9 and REQ-4, REQ-19, REQ-20. Prerequisite: none. Maps: REQ-1, REQ-2, REQ-3, REQ-4, REQ-19, REQ-20.
- `[STP-2]` Fold the wrapper layer in: `send` = deliver + ring (DEC-5, REQ-5), `ring` (REQ-8), `register`/`unregister`/`seats` from the doorbell and reachability code (REQ-10), `remind` from the progress reminder (REQ-16). Delete the no-wake notice. Prerequisite: STP-1. Maps: REQ-5, REQ-8, REQ-10, REQ-16.
- `[STP-3]` Fold the seat scripts in: `spawn` from the ACP spawn script plus the tmux seat half of the launcher (REQ-9), `call` and `watch` from the direct-message and ACP message scripts (REQ-6, REQ-7), `seats` from the locate script. Prerequisite: STP-2. Maps: REQ-6, REQ-7, REQ-9.
- `[STP-4]` Fold `standby` into `wait`, `claim` into `init --name`, `peek`/`informed` into `inbox`/`inbox --cc`; keep the acceptance gate (REQ-11..15). Prerequisite: STP-1. Maps: REQ-11, REQ-12, REQ-13, REQ-14, REQ-15.
- `[STP-5]` Add the personal-binding gate and the per-verb start-time check to `tests/apps/reach/run.sh` (REQ-17, REQ-18). Prerequisite: STP-1. Maps: REQ-17, REQ-18.
- `[STP-6]` `make install` and `sno reach --version`/`doctor` (REQ-1, REQ-2). Prerequisite: STP-1..5. Maps: REQ-1, REQ-2.
- `[STP-7]` End-to-end journeys of section 7 on the installed `current`, with the planted defects. Prerequisite: STP-6. Maps: every `[E2E]` row.
- `[STP-8]` Write the workshop follow-on work order (S3): switch the TPM skill's callers to `sno reach`, delete `src/agent-mbox`, `deploy-agent-mbox.sh`, the wrapper scripts and the two S-skill script directories from the workshop; record it in section 10. Prerequisite: STP-7. Maps: none (out-of-repo consequence).

## 6. Non-goals and fences
<!-- prd:non-goals -->

- NOT building: an installer (`sno assemble`/`sno update`) — sibling PRD in `sno-cli`; the
  agent-facing skill text — sibling PRD in `sno-station-skills`; a new transport; a web UI; a daemon.
- NOT changing: the card format beyond `X-Journey` → `X-Work`; the type/state vocabulary; the

exit=0
```

## Command

```sh
sed -n '1,160p' /home/lh/code/sno-station-skills/ai-doc/ACTIVE/PRD/cat-S/s-category-publish-PRD.md
```

```text
---
name: "s-category-publish"
title: "Publish the Communication & Handoff skills (category S)"
version: "1.2"
prd_status: "released"
project_status: "not_started"
updated: "2026-09-13"
owner: "larry"
pipeline: "mid"
---

# PRD — Publish the Communication & Handoff skills (category S)

This PRD ships category S of the skill library: the skill text that tells an agent when and
how to use the Reach program (`sno reach <verb>`), how to hand work to another agent, how to
arm a wake-up, and how to read subscription quota. Programs are not shipped here; they ship in
`sno-station-core` (the Reach program PRD, `ai-doc/ACTIVE/PRD/reach/reach-PRD.md` in that
repository, released 2026-09-13, with its naming contract beside it). Installing both is the
`sno` CLI's job (installer PRD in the `sno-cli` repository, `ai-docs/PRD/installer/`).

## 1. Owner rulings and settled decisions
<!-- prd:owner-rulings -->

| # | Date | Decider | Ruling |
|---|---|---|---|
| 1 | 2026-09-10 | larry | Two public homes only: programs to `sno-station-core`, skill text to this repository. Nothing here is authored in place; `scripts/promote.sh` is the only writer of `skills/**`. |
| 2 | 2026-09-12 | larry | Every communication script (mailbox runtime, wrapper layer, seat/locate/message scripts) is one program, Reach, in `sno-station-core`. Category S here carries text only. |
| 3 | 2026-09-12 | larry | Public names are the Reach naming contract: verbs `spawn register unregister seats call watch ring send reply inbox wait dismiss log state flush init rebind doctor export lint remind`, flags `--as --name --work --card --state --reason --reply-to --from --timeout --every --idle --channel --handle --window --cwd --cc --no-ring --expect --output --json`, headers `X-Work` and `X-Name`. Old names (`mbox`, `mailbox.sh`, doorbell, `--journey`, `--callsign`, `X-Journey`, `X-Callsign`) never appear in shipped text. |
| 4 | 2026-09-12 | larry | All install and update goes through the `sno` CLI; nothing here installs itself. |
| 5 | 2026-09-10 | larry | Category M is mostly not skills: the only workshop-side M work is the requirements declaration each skill carries so `sno assemble` can decide install / degrade / skip. |

Decisions taken under standing authority:

| # | Date | Decision | Why no escalation |
|---|---|---|---|
| S1 | 2026-09-13 | Category S ships four units: `reach` (new, text only — when to use which verb), `handoff` (the workshop's `agent-cutover`, renamed, text only), `heartbeat` and `subscription-quota-check` (text plus one small command each under `public-bin/`). The workshop's `agent-window-locate` and `agent-window-direct-message` are absorbed into `reach` and are not promoted. | Follows rulings 2–3 directly; the two absorbed units are 1,861 lines of script and 360 lines of text, and the text is what `reach` keeps. |
| S2 | 2026-09-13 | A unit whose only executable is one small command under `public-bin/` (heartbeat 96 lines, quota check) stays a skill; the installer places the command. | Ruling 2 named the communication program; these two are wake-up and quota tools, not communication, and each is under 100 lines. Veto window open. |
| S3 | 2026-09-13 | The gate gains two checks: every unit must ship at least one self-test (a text-only unit's self-test checks its own vocabulary), and every `SKILL.md` must carry a parseable `requires:` block. | The gate today passes a unit with zero self-tests vacuously (measured: `agent-cutover`, 0 self-tests, "passes the gate"); a gate that can pass on zero work is the failure the repository's own rules name. |
| S4 | 2026-09-13 | This PRD is the category PRD; each unit's "what changes for the public version" is a subsection of section 5 and serves as that unit's short PRD. | Four separate documents for four dogfood ports would repeat the same rulings four times. |
| S5 | 2026-09-13 | This repository is public; the document cites workshop evidence by document name and date only. | The gate's own binding patterns apply to documents. |

## 2. The problem, measured
<!-- prd:problem-measured -->

Measured 2026-09-12 in this repository and the workshop (`sno-skills`, private).

**Nothing has shipped.** `skills/S-communication-and-handoff/` holds one README and no unit;
every workshop unit is tier `candidate` (`registry.yaml`: 30 candidate rows, 0 shipped).

**The gate's verdict on the five S candidates today** (`scripts/promote.sh --dry-run`,
workshop at its 2026-09-12 head):

| Unit | Verdict | Why |
|---|---|---|
| `agent-window-locate` | passes | 1 self-test green on the copy |
| `subscription-quota-check` | passes | 2 stages green |
| `agent-cutover` | passes **with 0 self-tests** | nothing to run, so nothing fails — a vacuous pass |
| `heartbeat` | refused | its self-test looks for the `heartbeat` command on PATH and finds the owner's private build, not the copy: "heartbeat on PATH is a different build … public-command stages skipped", 2 of 2 stages failed |
| `agent-window-direct-message` | not run | absorbed by Reach (S1) |

**Shipped text would still teach old names.** The two absorbed units' text (360 lines) and
the TPM skill's hook reference name `mailbox.sh`, `mbox`, the doorbell script, `--journey`,
`--callsign` and `X-Journey`; the gate found 5 personal bindings in the TPM skill's hook
reference alone (`~/.codex/skills/`, `~/.claude/skills/`, an internal host name). None of that
text can ship as is.

**No unit declares what it needs.** Zero `SKILL.md` files in the workshop carry a requirements
block; `skills/M-onboarding-and-rollout/README.md` says the declaration is what `sno assemble`
reads, and defines no field.

**The library has no LICENSE or NOTICE file at its root** (`ls` of the repository root,
2026-09-12), although the license is settled as Apache-2.0.

What already works (workshop dogfood, 2026-09-11, four agent kinds): the journeys the `reach`
text must describe — spawn a seat, send a card with a ring, accept, complete; `--window`
seats; real-time call on both channels. Recorded in the workshop's
`ai-doc/ARCHIVE/PRD-done/acp-v1-communication/`.

Falsified: "promote the five units as they are" — two cannot pass, one passes on nothing, two
teach dead names.

## 3. Verified facts the executor must not re-derive
<!-- prd:verified-facts -->

| # | Fact | Source | Date |
|---|---|---|---|
| F1 | The gate refuses a unit unless: it is in the workshop `registry.yaml` with tier `candidate` or `shipped`; its payload has no personal binding (patterns: `/home/<name>`, `~/code/`, `/mnt/ramdisk`, `~/.claude|.codex|.agents/skills/`, the owner's name); every `scripts/selftest/run-selftest.sh` it ships is green on the copy; then it writes `PROMOTED.json` last. | `scripts/promote.sh` header and lines 53–60 | 2026-09-12 |
| F2 | `PROMOTED.json` fields: `unit category tier split source{repo,commit,unit_tree_clean} selftests_run promoted_at`. | `scripts/promote.sh` line 221 | 2026-09-12 |
| F3 | `scripts/promote.sh --selftest` proves the gate still bites on temp trees; the README requires running it after any change to the gate. | `README.md`, `scripts/promote.sh` usage | 2026-09-12 |
| F4 | The harness capability map has seven perspectives: 1 Launch, 2 Input, 3 Observation, 4 Action, 5 Install surface, 6 Identity and billing, 7 Place in a squad; rows are named in bold in each table (e.g. "Type into a tmux pane", "Pre-turn context injection, pure config", "Hook event count"). Fourteen cells are marked unverified. | `ai-doc/RESEARCH/harness-capability-matrix/harness-capability-matrix.md` | 2026-09-12 |
| F5 | A workshop unit's payload shapes: `skills/<unit>/skill/` (shared), `skills/<unit>/skills/<name>/` (family), `overlay/claude|codex/`, `public-bin/<name>`; every payload has `SKILL.md` with `name` and `description` frontmatter. | workshop `CLAUDE.md` "Working on skills" | 2026-09-12 |
| F6 | `heartbeat`'s self-test has two stages, `skill-copy-hook` and `skill-copy-registry`, and a "public-command" stage group it skips when the `heartbeat` on PATH is not the copy's build. | gate dry-run output, 2026-09-12 | 2026-09-12 |
| F7 | `agent-cutover` is 2 files (SKILL.md 95 lines, no scripts); `heartbeat` 11 files (453 lines of text, 96 of script); `subscription-quota-check` 15 files. | `find`/`wc` over the workshop units | 2026-09-12 |
| F8 | The Reach program ships its own agent-facing guide (`guide/`) and `sno reach --help` prints the verb table; the `reach` skill text therefore need not restate verb syntax. | `sno-station-core` `ai-doc/ACTIVE/PRD/reach/reach-PRD.md` REQ-1, REQ-2 | 2026-09-13 |
| F9 | Claude Code and Codex both inject pre-turn context from a hook that prints `{"hookSpecificOutput":{"additionalContext":…}}`; the workshop's progress-reporting hook reference gives the exact entries. | capability map perspective 2, "Pre-turn context injection"; workshop `skills/tpm/skills/tpm/references/progress-reporting-hooks.md` | 2026-09-12 |

## 4. Actors and handoff boundaries
<!-- prd:actors-handoffs -->

| Actor | Touches this how | Succeeds when |
|---|---|---|
| The workshop (private, `sno-skills`) | Authors the four units, their self-tests and `requires:` blocks; sets tier `candidate`. | Each unit passes `scripts/promote.sh --dry-run` from this repository. |
| The promotion gate (`scripts/promote.sh`, this repository) | The only writer of `skills/S-communication-and-handoff/`. | Every promoted directory carries `PROMOTED.json`; a unit failing any check is refused with the reason on stderr. |
| The `sno` CLI installer (sibling PRD) | Reads each promoted unit's `PROMOTED.json` and `requires:` block; installs text into each harness's skill directory, places `public-bin` commands, wires hooks. | It can decide install / degrade / skip from the declaration alone, without reading the skill body. |
| An agent (Claude Code, Codex, Hermes, OpenClaw) reading the installed text | Follows `reach`, `handoff`, `heartbeat`, `subscription-quota-check`. | With only the text and the installed programs, it completes the journeys in section 8. |
| A reader of this repository | Opens `skills/S-communication-and-handoff/README.md`. | Sees the four units, one line each, and where their PRD is. |

Boundary: this PRD ends when the four units are promoted (tier `shipped` in the workshop
registry, `PROMOTED.json` here), the gate carries S3's two checks, the category README lists
them, and LICENSE/NOTICE exist. Installing them anywhere is the installer PRD.

## 5. The design
<!-- prd:design -->

### 5.1 Settled decisions

- `[DEC-1]` Category S ships exactly four units: `reach`, `handoff`, `heartbeat`, `subscription-quota-check` (S1).
- `[DEC-2]` `reach` is text only. It says when to use which verb and what the exit codes mean for the agent's next move; it does not restate syntax (F8) and it names no old vocabulary (ruling 3).
- `[DEC-3]` Every unit carries a `requires:` block in its `SKILL.md` frontmatter with this shape (REQ-2 enforces it): `programs:` a list of `{name, min_version}`; `harness:` a list of `{slot, need}` where `slot` is `<perspective-number>.<row-slug>` from the capability map (F4; e.g. `2.pre-turn-context-injection`, `2.type-into-a-tmux-pane`, `1.tui-in-tmux`) and `need` is one of `required` (absent → the installer skips the unit on that harness), `preferred` (absent → installs with a degraded note), `optional` (no effect). This block is the contract the installer consumes.
- `[DEC-4]` A text-only unit ships `scripts/selftest/run-selftest.sh` that checks the copy's own text: every `sno reach <verb>` it names is in the contract's verb list, every `--flag` it names is in the flag list, and none of the old names appear. It runs with or without the program installed.
- `[DEC-5]` The gate refuses a unit with zero self-tests and a unit whose `SKILL.md` lacks a parseable `requires:` block (S3). `scripts/promote.sh --selftest` gains one case for each.
- `[DEC-6]` `heartbeat`'s self-test runs the copy's own command from the copy's `public-bin/`, not the one on PATH; the "public-command" stages run every time.
- `[DEC-7]` `handoff` keeps the workshop text's meaning (transfer full ownership of a work item to a fresh seat with a verified context hand-over) rewritten over Reach verbs: the outgoing seat's item ends `cancelled` with a reason naming the new seat, the new seat's copy starts `accepted`, and the outgoing seat unregisters.
- `[DEC-8]` Root `LICENSE` (Apache-2.0) and `NOTICE` exist; `NOTICE` lists nothing third-party for category S (no vendored code ships here).
- `[DEC-9]` The category README lists the four units, one line each, and points to this PRD.

### 5.2 Requirements

- `[REQ-1]` The gate SHALL refuse a unit that ships no `scripts/selftest/run-selftest.sh`, with stderr naming the unit and the missing file, and `--selftest` SHALL prove it on a temp tree.
- `[REQ-2]` The gate SHALL refuse a unit whose `SKILL.md` frontmatter has no `requires:` block or one that does not match DEC-3, with stderr naming the first bad field, and `--selftest` SHALL prove it.
- `[REQ-3]` The gate SHALL refuse a category-S unit whose payload contains any of `mbox`, `mailbox.sh`, `mailbox-doorbell`, `--journey`, `--callsign`, `--message `, `X-Journey`, `X-Callsign`, `SNO_MBOX_`, `sno-agent-mbox`, printing file:line:match as it does for personal bindings.
- `[REQ-4]` The `reach` unit's text SHALL name each of the twenty-one verbs at least once with the situation that calls for it, SHALL state the `call` exit codes 0/3/4/5 and what the agent does on each, SHALL state that a work item is answered with `reply --state accepted` first and exactly one `reply --state completed|failed` last, and SHALL pass its DEC-4 self-test on the copy.
- `[REQ-5]` The `handoff` unit's text SHALL describe DEC-7 as an ordered list of `sno reach` commands the outgoing and incoming seats run, and SHALL pass its DEC-4 self-test on the copy.
- `[REQ-6]` `heartbeat` SHALL pass the gate with its public-command stages executed on the copy's own command (DEC-6), and `subscription-quota-check` SHALL pass the gate unchanged.
- `[REQ-7]` Each of the four promoted units SHALL carry `PROMOTED.json` with `selftests_run` ≥ 1, and the workshop registry SHALL show each at tier `shipped` after promotion.
- `[REQ-8]` Each of the four `SKILL.md` files SHALL carry a `requires:` block per DEC-3; `reach` and `handoff` SHALL declare `programs: [{name: reach, min_version: "2.0"}]` (the Reach PRD's first release version, its ruling 9); `heartbeat` and `subscription-quota-check` SHALL declare no program.
- `[REQ-9]` `skills/S-communication-and-handoff/README.md` SHALL list exactly the four units with one line each and a link to this PRD, and SHALL name no other unit.
- `[REQ-10]` Root `LICENSE` SHALL be the Apache-2.0 text and root `NOTICE` SHALL exist and name the project; the gate SHALL refuse any promotion while either is missing.

### 5.3 What changes for each unit (the per-unit short PRDs, S4)

| Unit | Workshop source | Changes for the public version |
|---|---|---|
| `reach` | new; absorbs the text of `agent-window-locate`, `agent-window-direct-message`, and the "when to use the mailbox" parts of the TPM skill | written fresh over the naming contract; text only; DEC-4 self-test; `requires: programs reach; harness 2.pre-turn-context-injection preferred` |
| `handoff` | `agent-cutover` | renamed; rewritten over Reach verbs (DEC-7); DEC-4 self-test added (today 0); `requires: programs reach` |
| `heartbeat` | `heartbeat` | self-test made portable (DEC-6); `requires: harness 1.tui-in-tmux optional`; de-personalised (gate list) |
| `subscription-quota-check` | same | `requires:` block added; otherwise unchanged (passes the gate today) |

### 5.4 Disposition table

| Input or failure class | Fate |
|---|---|
| Unit in registry, tier candidate/shipped, clean, self-tests green, `requires:` valid | Promoted; `PROMOTED.json` written last |
| Unit with zero self-tests | Refused (REQ-1) |
| Unit with no or malformed `requires:` | Refused naming the field (REQ-2) |
| Payload with a personal binding | Refused with file:line:match (F1) |
| Category-S payload with old vocabulary | Refused with file:line:match (REQ-3) |
| `heartbeat` on PATH differs from the copy | Irrelevant: the self-test runs the copy's command (DEC-6) |
| LICENSE or NOTICE missing | Every promotion refused (REQ-10) |
| Unit not in registry or tier internal | Refused (F1) |

exit=0
```

## Command

```sh
sed -n '38,60p' /home/lh/code/sno-station-skills/ai-doc/RESEARCH/harness-capability-matrix/harness-capability-matrix.md; sed -n '95,108p' /home/lh/code/sno-station-skills/ai-doc/RESEARCH/harness-capability-matrix/harness-capability-matrix.md; sed -n '140,157p' /home/lh/code/sno-station-skills/ai-doc/RESEARCH/harness-capability-matrix/harness-capability-matrix.md
```

```text

## 2 · Input — how do I push a task or a message in

| | Claude Code | Codex | OpenClaw | Hermes | dsh |
|---|---|---|---|---|---|
| **Prompt at launch** | argv or stdin | argv, stdin (`-`), `-i <image>` | `agent exec "<text>"`, `--message`/`--message-file` (4 MiB, `-` = stdin) | `-z`, `chat -q`, `--query-file <path>` (nothing shell-interpreted) | argv to `--profile headless` |
| **Type into a tmux pane** | yes, `tmux send-keys ... C-m` | yes; composer placeholder is `Ask Codex to do anything` | yes (TUI) | yes (TUI) | no TUI to type into |
| **Its own message channel** | `--input-format stream-json` control channel: `initialize`, `interrupt`, `set_model`, `set_permission_mode`, `can_use_tool`, `hook_callback`, `mcp_message`, `side_question`, `get_usage`, `stop_task`, `reload_skills`, `scheduled_task_fire` | **`codex queue --thread <id> --message "..."`**; app-server `turn/start`, **`turn/steer`** (mid-turn redirect), `turn/interrupt`, **`thread/inject_items`** (append raw items to model-visible history) | `openclaw message send --channel telegram\|discord\|slack\|signal\|matrix --target ...`; **`openclaw system event --text "..." --session-key <k> --mode now\|next-heartbeat`** is the external-nudge primitive | `hermes send --to telegram\|discord:#ops\|slack:C0123 "text"` — no LLM, no agent loop, no gateway needed; **documented exit codes 0/1/2**; cross-machine `hermes peer dm\|run` | ACP stdio, SDK stdio, web UI |
| **File inbox** | none native | none native | none native | `~/.hermes/pending_messages/` exists, **contract unknown** | none native |
| **HTTP ingress** | no | no | **OpenAI-compatible**: `GET /v1/models` lists each agent as a model id; `/health` unauthenticated 200 | `hermes webhook subscribe\|list\|remove\|test` | `dsh-webhook` + `dsh-webhook-github` bundled |
| **Scheduled** | `scheduled_task_fire` control request | no first-class cron | `openclaw cron add\|run\|runs\|list` | `hermes cron create\|edit\|run\|runs\|incidents\|notepad\|tick` | `dsh-schedule` bundled (cron / interval / at) |
| **Durable task board** | no | no | `openclaw tasks` (queued/running/succeeded/failed/timed_out/cancelled/lost/blocked) | **`hermes kanban create\|claim\|assign\|dispatch\|watch\|tail\|swarm`** — SQLite board, atomic claim, isolated workspace per worker. The intended orchestrator ingress. | `dsh-jobs-local` + `dsh-tool-jobs` |
| **Pre-turn context injection, pure config** | **yes.** `~/.claude/settings.json` `hooks:` → hook prints `{"hookSpecificOutput":{"additionalContext":"..."}}`, cap 8000 chars | **yes.** `~/.codex/hooks.json`, same wire shape, `additionalContextLimit` default 2500 tokens; trust hash gate | **partly.** Bundled `bootstrap-extra-files` internal hook injects files on `agent:bootstrap` — but **only these basenames**: `AGENTS.md`, `SOUL.md`, `IDENTITY.md`, `USER.md`, `BOOTSTRAP.md`, `MEMORY.md`. Real prompt rewriting needs a **TypeScript plugin** (`before_prompt_build`). | **yes, best of the five.** `hooks:` in `~/.hermes/config.yaml`, any language with a shebang. `pre_llm_call` returns `{"context":"..."}` → injected into the user message once per turn. Payload carries `user_message`, `conversation_history`, `is_first_turn`, `model`, `platform`, `sender_id`. | **yes, and it speaks both vendors' formats**: bundled `dsh-hooks-claude-code` (PreToolUse, PostToolUse, SessionStart, Stop) and `dsh-hooks-codex` (SessionStart, UserPromptSubmit) |
| **Hook event count** | 27 named events | 12 | internal 12 (observation only) + ~20 plugin hooks | ~30 in `VALID_HOOKS` | Claude + Codex compatibility sets |
| **MCP resource pull** | client with `resources/list\|read\|templates/list\|subscribe` | client + app-server `mcpServer/resource/read` | client (`openclaw mcp`); per-session MCP over ACP is **rejected** | client with OAuth (`hermes mcp login`) + curated catalog | `dsh-mcp-client`: stdio, http, sse, streamable |

**The big compatibility fact.** Codex's own embedded schema comment says it deliberately implements
Claude Code's hook wire format. Hermes accepts Claude's `{"decision":"block","reason":...}` shape and
Claude's exit-code-2-blocks convention. dsh ships compatibility layers for both. **One hook script can
target four of the five harnesses.** Only the config file location differs.

---

---

## 5 · Install surface — how do our things get in

| | Claude Code | Codex | OpenClaw | Hermes | dsh |
|---|---|---|---|---|---|
| **Skill directory** | `~/.claude/skills/<n>/SKILL.md`; repo `.claude/skills` | `~/.codex/skills/<n>/SKILL.md` (a **symlink to `~/.agents/skills`** on our machines); repo `.codex/skills`; scopes user/repo/system/admin | `<workspace>/skills`, `<workspace>/.agents/skills`, `~/.agents/skills`, `<stateDir>/skills`, bundled, `skills.load.extraDirs` | project `<repo>/.hermes/skills` + `<repo>/.agents/skills` (needs `hermes skills trust`), local `~/.hermes/skills/`, external `skills.external_dirs` | `<proj>/.dsh/skills`, `<proj>/.agents/skills`, custom dirs, `$DSH_HOME/skills`, **`~/.agents/skills`** |
| **Generic SKILL.md** | yes | yes | yes | yes, plus `version`, `platforms`, `metadata.hermes.*` | yes, directory-bundle or flat Markdown |
| **Reads `~/.agents/skills`** | no | **yes** (via the symlink) | **yes — but see the trap** | **yes**, via `skills.external_dirs` | **yes**, built in |
| **The trap** | — | — | **when `OPENCLAW_STATE_DIR` is not the default `~/.openclaw`, home-scoped roots such as `~/.agents/skills` are excluded from the session skill index.** On our `sno-e2e` profile that tier is dead. Use `skills.load.extraDirs` or `<stateDir>/skills`. | "External dirs are not a write-protection boundary" — the agent's own `skill_manage` can rewrite your source of truth in place. Use filesystem permissions. | — |
| **Import another vendor** | `claude import codex\|gemini` | app-server `externalAgentConfig/detect`+`import`: AGENTS_MD, CONFIG, SKILLS, PLUGINS, MCP_SERVER_CONFIG, SUBAGENTS, HOOKS, COMMANDS, MEMORY, SESSIONS | `openclaw migrate claude\|codex\|hermes` | `hermes import-agent claude-code\|codex`; `hermes claw migrate`; `hermes sessions import` | reads `AGENTS.md` **and** `CLAUDE.md`; ships Claude and Codex hook layers |
| **Hooks: config or plugin** | **pure config** | **pure config** (+ trust hash) | internal hooks are config + a JS/TS file but **cannot rewrite the prompt**; prompt injection needs a **TypeScript plugin** | **pure config**, any language with a shebang | **pure config**, in both vendors' formats |
| **MCP client** | yes | yes | yes | yes, with OAuth + curated catalog | yes |
| **MCP server** | `claude mcp serve` | `codex mcp-server` | `openclaw mcp serve` | `hermes mcp serve` | not observed |
| **Breaking-change signal** | none published — **unknown**. `--safe-mode` and `--bare` exist because customization surfaces break. | **visible**: `codex features list` shows 26 `removed` and 41 `under development`; app-server protocol versioned v1/v2 with a `deprecationNotice` notification | docs carry "Upcoming deprecations" + a plugin-API compat range; no BREAKING markers in the changelog | mechanical, not announced: `hermes migrate`, `config migrate`, `doctor --ack <ID>`, runtime deprecation warnings. **Expect config-shape churn on any bump.** | **all pre-release.** Breaking changes expected. |
| **Squad position** | boss | trusted worker | boss / orchestrator | worker that can also boss | **watch position** — everything is pre-release |
| **Platforms** | darwin arm64/x64, linux arm64/x64; Windows via hook templates | linux musl+gnu x86_64/aarch64, macOS Seatbelt, Windows (sandbox in flux) | Node ≥22.22.3; macOS/Linux/Windows | Python 3.11-3.13; Linux/macOS/Windows | Node; `os: "any"` |

---

## What this matrix says, in short

1. **One SKILL.md already serves four of the five.** Codex, OpenClaw, Hermes and dsh all read the
   vendor-neutral `~/.agents/skills`, which is where this repo already deploys. Claude Code needs its
   own copy, which `deploy.sh` already writes. One trap: OpenClaw drops that tier when
   `OPENCLAW_STATE_DIR` is not the default — our `sno-e2e` profile is exactly that case.

2. **One hook script serves four of the five.** Claude Code defined the wire format; Codex implements
   it deliberately; Hermes accepts its block shape and its exit-code-2 convention; dsh ships
   compatibility layers for both. Only OpenClaw needs a TypeScript plugin for real prompt injection.

3. **Rate-limit behaviour is the scheduling landmine.** Claude Code waits and resumes. Codex fails and

exit=0
```

## Command

```sh
rg -n 'PROMOTED|payload|rsync|cp |split' /home/lh/code/sno-station-skills/scripts/promote.sh
```

```text
4:# Copies one unit's deployable payload from the private workshop (sno-skills) into
8:#   2. the payload carries no personal binding (home paths, owner names, internal hosts);
9:#   3. every self-test the payload ships (scripts/selftest/run-selftest.sh) is green when
11:#   4. PROMOTED.json is written last, so a directory without it was never promoted.
24:# the unit; members the registry marks `split` to another category are recorded in
25:# PROMOTED.json and filed in that category's PRD directory, not moved on disk.
35:readonly STAMP=PROMOTED.json
86:# registry_row <unit> -> "cat tier split" (split may be empty). Reads the workshop registry,
87:# which keeps one unit per line in the shape  name: {cat: X, tier: Y[, split: {...}]}.
93:  local cat tier split
96:  split="$(sed -nE 's/.*split:[[:space:]]*\{([^}]*)\}.*/\1/p' <<<"$line" | tr -d ' ')"
98:  printf '%s %s %s\n' "$cat" "$tier" "$split"
101:# payload_dirs <unit-dir> -> lines of "<relative source>|<relative destination>"
105:payload_dirs() {
139:      cp -a -- "$u/skill" "$stage/$base"
145:        cp -a -- "$d" "$stage/${d##*/}"
180:  local cat tier split
181:  read -r cat tier split <<<"$(registry_row "$unit")"
190:  local pairs; pairs="$(payload_dirs "$src")" || die "workshop: '$unit' has no skill/SKILL.md, skills/*/SKILL.md or overlay/ — nothing to promote"
197:    cp -a -- "$src/$from" "$tmp/$to"
200:  # hashes, and PRD/report material that some family units keep inside their payload tree.
208:    die "gate: personal bindings in the payload of '$unit' (above, file:line:match) — remove them in the workshop, then promote again"
216:  local split_json='{}'
217:  if [[ -n "$split" ]]; then
218:    split_json="$(tr ',' '\n' <<<"$split" | jq -Rn '[inputs | select(length>0) | split(":") | {(.[0]): .[1]}] | add // {}')"
222:              --argjson clean "$clean" --argjson split "$split_json" --argjson tests "$ntests" \
224:              '{unit:$unit, category:$cat, tier:$tier, split:$split,
248:  cp -- "${BASH_SOURCE[0]}" "$ship/promote.sh"
255:  fam:         {cat: T, tier: candidate, split: {fam-b: R}}
303:  check "split recorded in stamp"           0 bash -c "jq -e '.split[\"fam-b\"]==\"R\"' '$shipdir/T-delegation-and-work-orders/fam/$STAMP'"
304:  check "prd/ inside payload is stripped"   1 test -e "$shipdir/T-delegation-and-work-orders/fam/skills/fam-a/prd"

exit=0
```
