# Sno CLI

The command line for [Sno Station](https://github.com/sno-ai/sno-station), from [sno.ai](https://www.sno.ai).

Sno Station turns the agents you already use into a team. Claude Code, Codex, OpenClaw and Hermes Agent share
one memory, pass work to each other, and check each other's work before it reaches you. `sno` is the one
command that puts it on your computer and keeps it current.

## Install

```sh
curl -fsSL https://sno.ai/install/crate | sh
```

That installs the latest official `sno` in `~/.local/bin`. Or tell your AI agent: `install sno.ai from GitHub`.

## About `cargo install sno`

```sh
cargo install sno
```

This crate is a small launcher, not the whole program. The first time you run `sno`, it downloads the latest
official `sno` with the installer above, then hands over to it. After that it just starts it. The official
`sno` keeps itself current once a day, so you never need to reinstall this crate to get a new version.

## Then

```sh
sno setup     # find your agents and connect each of them
sno           # every command, one line each
```

Linux (x86-64 and ARM64), macOS (Intel and Apple silicon), and WSL2 on Windows. Everything else is on
[GitHub](https://github.com/sno-ai/sno-cli).

## License

Apache License 2.0.
