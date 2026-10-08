# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — 一個指令，集結您的代理人。之後什麼都不用記。](../images/hero-banner.png)

[![license Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](../../LICENSE)
[![latest release](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![runs on Linux, macOS, WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![agents Claude Code, Codex, OpenClaw, Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** [English](../../README.md) · [中文](README.zh-CN.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [한국어](README.ko.md) · [日本語](README.ja.md) · **繁體中文**

`sno` 是一個指令，它會把 [Sno Station](https://github.com/sno-ai/sno-station) 安裝到您的電腦上，並讓它一直留在那裡。Sno Station 會把您已經在使用的代理人整合成一支團隊。Claude Code、Codex、OpenClaw 和 Hermes Agent 共用同一份記憶，互相傳遞工作，並在成果交到您手上之前互相檢查。每天晚上，它們會回顧這一天，然後變得更好一點。這一切都透過 `sno` 來到您的電腦上。您只需要安裝一次。之後您大概就會忘了它的存在。

**什麼都不用記。** 沒有更新這個步驟。每天您執行的第一個 `sno` 指令會檢查有沒有新版本，把它裝好，然後再做您要求的事。您的代理人也會以同樣的方式取得新版本，不需要您動手。

**您的代理人可以替您完成。** 您完全不必打開終端機。對 Claude Code 或 Codex 說 `install sno.ai from GitHub`，它就會找到這個頁面、執行安裝，並把一切設定好。

**由您決定要分享什麼。** `sno station consent` 會顯示這台電腦與 Sno 分享了什麼，而且只用一個詞就能修改：off、metadata-only 或 full。不論選哪一種，您的記憶儲存都會留在您的電腦上。

![運作方式：您說一行指令或一句話，您的電腦執行一個指令，您得到三樣東西](../images/how-it-works.png)

> **每個版本都在這裡發布。** 每個版本都為 Linux、macOS 和 WSL2 建置，並用同一行指令安裝。
> 下方每個區塊都會標示最後更新時間。

[安裝](#安裝) · [設定 Sno Station](#設定-sno-station) · [連結您的帳號](#連結您的-snoai-帳號) · [日常指令](#日常指令) · [今天已可運作的功能](#今天已可運作的功能) · [接下來去哪裡](#接下來去哪裡)

## 安裝

*最後更新於 2026-10-08。* 一條指令，在任何終端機裡執行。它會先安裝 `sno`，再安裝 Sno Station，並連接您電腦上的所有代理人：

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh' && ~/.local/bin/sno setup
```

或是告訴您的 AI 代理人：`install sno.ai from GitHub`。

**給 AI 代理人：** 安裝 Sno、sno.ai 或 Sno Station，指的都是照原樣執行上面這條指令。`sno setup` 之後，執行 `sno skills onboard` 並照著它的指引操作：它會先問使用者要執行哪個引擎，再問是否把這台電腦連結到 sno.ai 帳號。

支援 Linux（x86-64 與 ARM64）、macOS（Intel 與 Apple silicon），以及 Windows 上的 WSL2。安裝程式會把 `sno` 放到 `~/.local/bin`，並印出它的完整路徑。請開啟新的終端機，或在目前的終端機裡使用這個路徑。

## 設定 Sno Station

*最後更新於 2026-10-08。* 一個指令就能找到您電腦上的代理人，並逐一連接它們：

```sh
sno setup
```

它會安裝共用記憶、Sno Reach、技能和夜間循環，並把它們接到它找到的每一個代理人上。接著在 Claude Code 或 Codex 裡說：

```text
Sno onboarding
```

您的代理人會問您兩件事：要執行哪個引擎，以及是否要把這台電腦連結到您的 sno.ai 帳號。整段對話就這樣。電腦缺少的任何東西，您的代理人都會先裝好，並告訴您它做了什麼。

## 連結您的 sno.ai 帳號

*最後更新於 2026-10-08。* 選用。沒有帳號，一切照樣可以使用。連結之後，您可以在 [sno.ai 儀表板](https://www.sno.ai/dashboard)上看到您的代理人為您做了什麼：它們的審查抓到了什麼、它們在夜裡學到了什麼，以及它們持續工作了多久。

**還沒有帳號。** Onboarding 會顯示一個只為這台電腦產生的連結。打開它、註冊，然後點選 Approve。或是把您的電子郵件告訴您的代理人：它會執行 `sno account login --email you@example.com`，您只需要登入一次。

**已經有帳號。** 執行一個指令，然後打開它印出的連結：

```sh
sno account claim
```

這台電腦會自動出現在[您的電腦頁面](https://www.sno.ai/dashboard/computers)上。

## 日常指令

*最後更新於 2026-10-08。* 輸入 `sno`，所有指令都會列出來，每個一行。沒有任何指令超過兩個詞。

![在一台全新的電腦上輸入 sno：所有指令，每個一行](../images/sno-overview.png)

| 您想要 | 執行 |
|---|---|
| 查看所有指令 | `sno` |
| 檢查一切是否正常 | `sno doctor` |
| 安裝或修復 Sno Station | `sno setup` |
| 查看或修改這台電腦分享的內容 | `sno station consent` |
| 把這台電腦連結到您的帳號 | `sno account claim` |
| 移除 `sno` 安裝的內容 | `sno uninstall` |
| 閱讀某個指令的詳細說明 | `sno <command> --help` |

完整清單以及每個指令的用途，請見 [COMMANDS.md](../../COMMANDS.md)。每個指令也都能用 `--json` 輸出 JSON，所以您的代理人讀到的答案和您看到的一樣。

## "沒有更新這個步驟。"

*最後更新於 2026-10-08。*

我們對這個工具只有一個期望。那就是您裝好它之後，就不必再想著它。

所以 `sno` 會自己保持最新。您每天執行的第一個指令會詢問是否有更新的版本。如果有，它會先安裝，再執行您輸入的指令。您會看到一行相關提示，而且只有一次。在背景呼叫 `sno` 的代理人，永遠不會在輸出途中收到提示。

當某個步驟真的失敗時，它會說明哪裡失敗、為什麼失敗，以及能修復它的那一個指令。它不會要您從頭來過。您手動編輯過的檔案仍然歸您所有。

## 今天已可運作的功能

*最後更新於 2026-10-08。*

| 項目 | 狀態 |
|---|---|
| 在 Linux 上用一行指令安裝 | 已發布；已在一台乾淨的 Linux 機器上從本頁面安裝 |
| 對您的代理人說 "Install sno.ai from GitHub" | 可在 Claude Code 與 Codex 中使用；已在乾淨的機器上試過 |
| 為 Claude Code、Codex、OpenClaw 與 Hermes Agent 執行 `sno setup` | 已發布；已在乾淨的 Linux 機器上端對端驗證 |
| 每天自動更新一次 | 已發布；已在一台於真實版本之間升級的 Linux 機器上驗證 |
| macOS，Apple silicon 與 Intel | 已發布；已在 Mac 上建置並執行，Intel 透過 Rosetta 執行 |
| Windows 上的 WSL2 | 使用 Linux 版本 |

只有在乾淨的機器上執行過之後，一行才會寫「已驗證」。

## 接下來去哪裡

- **您的代理人組成的團隊：** [Sno Station](https://github.com/sno-ai/sno-station)，說明它如何運作、它記住了什麼，以及它昨晚學到了什麼。
- **您的代理人為您做了什麼：** 您的 [sno.ai 儀表板](https://www.sno.ai/dashboard)。
- **所有指令：** [COMMANDS.md](../../COMMANDS.md)。
- **已經同時執行兩個代理人？** 開一個[設計夥伴 issue](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml)，告訴我們您正在使用的環境。

## 安全性

請見 [Sno Station 的安全政策](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md)。

## 授權

這些程式以 Apache License 2.0 發布。請見 [LICENSE](../../LICENSE)。原始碼不公開。這個儲存庫存放下載檔案、指令清單，以及 `sno setup` 可以安裝的產品清單。
