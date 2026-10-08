# Sno CLI

`sno` is the one command for Sno. It sets up Sno Station on your computer, checks that it works, and keeps it up to date.

Sno Station gives Claude Code, Codex, OpenClaw and Hermes Agent a shared memory, a way to message each other, and a set of
skills. It runs on your own machine.

## Install

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
```

It works on Linux (x86-64 and ARM64), WSL2 and macOS (Intel and Apple silicon). Native Windows is not supported. Use WSL2.

The installer puts `sno` in `~/.local/bin` and prints its full path. Use that path in the terminal you are in, or open a new
one. Installing the CLI does not install Sno Station. That is the next step.

## First steps

```sh
sno setup     # install Sno Station
sno doctor    # check that everything works
sno update    # update the CLI and what you installed
```

Type `sno` on its own to see every command. Type `sno <command> --help` for one of them.

## Downloads and source

This repository holds the downloads and the list of products `sno setup` can install. Each release page carries the programs
for every supported system. The source code is not public. The programs are released under the Apache License 2.0.
