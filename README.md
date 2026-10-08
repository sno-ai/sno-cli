# Sno CLI

One command that puts Sno Station on your computer and keeps it current.

Sno Station turns the agents you already use into a team. Claude Code, Codex, OpenClaw and Hermes Agent share one memory
and pass work to each other. Before anything reaches you, one of them has checked the other's work. Every night they look
back at the day and get a little better at it. All of it lives on your own machine.

`sno` is how you get there. You install it once, and after that you mostly forget it's there.

## Install

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
```

Or tell your AI agent: `install sno.ai from GitHub`.

Linux (x86-64 and ARM64), macOS (Intel and Apple silicon), and WSL2 on Windows.

The installer puts `sno` in `~/.local/bin` and prints its full path. Open a new terminal, or use that path in the one you
are in.

## Set up Sno Station

```sh
sno setup
```

It finds the agents on your computer and connects each of them. Then say "Sno onboarding" to Claude Code or Codex. Your agent
asks you two things: which engine to run, and whether to link this computer to your sno.ai account. That is the whole
conversation.

Linking is optional. If you do it, https://www.sno.ai/dashboard shows what your agents did for you, from what their
reviews caught to what they learned overnight.

## After that

```sh
sno doctor    # what is installed, and whether it works
sno           # every command, one line each
```

The full list, with what each command does, is in [COMMANDS.md](COMMANDS.md).

There is no update command to remember. Once a day, the first `sno` command you run checks for a new release, installs it,
and then does what you asked.

## License

The programs are released under the Apache License 2.0. The source is not public. This repository holds the downloads and
the list of products `sno setup` can install.
