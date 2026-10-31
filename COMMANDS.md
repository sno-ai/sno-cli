# Sno CLI commands

Every command `sno` has. Run `sno` on its own for the short version, or `sno <command> --help` for one command
with examples.

Every command accepts `--json` (exactly one JSON value on stdout) and `--help`. A command with a missing or wrong
argument prints guidance and exits 0 without running; a real failure exits non-zero. A unique start of a command
name works as the whole name: `sno stati cons` is `sno station consent`.

## Commands

| Command | What it does | Arguments |
|---|---|---|
| `sno setup` | Install Sno Station, or another Sno product by name | `[PRODUCT]` `--reach-version` `--skills-version` `--memory-mode` `--harness` |
| `sno init` | Set up the sno command and teach your agents to use it |  |
| `sno onboarding` | Choose Station settings and check that they work |  |
| `sno onboarding status` | Show installed agents, memory settings and saved choices without changing them |  |
| `sno onboarding apply` | Save your Station choices; anything you leave out keeps its current or default value | `--engine` `--writer` `--reviewer` `--daily-retrospective` |
| `sno onboarding verify` | Check that memory and agent-to-agent mail work, using a test memory and a test message |  |
| `sno skills` | List agent instructions, or read one by name | `[NAME]` `--full` `--all` |
| `sno products` | List optional product offers, or record a reply to one | `[PRODUCT]` `[ANSWER]` |
| `sno update` | Update sno and the products you installed | `--reach-version` `--skills-version` `--memory-mode` `--harness` `--auto` `--quiet` |
| `sno doctor` | Check CLI updates, installed products, skills and Station state |  |
| `sno usage` | Show the remaining weekly allowance and your balance |  |
| `sno stats` | Show allowance, review catches, Self Learning lessons and stored memories on one page |  |
| `sno memory` | Use memory installed for Claude, Codex or Cursor | `<ARGUMENTS>` |
| `sno observe` | Record agent activity (used by agent hooks) | `<ARGUMENTS>` |
| `sno project` | Show memories and Self Learning lessons for each project |  |
| `sno project status` | Show memories and Self Learning lessons for this folder |  |
| `sno project list` | List every project Sno has memories or lessons for |  |
| `sno uninstall` | Uninstall one installed product, or all of them | `[PRODUCT]` `--yes` `--purge-state` |
| `sno account` | Link this computer to your sno.ai account |  |
| `sno account login` | Sign up or sign in, and link this computer to your sno.ai account | `--email` |
| `sno account claim` | Print the link that approves linking this computer to your sno.ai account |  |
| `sno account register` | Register this computer with Sno, without an account |  |
| `sno rem` | Send Self Learning results and your answers on lessons to Sno |  |
| `sno rem judge` | Send a finished Self Learning as JSON on stdin |  |
| `sno rem recall` | Pick the lessons that fit a session's first message |  |
| `sno rem verdict` | Record your accept, reject or tbd answer on a lesson | `<JUDGMENT_ID>` `<VERDICT>` `--all-projects` |
| `sno station` | Upload settings, checks and records for Station on this computer |  |
| `sno station rem-start` | Start a memory maintenance job in the background | `--type` `--scope` |
| `sno station rem-status` | Show or wait for a background memory maintenance job | `[JOB_ID]` `--wait` `--timeout` |
| `sno station consent` | Show what this computer uploads to Sno, or set it to off, metadata-only or full | `[VALUE]` |
| `sno station pause` | Pause uploads to Sno until you resume them |  |
| `sno station resume` | Resume uploads at the setting you had before pausing |  |
| `sno station export` | Save a copy of the events this computer recorded | `[PATH]` `--out` `--format` |
| `sno station audit` | Check that an event Sno received matches this computer's record | `[EVENT_ID]` |
| `sno station doctor` | Check this computer's Sno ID, upload setting and event log |  |

## Actions handled by the installed memory and observability packages

`sno memory` and `sno observe` pass these to the packages `sno setup` installs.

| Command | What it does |
|---|---|
| `sno memory remember --harness claude|codex|cursor TEXT` | Store a project memory and print its id |
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
| `sno rem-reflect` | The Self Learning program and its lesson commands |
| `sno rotate-agent-resume` | Move a job to the other agent and verify it resumed |

## Not Sno commands

`claude`, `codex`, `hermes`, `openclaw`, `npm`, `git`, `systemctl` and `launchctl` belong to other products and keep their own names.
