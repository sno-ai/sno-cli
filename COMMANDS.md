# Sno CLI commands

Every command `sno` has. Run `sno` on its own for the short version, or `sno <command> --help` for one command
with examples.

Every command accepts `--json` (exactly one JSON value on stdout) and `--help`. A command with a missing or wrong
argument prints guidance and exits 0 without running; a real failure exits non-zero. A unique start of a command
name works as the whole name: `sno stat cons` is `sno station consent`.

## Commands

| Command | What it does | Arguments |
|---|---|---|
| `sno setup` | Install the default or selected catalog product | `[PRODUCT]` `--reach-version` `--skills-version` `--memory-mode` `--harness` |
| `sno init` | Initialize independent CLI state and entry skills |  |
| `sno onboarding` | Configure and prove the selected Station installation |  |
| `sno onboarding status` | Show installed agents, memory settings and saved choices without changing them |  |
| `sno onboarding apply` | Configure Station using supplied choices and defaults for omitted choices | `--engine` `--writer` `--reviewer` `--daily-retrospective` |
| `sno onboarding verify` | Write and read back a test memory and deliver a test Reach message |  |
| `sno skills` | List agent instructions, or read one by name | `[NAME]` `--full` `--all` |
| `sno products` | List optional product offers, or record a reply to one | `[PRODUCT]` `[ANSWER]` |
| `sno update` | Update CLI and actually installed products | `--reach-version` `--skills-version` `--memory-mode` `--harness` `--auto` `--quiet` |
| `sno doctor` | Check CLI updates, installed products, skills and Station state |  |
| `sno usage` | Show remaining model allowance and purchased balance |  |
| `sno stats` | Show allowance, review catches, nightly lessons and stored memories on one page |  |
| `sno memory` | Use memory installed for Claude or Codex | `<ARGUMENTS>` |
| `sno observe` | Send observability events | `<ARGUMENTS>` |
| `sno project` | Report local project memory and recursive self-improvement records |  |
| `sno project status` | Show recorded current contexts, memory availability and improvement outcomes |  |
| `sno project list` | List known source project identities and shared general records |  |
| `sno uninstall` | Uninstall one installed product, or all of them | `[PRODUCT]` `--yes` `--purge-state` |
| `sno account` | Manage account and machine identity |  |
| `sno account login` | Sign up or sign in and attach this computer to a SNO account | `--email` |
| `sno account claim` | Attach this computer to a SNO account and print the link to approve it |  |
| `sno account register` | Register this machine anonymously |  |
| `sno rem` | Send REM runs and human verdicts to SNO |  |
| `sno rem judge` | Send a complete REM run as JSON on stdin |  |
| `sno rem recall` | Rank accepted lessons for a first owner message |  |
| `sno rem verdict` | Send a linked human verdict | `<JUDGMENT_ID>` `<VERDICT>` `--all-projects` |
| `sno station` | Manage this SNO Station |  |
| `sno station rem-start` | Start an asynchronous local REM job | `--type` `--scope` |
| `sno station rem-status` | Read or wait for a local REM job | `[JOB_ID]` `--wait` `--timeout` |
| `sno station consent` | Show what this computer shares, or set it to off, metadata-only or full | `[VALUE]` |
| `sno station pause` | Pause cloud telemetry |  |
| `sno station resume` | Resume cloud telemetry |  |
| `sno station export` | Export local audit events | `[PATH]` `--out` `--format` |
| `sno station audit` | Verify a server-stored event | `[EVENT_ID]` |
| `sno station doctor` | Check local Station identity, telemetry, buffer and configuration |  |

## Actions handled by the installed memory and observability packages

`sno memory` and `sno observe` pass these to the packages `sno setup` installs.

| Command | What it does |
|---|---|
| `sno memory remember --harness claude|codex TEXT` | Store a project memory and print its id |
| `sno memory recall --harness H QUERY` | Search project and global memory |
| `sno memory get --harness H ID` | Print one full memory |
| `sno memory correct --harness H ID TEXT` | Store a corrected memory and retire the old one |
| `sno memory import --harness H (--repo ROOT | --user)` | Queue existing agent notes for import |
| `sno memory doctor --harness H` | Print memory service health, hook state and permission rule state |
| `sno memory dump --db PATH` | Read the memory store and print rows |
| `sno memory hook EVENT --harness H` | Entry point for the agent's hooks; never blocks the agent |
| `sno observe append EVENT_TYPE --agent=HARNESS --FIELD=VALUE` | Record one observability event |

## Station commands

`sno setup` installs these. They run as `sno <name> [arguments]`; nothing else is put on PATH.
`sno <name> --help` prints each one's own help, and `sno --help` ends with the list installed on this machine.

| Command | What it does |
|---|---|
| `sno reach` | Contact another agent, exchange work cards, read the inbox, wait for a reply, manage seats |
| `sno heartbeat` | Report every so often, or wait until a file appears, for long jobs |
| `sno report-time` | Print a clock time for the user in their own time zone |
| `sno subscription-quota-check` | Read Codex and Claude Code subscription quota without spending any |
| `sno away-brief` | One page of what is done, stuck, needs the owner, and spend |
| `sno catch-report` | Count what mutual review and self-correction caught |
| `sno deliver-proof` | Prove a work-order step by running a command and recording the result |
| `sno handoff-checkpoint` | Write and verify the progress record that lets another agent take over |
| `sno medic` | Check the agent team on this machine; repairs nothing |
| `sno rem-reflect` | The nightly self-reflection program and its lesson commands |
| `sno rotate-agent-resume` | Move a job to the other agent and verify it resumed |

## Not Sno commands

`claude`, `codex`, `hermes`, `openclaw`, `npm`, `git`, `systemctl` and `launchctl` belong to other products and keep their own names.
