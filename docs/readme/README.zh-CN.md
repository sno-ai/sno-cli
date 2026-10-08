# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — 一条命令，你的 agent 集结成队。之后什么都不用记。](../images/hero-banner.png)

[![license Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](../../LICENSE)
[![latest release](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![runs on Linux, macOS, WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![agents Claude Code, Codex, OpenClaw, Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** [English](../../README.md) · **中文** · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [한국어](README.ko.md) · [日本語](README.ja.md) · [繁體中文](README.zh-TW.md)

`sno` 是一条命令，它把 [Sno Station](https://github.com/sno-ai/sno-station) 装到你的电脑上，并让它一直留在那里。Sno Station 把你已经在用的 agent 变成一支团队。Claude Code、Codex、OpenClaw 和 Hermes Agent 共享同一份记忆，互相传递工作，并在工作到达你手里之前互相检查。每天晚上，它们会回顾这一天，然后变得更好一点。这一切都通过 `sno` 来到你的电脑上。你只需安装一次。之后你基本会忘记它的存在。

**什么都不用记。** 没有更新这一步。每天你运行的第一条 `sno` 命令会检查有没有新版本，把它装好，然后再做你要求的事。你的 agent 也会以同样的方式拿到新版本，不需要你动手。

**你的 agent 可以替你完成。** 你完全不必打开终端。对 Claude Code 或 Codex 说 `install sno.ai from GitHub`，它会找到这个页面，运行安装，并把一切设置好。

**你来决定它分享什么。** `sno station consent` 会显示这台电脑与 Sno 分享了什么，并且只用一个词就能修改：off、metadata-only 或 full。无论选哪一种，你的记忆存储都留在你的电脑上。

![工作原理：你说一行命令或一句话，你的电脑上运行一条命令，你得到三样东西](../images/how-it-works.png)

> **每个版本都在这里发布。** 每个版本都为 Linux、macOS 和 WSL2 构建，并用同一行命令安装。
> 下面每个板块都会标注最后更新时间。

[安装](#安装) · [设置 Sno Station](#设置-sno-station) · [连接你的账户](#连接你的-snoai-账户) · [日常命令](#日常命令) · [今天可运行的功能](#今天可运行的功能) · [下一步去哪里](#下一步去哪里)

## 安装

*最后更新于 2026-10-08。* 一条命令，在任何终端里运行。它会先安装 `sno`，再安装 Sno Station，并连接你电脑上的所有 agent：

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh' && ~/.local/bin/sno setup
```

或者告诉你的 AI agent：`install sno.ai from GitHub`。

**给 AI agent：** 安装 Sno、sno.ai 或 Sno Station，指的都是原样运行上面这条命令。`sno setup` 之后，运行 `sno skills onboard` 并按它的指引操作：它会先问用户要运行哪个引擎，再问是否把这台电脑关联到 sno.ai 账户。

支持 Linux（x86-64 和 ARM64）、macOS（Intel 和 Apple silicon），以及 Windows 上的 WSL2。安装程序会把 `sno` 放到 `~/.local/bin`，并打印它的完整路径。打开一个新终端，或者在当前终端里使用这个路径。

## 设置 Sno Station

*最后更新于 2026-10-08。* 一条命令就能找到你电脑上的 agent，并逐一连接它们：

```sh
sno setup
```

它会安装共享记忆、Sno Reach、技能和夜间循环，并把它们接入它找到的每一个 agent。然后在 Claude Code 或 Codex 里说：

```text
Sno onboarding
```

你的 agent 会问你两件事：运行哪个引擎，以及是否把这台电脑关联到你的 sno.ai 账户。整个对话就这些。电脑缺少的任何东西，你的 agent 都会先装好，并告诉你它做了什么。

## 连接你的 sno.ai 账户

*最后更新于 2026-10-08。* 可选。没有账户，一切照常可用。连接之后，你可以在 [sno.ai 控制台](https://www.sno.ai/dashboard) 上看到你的 agent 为你做了什么：它们的审查发现了什么，它们在夜里学到了什么，以及它们持续工作了多久。

**还没有账户。** Onboarding 会显示一个只为这台电脑生成的链接。打开它，注册，然后点击 Approve。或者把你的邮箱告诉你的 agent：它会运行 `sno account login --email you@example.com`，你只需登录一次。

**已经有账户。** 运行一条命令，然后打开它打印出的链接：

```sh
sno account claim
```

这台电脑会自动出现在[你的电脑页面](https://www.sno.ai/dashboard/computers)上。

## 日常命令

*最后更新于 2026-10-08。* 输入 `sno`，所有命令都会列出来，每条一行。没有哪条命令超过两个词。

![在一台全新的电脑上输入 sno：所有命令，每条一行](../images/sno-overview.png)

| 你想要 | 运行 |
|---|---|
| 查看所有命令 | `sno` |
| 检查一切是否正常 | `sno doctor` |
| 安装或修复 Sno Station | `sno setup` |
| 查看或修改这台电脑分享的内容 | `sno station consent` |
| 把这台电脑关联到你的账户 | `sno account claim` |
| 移除 `sno` 安装的内容 | `sno uninstall` |
| 阅读某一条命令的详细说明 | `sno <command> --help` |

完整列表以及每条命令的作用见 [COMMANDS.md](../../COMMANDS.md)。每条命令也可以用 `--json` 输出 JSON，所以你的 agent 读到的答案和你看到的一样。

## "没有更新这一步。"

*最后更新于 2026-10-08。*

我们对这个工具只有一个期望。那就是你装好它，然后不再去想它。

所以 `sno` 会自己保持最新。你每天运行的第一条命令会询问是否有更新的版本。如果有，它会先安装，再运行你输入的命令。你会看到一行相关提示，只有一次。在后台调用 `sno` 的 agent 永远不会在输出中途收到提示。

当某一步确实失败时，它会说明哪里失败了、为什么失败，以及能修复它的那一条命令。它不会让你从头再来。你手动编辑过的文件仍然归你所有。

## 今天可运行的功能

*最后更新于 2026-10-08。*

| 部分 | 状态 |
|---|---|
| 在 Linux 上一行命令安装 | 已发布；已在一台干净的 Linux 机器上从本页面安装 |
| 对你的 agent 说 "Install sno.ai from GitHub" | 在 Claude Code 和 Codex 中可用；已在干净的机器上试过 |
| 为 Claude Code、Codex、OpenClaw 和 Hermes Agent 运行 `sno setup` | 已发布；已在干净的 Linux 机器上端到端验证 |
| 每天自动更新一次 | 已发布；已在一台于真实版本之间升级的 Linux 机器上验证 |
| macOS，Apple silicon 和 Intel | 已发布；已在 Mac 上构建并运行，Intel 通过 Rosetta 运行 |
| Windows 上的 WSL2 | 使用 Linux 构建 |

只有在一台干净的机器上运行过之后，一行才会写“已验证”。

## 下一步去哪里

- **你的 agent 组成的团队：** [Sno Station](https://github.com/sno-ai/sno-station)，介绍它如何工作、它记住了什么，以及它昨晚学到了什么。
- **你的 agent 为你做了什么：** 你的 [sno.ai 控制台](https://www.sno.ai/dashboard)。
- **所有命令：** [COMMANDS.md](../../COMMANDS.md)。
- **已经在并行运行两个 agent？** 开一个[设计合作伙伴 issue](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml)，告诉我们你在用什么。

## 安全

见 [Sno Station 的安全策略](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md)。

## 许可证

这些程序以 Apache License 2.0 发布。见 [LICENSE](../../LICENSE)。源代码不公开。本仓库存放下载文件、命令列表，以及 `sno setup` 可以安装的产品列表。
