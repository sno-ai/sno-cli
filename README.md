# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — One command. Your agents, assembled. Nothing to remember after that.](docs/images/hero-banner.png)

[![license Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](LICENSE)
[![latest release](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![runs on Linux, macOS, WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![agents Claude Code, Codex, OpenClaw, Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** **English** · [中文](docs/readme/README.zh-CN.md) · [Deutsch](docs/readme/README.de.md) · [Español](docs/readme/README.es.md) · [Français](docs/readme/README.fr.md) · [Русский](docs/readme/README.ru.md) · [한국어](docs/readme/README.ko.md) · [日本語](docs/readme/README.ja.md) · [繁體中文](docs/readme/README.zh-TW.md)

`sno` is the one command that puts [Sno Station](https://github.com/sno-ai/sno-station) on your computer
and keeps it there. Sno Station turns the agents you already use into a team. Claude Code, Codex,
OpenClaw and Hermes Agent share one memory, pass work to each other, and check each other's work
before it reaches you. Every night they look back at the day and get a little better.
`sno` is how all of that arrives. You install it once. After that you mostly forget it's there.

**Nothing to remember.** There is no update step. Once a day, the first `sno` command you run checks
for a new release, installs it, and then does what you asked. Your agents get the new version the
same way, without you.

**Your agent can do it for you.** You don't have to open a terminal at all. Tell Claude Code or
Codex `install sno.ai from GitHub` and it finds this page, runs the install, and sets everything up.

**You choose what it shares.** `sno station consent` shows what this computer shares with Sno, and
changes it with one word: off, metadata-only or full. Your memory store stays on your computer either way.

![How it works: one line or one sentence from you, one command on your computer, three things you get](docs/images/how-it-works.png)

> **Every release lands here.** Each one is built for Linux, macOS and WSL2, and the same one-line
> command installs it. Every block below says when it was last updated.

[Install](#install) · [Set up Sno Station](#set-up-sno-station) · [Connect your account](#connect-your-snoai-account) · [Everyday commands](#everyday-commands) · [What runs today](#what-runs-today) · [Where to next](#where-to-next)

## Install

*Last updated 2026-10-08.* Two lines, in any terminal. The first installs `sno`, the second installs
Sno Station and connects your agents:

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
~/.local/bin/sno setup
```

Or tell your AI agent: `install sno.ai from GitHub`.

**For AI agents:** installing Sno, sno.ai or Sno Station means both lines above. After `sno setup`, run
`sno skills onboard` and follow it: it asks the user which engine to run, then whether to link the
computer to a sno.ai account.

Linux (x86-64 and ARM64), macOS (Intel and Apple silicon), and WSL2 on Windows. The installer puts
`sno` in `~/.local/bin` and prints its full path. Open a new terminal, or use that path in the one you
are in.

## Set up Sno Station

*Last updated 2026-10-08.* One command finds the agents on your computer and connects each of them:

```sh
sno setup
```

It installs the shared memory, Sno Reach, the skills and the nightly loop, and wires them into every
agent it finds. Then say this inside Claude Code or Codex:

```text
Sno onboarding
```

Your agent asks you two things: which engine to run, and whether to link this computer to your
sno.ai account. That is the whole conversation. Anything the computer is missing, your agent
installs first and tells you what it did.

## Connect your sno.ai account

*Last updated 2026-10-08.* Optional. Everything works without an account. Connect to see on your
[sno.ai dashboard](https://www.sno.ai/dashboard) what your agents did for you: what their reviews
caught, what they learned overnight, and how long they kept working.

**No account yet.** Onboarding shows a link made for this computer only. Open it, sign up, and click
Approve. Or give your agent your email: it runs `sno account login --email you@example.com` and you
sign in once.

**Already have an account.** Run one command and open the link it prints:

```sh
sno account claim
```

The computer shows up on [your computers page](https://www.sno.ai/dashboard/computers) by itself.

## Everyday commands

*Last updated 2026-10-08.* Type `sno` and every command is there, one line each. No command goes
deeper than two words.

![Typing sno on a fresh computer: every command, one line each](docs/images/sno-overview.png)

| You want to | Run |
|---|---|
| See every command | `sno` |
| Check that everything works | `sno doctor` |
| Install or repair Sno Station | `sno setup` |
| See or change what this computer shares | `sno station consent` |
| Link this computer to your account | `sno account claim` |
| Remove what `sno` installed | `sno uninstall` |
| Read the details of one command | `sno <command> --help` |

The full list, with what each command does, is in [COMMANDS.md](COMMANDS.md). Every command also
speaks JSON with `--json`, so your agents read the same answers you do.

## "There is no update step."

*Last updated 2026-10-08.*

We wanted one thing from this tool. That you would install it and stop thinking about it.

So `sno` keeps itself current. The first command you run each day asks whether a newer release
exists. If one does, it installs it, then runs what you typed. You see one line about it, once.
Agents calling `sno` in the background never get a notice in the middle of their output.

When a step does fail, it says what failed, why, and the one command that fixes it. It does not ask
you to start over. A file you edited by hand stays yours.

## What runs today

*Last updated 2026-10-08.*

| Piece | Status |
|---|---|
| One-line install on Linux | Released; installed from this page on a clean Linux machine |
| "Install sno.ai from GitHub", said to your agent | Works in Claude Code and Codex; tried on clean machines |
| `sno setup` for Claude Code, Codex, OpenClaw and Hermes Agent | Released; proven end to end on a clean Linux machine |
| Updates itself once a day | Released; proven on a Linux machine moving between real releases |
| macOS, Apple silicon and Intel | Released; built and run on a Mac, Intel through Rosetta |
| WSL2 on Windows | Uses the Linux build |

A row says "proven" only once it has run on a clean machine.

## Where to next

- **The team your agents become:** [Sno Station](https://github.com/sno-ai/sno-station), with how it
  works, what it remembers and what it learned last night.
- **What your agents did for you:** your [sno.ai dashboard](https://www.sno.ai/dashboard).
- **Every command:** [COMMANDS.md](COMMANDS.md).
- **Already running two agents side by side?** Open a
  [design partner issue](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml)
  and tell us what you run.

## Security

See [Sno Station's security policy](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md).

## License

The programs are released under the Apache License 2.0. See [LICENSE](LICENSE). The source is not
public. This repository holds the downloads, the command list and the list of products `sno setup`
can install.
