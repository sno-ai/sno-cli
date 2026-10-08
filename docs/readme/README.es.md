# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — Un comando. Tus agentes, reunidos. Después, nada que recordar.](../images/hero-banner.png)

[![license Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](../../LICENSE)
[![latest release](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![runs on Linux, macOS, WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![agents Claude Code, Codex, OpenClaw, Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** [English](../../README.md) · [中文](README.zh-CN.md) · [Deutsch](README.de.md) · **Español** · [Français](README.fr.md) · [Русский](README.ru.md) · [한국어](README.ko.md) · [日本語](README.ja.md) · [繁體中文](README.zh-TW.md)

`sno` es el único comando que pone [Sno Station](https://github.com/sno-ai/sno-station) en tu computadora
y hace que siga ahí. Sno Station convierte los agentes que ya usas en un equipo. Claude Code, Codex,
OpenClaw y Hermes Agent comparten una memoria, se pasan el trabajo y se revisan el trabajo
mutuamente antes de que llegue a ti. Cada noche repasan el día y mejoran un poco.
`sno` es la forma en que llega todo eso. Lo instalas una vez. Después, casi olvidas que está ahí.

**Nada que recordar.** No hay paso de actualización. Una vez al día, el primer comando `sno` que
ejecutas busca una nueva versión, la instala y luego hace lo que pediste. Tus agentes reciben la
nueva versión de la misma forma, sin ti.

**Tu agente puede hacerlo por ti.** No tienes que abrir una terminal. Dile a Claude Code o
Codex `install sno.ai from GitHub` y encontrará esta página, ejecutará la instalación y lo configurará todo.

**Tú eliges qué comparte.** `sno station consent` muestra qué comparte esta computadora con Sno, y
lo cambia con una palabra: off, metadata-only o full. Tu almacén de memoria se queda en tu computadora en cualquier caso.

![Cómo funciona: una línea o una frase tuya, un comando en tu computadora, tres cosas que obtienes](../images/how-it-works.png)

> **Cada versión llega aquí.** Cada una se compila para Linux, macOS y WSL2, y el mismo comando de
> una línea la instala. Cada bloque a continuación dice cuándo se actualizó por última vez.

[Instalación](#instalación) · [Configura Sno Station](#configura-sno-station) · [Conecta tu cuenta](#conecta-tu-cuenta-de-snoai) · [Comandos de uso diario](#comandos-de-uso-diario) · [Qué funciona hoy](#qué-funciona-hoy) · [Siguientes pasos](#siguientes-pasos)

## Instalación

*Última actualización 2026-10-08.* Una línea, en cualquier terminal:

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
```

O dile a tu agente de IA: `install sno.ai from GitHub`.

Linux (x86-64 y ARM64), macOS (Intel y Apple silicon) y WSL2 en Windows. El instalador pone
`sno` en `~/.local/bin` e imprime su ruta completa. Abre una terminal nueva, o usa esa ruta en la
que ya tienes abierta.

## Configura Sno Station

*Última actualización 2026-10-08.* Un comando encuentra los agentes de tu computadora y conecta cada uno de ellos:

```sh
sno setup
```

Instala la memoria compartida, Sno Reach, las skills y el bucle nocturno, y los conecta a cada
agente que encuentra. Después di esto dentro de Claude Code o Codex:

```text
Sno onboarding
```

Tu agente te pregunta dos cosas: qué motor usar y si quieres vincular esta computadora a tu
cuenta de sno.ai. Esa es toda la conversación. Si a la computadora le falta algo, tu agente
lo instala primero y te dice qué hizo.

## Conecta tu cuenta de sno.ai

*Última actualización 2026-10-08.* Opcional. Todo funciona sin cuenta. Conéctala para ver en tu
[panel de sno.ai](https://www.sno.ai/dashboard) lo que tus agentes hicieron por ti: qué detectaron
sus revisiones, qué aprendieron durante la noche y cuánto tiempo siguieron trabajando.

**Todavía no tienes cuenta.** El onboarding muestra un enlace creado solo para esta computadora. Ábrelo,
regístrate y haz clic en Approve. O dale tu correo a tu agente: ejecuta `sno account login --email you@example.com` y tú
inicias sesión una vez.

**Ya tienes cuenta.** Ejecuta un comando y abre el enlace que imprime:

```sh
sno account claim
```

La computadora aparece sola en [tu página de computadoras](https://www.sno.ai/dashboard/computers).

## Comandos de uso diario

*Última actualización 2026-10-08.* Escribe `sno` y ahí están todos los comandos, una línea cada uno.
Ningún comando tiene más de dos palabras.

![Escribir sno en una computadora recién instalada: todos los comandos, una línea cada uno](../images/sno-overview.png)

| Quieres | Ejecuta |
|---|---|
| Ver todos los comandos | `sno` |
| Comprobar que todo funciona | `sno doctor` |
| Instalar o reparar Sno Station | `sno setup` |
| Ver o cambiar qué comparte esta computadora | `sno station consent` |
| Vincular esta computadora a tu cuenta | `sno account claim` |
| Quitar lo que instaló `sno` | `sno uninstall` |
| Leer los detalles de un comando | `sno <command> --help` |

La lista completa, con lo que hace cada comando, está en [COMMANDS.md](../../COMMANDS.md). Cada comando
también habla JSON con `--json`, así que tus agentes leen las mismas respuestas que tú.

## "No hay paso de actualización."

*Última actualización 2026-10-08.*

Queríamos una sola cosa de esta herramienta. Que la instalaras y dejaras de pensar en ella.

Así que `sno` se mantiene al día solo. El primer comando que ejecutas cada día pregunta si existe
una versión más nueva. Si existe, la instala y luego ejecuta lo que escribiste. Ves una línea sobre ello, una vez.
Los agentes que llaman a `sno` en segundo plano nunca reciben un aviso en medio de su salida.

Cuando un paso falla, dice qué falló, por qué y el único comando que lo arregla. No te pide
empezar de nuevo. Un archivo que editaste a mano sigue siendo tuyo.

## Qué funciona hoy

*Última actualización 2026-10-08.*

| Pieza | Estado |
|---|---|
| Instalación de una línea en Linux | Publicada; instalada desde esta página en una máquina Linux limpia |
| «Install sno.ai from GitHub», dicho a tu agente | Funciona en Claude Code y Codex; ensayado en máquinas limpias |
| `sno setup` para Claude Code, Codex, OpenClaw y Hermes Agent | Publicado; probado de extremo a extremo en una máquina Linux limpia |
| Se actualiza solo una vez al día | Publicado; probado en una máquina Linux que pasó entre versiones reales |
| macOS, Apple silicon e Intel | Publicado; compilado y ejecutado en un Mac, Intel mediante Rosetta |
| WSL2 en Windows | Usa la compilación de Linux |

Una fila dice "probado" solo cuando se ha ejecutado en una máquina limpia.

## Siguientes pasos

- **El equipo en que se convierten tus agentes:** [Sno Station](https://github.com/sno-ai/sno-station), con cómo
  funciona, qué recuerda y qué aprendió anoche.
- **Lo que tus agentes hicieron por ti:** tu [panel de sno.ai](https://www.sno.ai/dashboard).
- **Todos los comandos:** [COMMANDS.md](../../COMMANDS.md).
- **¿Ya usas dos agentes en paralelo?** Abre un
  [design partner issue](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml)
  y cuéntanos qué usas.

## Seguridad

Consulta la [política de seguridad de Sno Station](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md).

## Licencia

Los programas se publican bajo la Apache License 2.0. Consulta [LICENSE](../../LICENSE). El código fuente no
es público. Este repositorio contiene las descargas, la lista de comandos y la lista de productos que `sno setup`
puede instalar.
