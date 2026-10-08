# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — 명령 하나로 여러분의 에이전트가 모입니다. 그 후에는 기억할 것이 없습니다.](../images/hero-banner.png)

[![라이선스 Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](../../LICENSE)
[![최신 릴리스](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![실행 환경: Linux, macOS, WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![지원 에이전트: Claude Code, Codex, OpenClaw, Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** [English](../../README.md) · [中文](README.zh-CN.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Русский](README.ru.md) · **한국어** · [日本語](README.ja.md) · [繁體中文](README.zh-TW.md)

`sno`는 [Sno Station](https://github.com/sno-ai/sno-station)을 여러분의 컴퓨터에 설치하고 계속 유지해 주는 하나의 명령입니다. Sno Station은 여러분이 이미 사용하는 에이전트들을 하나의 팀으로 만듭니다. Claude Code, Codex, OpenClaw, Hermes Agent가 하나의 메모리를 공유하고, 서로 작업을 넘겨주며, 결과가 여러분에게 도달하기 전에 서로의 작업을 확인합니다. 매일 밤 이들은 하루를 돌아보고 조금씩 나아집니다. 이 모든 것을 가져다주는 것이 `sno`입니다. 한 번만 설치하면 됩니다. 그 후에는 대부분 그 존재를 잊게 됩니다.

**기억할 것이 없습니다.** 업데이트 단계는 없습니다. 하루에 한 번, 여러분이 처음 실행하는 `sno` 명령이 새 릴리스를 확인하고 설치한 다음, 요청한 일을 수행합니다. 에이전트들도 같은 방식으로, 여러분 없이 새 버전을 받습니다.

**에이전트가 대신 해 줄 수 있습니다.** 터미널을 열 필요조차 없습니다. Claude Code나 Codex에게 `install sno.ai from GitHub`라고 말하면, 이 페이지를 찾아 설치를 실행하고 모든 것을 설정합니다.

**무엇을 공유할지는 여러분이 정합니다.** `sno station consent`는 이 컴퓨터가 Sno와 무엇을 공유하는지 보여 주고, off, metadata-only, full 중 한 단어로 바꿉니다. 어느 쪽이든 메모리 저장소는 여러분의 컴퓨터에 남습니다.

![작동 방식: 여러분의 한 줄 또는 한 문장, 여러분의 컴퓨터에서 실행되는 하나의 명령, 여러분이 얻는 세 가지](../images/how-it-works.png)

> **모든 릴리스는 여기에 올라옵니다.** 각 릴리스는 Linux, macOS, WSL2용으로 빌드되며, 같은 한 줄 명령으로 설치됩니다. 아래의 각 블록은 마지막으로 업데이트된 시점을 표시합니다.

[설치](#설치) · [Sno Station 설정](#sno-station-설정) · [계정 연결](#snoai-계정-연결) · [자주 쓰는 명령](#자주-쓰는-명령) · [오늘 작동하는 것](#오늘-작동하는-것) · [다음 단계](#다음-단계)

## 설치

*마지막 업데이트: 2026-10-08.* 어떤 터미널에서든 한 줄로 설치합니다:

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
```

또는 AI 에이전트에게 이렇게 말하세요: `install sno.ai from GitHub`.

Linux(x86-64와 ARM64), macOS(Intel과 Apple silicon), Windows의 WSL2에서 동작합니다. 설치 프로그램은 `sno`를 `~/.local/bin`에 넣고 전체 경로를 출력합니다. 새 터미널을 열거나, 지금 쓰고 있는 터미널에서는 그 경로를 사용하세요.

## Sno Station 설정

*마지막 업데이트: 2026-10-08.* 하나의 명령으로 컴퓨터에 있는 에이전트를 찾아 각각 연결합니다:

```sh
sno setup
```

공유 메모리, Sno Reach, 스킬, 야간 루프를 설치하고, 찾은 모든 에이전트에 연결합니다. 그런 다음 Claude Code나 Codex 안에서 이렇게 말하세요:

```text
Sno onboarding
```

에이전트는 두 가지를 묻습니다: 어떤 엔진을 실행할지, 그리고 이 컴퓨터를 여러분의 sno.ai 계정에 연결할지. 대화는 그게 전부입니다. 컴퓨터에 빠진 것이 있으면 에이전트가 먼저 설치하고 무엇을 했는지 알려 줍니다.

## sno.ai 계정 연결

*마지막 업데이트: 2026-10-08.* 선택 사항입니다. 계정이 없어도 모든 것이 작동합니다. 연결하면 [sno.ai 대시보드](https://www.sno.ai/dashboard)에서 에이전트가 여러분을 위해 한 일을 볼 수 있습니다: 리뷰에서 무엇을 잡아냈는지, 밤사이 무엇을 배웠는지, 얼마나 오래 작업을 이어 갔는지.

**아직 계정이 없다면.** 온보딩이 이 컴퓨터 전용으로 만든 링크를 보여 줍니다. 링크를 열고, 가입한 뒤 Approve를 클릭하세요. 또는 에이전트에게 이메일 주소를 알려 주세요: 에이전트가 `sno account login --email you@example.com`을 실행하고, 여러분은 한 번만 로그인하면 됩니다.

**이미 계정이 있다면.** 명령 하나를 실행하고 출력된 링크를 여세요:

```sh
sno account claim
```

컴퓨터는 [내 컴퓨터 페이지](https://www.sno.ai/dashboard/computers)에 저절로 나타납니다.

## 자주 쓰는 명령

*마지막 업데이트: 2026-10-08.* `sno`를 입력하면 모든 명령이 한 줄씩 나옵니다. 두 단어보다 깊은 명령은 없습니다.

![새 컴퓨터에서 sno를 입력한 모습: 모든 명령이 한 줄씩](../images/sno-overview.png)

| 하고 싶은 일 | 실행 |
|---|---|
| 모든 명령 보기 | `sno` |
| 모든 것이 작동하는지 확인 | `sno doctor` |
| Sno Station 설치 또는 복구 | `sno setup` |
| 이 컴퓨터가 공유하는 내용 보기 또는 변경 | `sno station consent` |
| 이 컴퓨터를 계정에 연결 | `sno account claim` |
| `sno`가 설치한 것 제거 | `sno uninstall` |
| 명령 하나의 세부 내용 읽기 | `sno <command> --help` |

각 명령이 하는 일을 담은 전체 목록은 [COMMANDS.md](../../COMMANDS.md)에 있습니다. 모든 명령은 `--json`으로 JSON도 출력하므로, 에이전트도 여러분과 같은 답을 읽습니다.

## "업데이트 단계는 없습니다."

*마지막 업데이트: 2026-10-08.*

우리가 이 도구에 바란 것은 하나였습니다. 설치하고 나면 더 이상 신경 쓰지 않아도 되는 것.

그래서 `sno`는 스스로 최신 상태를 유지합니다. 매일 처음 실행하는 명령이 새 릴리스가 있는지 확인합니다. 있으면 설치한 다음, 여러분이 입력한 명령을 실행합니다. 이에 대한 안내는 한 줄로, 한 번만 보입니다. 백그라운드에서 `sno`를 호출하는 에이전트는 출력 도중에 알림을 받지 않습니다.

어떤 단계가 실패하면, 무엇이 실패했는지, 왜 실패했는지, 그리고 그것을 고치는 명령 하나를 알려 줍니다. 처음부터 다시 시작하라고 하지 않습니다. 여러분이 직접 수정한 파일은 그대로 여러분의 것으로 남습니다.

## 오늘 작동하는 것

*마지막 업데이트: 2026-10-08.*

| 구성 요소 | 상태 |
|---|---|
| Linux에서 한 줄 설치 | 릴리스됨; 깨끗한 Linux 머신에서 이 페이지로 설치함 |
| 에이전트에게 "Install sno.ai from GitHub" 말하기 | Claude Code와 Codex에서 작동; 깨끗한 머신에서 시험함 |
| Claude Code, Codex, OpenClaw, Hermes Agent용 `sno setup` | 릴리스됨; 깨끗한 Linux 머신에서 처음부터 끝까지 검증됨 |
| 하루에 한 번 스스로 업데이트 | 릴리스됨; 실제 릴리스 사이를 오가는 Linux 머신에서 검증됨 |
| macOS, Apple silicon과 Intel | 릴리스됨; Mac에서 빌드하고 실행함, Intel은 Rosetta를 통해 |
| Windows의 WSL2 | Linux 빌드를 사용 |

행은 깨끗한 컴퓨터에서 실행된 경우에만 "검증됨"이라고 표시됩니다.

## 다음 단계

- **에이전트들이 이루는 팀:** [Sno Station](https://github.com/sno-ai/sno-station)에서 작동 방식, 무엇을 기억하는지, 지난밤 무엇을 배웠는지 볼 수 있습니다.
- **에이전트가 여러분을 위해 한 일:** 여러분의 [sno.ai 대시보드](https://www.sno.ai/dashboard).
- **모든 명령:** [COMMANDS.md](../../COMMANDS.md).
- **이미 두 에이전트를 나란히 실행하고 있나요?** [디자인 파트너 이슈](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml)를 열어 무엇을 실행하는지 알려 주세요.

## 보안

[Sno Station 보안 정책](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md)을 참고하세요.

## 라이선스

프로그램은 Apache License 2.0으로 배포됩니다. [LICENSE](../../LICENSE)를 참고하세요. 소스는 공개되어 있지 않습니다. 이 저장소에는 다운로드 파일, 명령 목록, 그리고 `sno setup`이 설치할 수 있는 제품 목록이 있습니다.
