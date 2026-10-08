# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — ひとつのコマンドで、あなたのエージェントがそろう。そのあとは何も覚えておく必要がありません。](../images/hero-banner.png)

[![ライセンス Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](../../LICENSE)
[![最新リリース](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![動作環境：Linux、macOS、WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![対応エージェント：Claude Code、Codex、OpenClaw、Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** [English](../../README.md) · [中文](README.zh-CN.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [한국어](README.ko.md) · **日本語** · [繁體中文](README.zh-TW.md)

`sno` は、[Sno Station](https://github.com/sno-ai/sno-station) をあなたのコンピューターにインストールし、そのあとも維持し続けるひとつのコマンドです。Sno Station は、あなたがすでに使っているエージェントたちをひとつのチームに変えます。Claude Code、Codex、OpenClaw、Hermes Agent がひとつのメモリを共有し、作業を互いに引き渡し、あなたに届く前に互いの作業をチェックします。毎晩、彼らはその日を振り返り、少しずつ良くなります。そのすべてを届けるのが `sno` です。インストールは一度だけです。そのあとは、ほとんどその存在を忘れていられます。

**覚えておくことはありません。** 更新の手順はありません。1日に一度、最初に実行した `sno` コマンドが新しいリリースを確認してインストールし、それから頼まれたことを実行します。エージェントたちも同じように、あなたの手を借りずに新しいバージョンを受け取ります。

**エージェントに任せることもできます。** ターミナルを開く必要はまったくありません。Claude Code か Codex に `install sno.ai from GitHub` と伝えれば、このページを見つけ、インストールを実行し、すべてをセットアップします。

**何を共有するかはあなたが決めます。** `sno station consent` は、このコンピューターが Sno と何を共有しているかを表示し、off、metadata-only、full のひとことで変更します。どの場合でも、メモリストアはあなたのコンピューターに残ります。

![仕組み：あなたからの1行または1文、あなたのコンピューター上のひとつのコマンド、あなたが得る3つのもの](../images/how-it-works.png)

> **すべてのリリースはここに届きます。** どのリリースも Linux、macOS、WSL2 向けにビルドされ、同じ1行のコマンドでインストールできます。以下の各ブロックには最終更新日が記されています。

[インストール](#インストール) · [Sno Station のセットアップ](#sno-station-のセットアップ) · [アカウントを接続する](#snoai-アカウントを接続する) · [日常のコマンド](#日常のコマンド) · [現在動いているもの](#現在動いているもの) · [次のステップ](#次のステップ)

## インストール

*最終更新 2026-10-08。* どのターミナルでも、2行で実行できます。1行目で `sno` をインストールし、2行目で Sno Station をインストールしてエージェントを接続します：

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
~/.local/bin/sno setup
```

または、AI エージェントにこう伝えます：`install sno.ai from GitHub`。

**AI エージェントへ：** Sno、sno.ai、Sno Station のインストールとは、上の2行の両方を実行することです。`sno setup` のあと、`sno skills onboard` を実行してその案内に従ってください。まずどのエンジンを動かすかをユーザーに尋ね、次にこのコンピューターを sno.ai アカウントにリンクするかどうかを尋ねます。

Linux（x86-64 と ARM64）、macOS（Intel と Apple silicon）、Windows 上の WSL2 に対応しています。インストーラーは `sno` を `~/.local/bin` に置き、そのフルパスを表示します。新しいターミナルを開くか、今のターミナルではそのパスを使ってください。

## Sno Station のセットアップ

*最終更新 2026-10-08。* ひとつのコマンドで、コンピューター上のエージェントを見つけ、それぞれを接続します：

```sh
sno setup
```

共有メモリ、Sno Reach、スキル、夜間ループをインストールし、見つかったすべてのエージェントにつなぎ込みます。そのあと、Claude Code か Codex の中でこう話しかけます：

```text
Sno onboarding
```

エージェントはあなたに2つのことを尋ねます。どのエンジンを動かすか、そしてこのコンピューターをあなたの sno.ai アカウントにリンクするかどうかです。会話はそれだけです。コンピューターに足りないものがあれば、エージェントが先にインストールし、何をしたかを伝えます。

## sno.ai アカウントを接続する

*最終更新 2026-10-08。* 任意です。アカウントがなくても、すべて動きます。接続すると、[sno.ai ダッシュボード](https://www.sno.ai/dashboard)で、エージェントがあなたのために何をしたかを確認できます。レビューで何を見つけたか、夜のあいだに何を学んだか、どれだけ長く作業を続けたかです。

**まだアカウントがない場合。** オンボーディングで、このコンピューター専用のリンクが表示されます。リンクを開いて登録し、Approve をクリックします。または、エージェントにメールアドレスを伝えます。エージェントが `sno account login --email you@example.com` を実行し、あなたは一度サインインするだけです。

**すでにアカウントがある場合。** コマンドをひとつ実行し、表示されたリンクを開きます：

```sh
sno account claim
```

コンピューターは自動的に[コンピューター一覧ページ](https://www.sno.ai/dashboard/computers)に表示されます。

## 日常のコマンド

*最終更新 2026-10-08。* `sno` と入力すれば、すべてのコマンドが1行ずつ表示されます。2語より深いコマンドはありません。

![新しいコンピューターで sno と入力したところ：すべてのコマンドが1行ずつ表示される](../images/sno-overview.png)

| やりたいこと | 実行するコマンド |
|---|---|
| すべてのコマンドを見る | `sno` |
| すべて正常に動いているか確認する | `sno doctor` |
| Sno Station をインストールまたは修復する | `sno setup` |
| このコンピューターが共有するものを確認または変更する | `sno station consent` |
| このコンピューターをアカウントにリンクする | `sno account claim` |
| `sno` がインストールしたものを削除する | `sno uninstall` |
| ひとつのコマンドの詳細を読む | `sno <command> --help` |

各コマンドの説明つきの全一覧は [COMMANDS.md](../../COMMANDS.md) にあります。すべてのコマンドは `--json` で JSON も出力するので、エージェントもあなたと同じ答えを読めます。

## "更新の手順はありません。"

*最終更新 2026-10-08。*

私たちがこのツールに求めたことはひとつだけです。インストールしたら、あとはもう考えなくていいことです。

そこで `sno` は、自分で自分を最新に保ちます。毎日最初に実行するコマンドが、新しいリリースがあるかを確認します。あれば、それをインストールしてから、あなたが入力したコマンドを実行します。そのことについての表示は1行だけ、一度だけです。バックグラウンドで `sno` を呼び出すエージェントの出力の途中に、通知が割り込むことはありません。

ある手順が失敗したときは、何が失敗したか、なぜ失敗したか、そしてそれを直すひとつのコマンドを示します。最初からやり直すよう求めることはありません。あなたが手で編集したファイルは、あなたのもののままです。

## 現在動いているもの

*最終更新 2026-10-08。*

| 構成要素 | 状態 |
|---|---|
| Linux での1行インストール | リリース済み。クリーンな Linux マシンでこのページからインストール済み |
| エージェントに伝える「Install sno.ai from GitHub」 | Claude Code と Codex で動作。クリーンなマシンで試験済み |
| Claude Code、Codex、OpenClaw、Hermes Agent 向けの `sno setup` | リリース済み。クリーンな Linux マシンでエンドツーエンドで実証済み |
| 1日に一度の自動更新 | リリース済み。実際のリリース間を移行する Linux マシンで実証済み |
| macOS（Apple silicon と Intel） | リリース済み。Mac でビルドと実行を確認。Intel は Rosetta 経由 |
| Windows 上の WSL2 | Linux ビルドを使用 |

ある行が「実証済み」と言えるのは、クリーンなマシンで動作した後だけです。

## 次のステップ

- **エージェントたちがなるチーム：** [Sno Station](https://github.com/sno-ai/sno-station)。仕組み、何を覚えているか、昨夜何を学んだかを紹介しています。
- **エージェントがあなたのためにしたこと：** あなたの [sno.ai ダッシュボード](https://www.sno.ai/dashboard)。
- **すべてのコマンド：** [COMMANDS.md](../../COMMANDS.md)。
- **すでに2つのエージェントを並べて動かしていますか？** [design partner issue](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml) を開いて、何を動かしているか教えてください。

## セキュリティ

[Sno Station のセキュリティポリシー](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md)を参照してください。

## ライセンス

プログラムは Apache License 2.0 の下で公開されています。[LICENSE](../../LICENSE) を参照してください。ソースコードは公開されていません。このリポジトリには、ダウンロード、コマンド一覧、そして `sno setup` がインストールできる製品の一覧が置かれています。
