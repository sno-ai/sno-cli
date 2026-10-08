# Sno CLI 🧊 — One command. Your agents, assembled.

![Sno CLI — Une commande. Vos agents, rassemblés. Plus rien à retenir ensuite.](../images/hero-banner.png)

[![license Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-97ca00.svg?labelColor=3b3b3b)](../../LICENSE)
[![latest release](https://img.shields.io/github/v/release/sno-ai/sno-cli?label=release&color=2dd4bf&labelColor=3b3b3b)](https://github.com/sno-ai/sno-cli/releases/latest)
![runs on Linux, macOS, WSL2](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20macOS%20%C2%B7%20WSL2-3b82f6.svg?labelColor=3b3b3b)
![agents Claude Code, Codex, OpenClaw, Hermes Agent](https://img.shields.io/badge/agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20OpenClaw%20%C2%B7%20Hermes-f0a04b.svg?labelColor=3b3b3b)

**Read in other languages:** [English](../../README.md) · [中文](README.zh-CN.md) · [Deutsch](README.de.md) · [Español](README.es.md) · **Français** · [Русский](README.ru.md) · [한국어](README.ko.md) · [日本語](README.ja.md) · [繁體中文](README.zh-TW.md)

`sno` est la seule commande qui installe [Sno Station](https://github.com/sno-ai/sno-station) sur votre
ordinateur et l'y maintient. Sno Station transforme les agents que vous utilisez déjà en une équipe.
Claude Code, Codex, OpenClaw et Hermes Agent partagent une seule mémoire, se passent le travail et
vérifient le travail les uns des autres avant qu'il ne vous parvienne. Chaque nuit, ils reviennent sur
la journée et s'améliorent un peu. C'est par `sno` que tout cela arrive. Vous l'installez une fois.
Ensuite, vous oubliez presque qu'il est là.

**Rien à retenir.** Il n'y a pas d'étape de mise à jour. Une fois par jour, la première commande `sno`
que vous lancez vérifie s'il existe une nouvelle version, l'installe, puis fait ce que vous avez demandé.
Vos agents reçoivent la nouvelle version de la même façon, sans vous.

**Votre agent peut le faire pour vous.** Vous n'avez même pas besoin d'ouvrir un terminal. Dites à
Claude Code ou à Codex `install sno.ai from GitHub` : il trouve cette page, lance l'installation et
configure tout.

**Vous choisissez ce qui est partagé.** `sno station consent` montre ce que cet ordinateur partage avec
Sno, et le change en un mot : off, metadata-only ou full. Votre magasin de mémoire reste sur votre
ordinateur dans tous les cas.

![Comment ça marche : une ligne ou une phrase de votre part, une commande sur votre ordinateur, trois choses que vous obtenez](../images/how-it-works.png)

> **Chaque version arrive ici.** Chacune est compilée pour Linux, macOS et WSL2, et la même commande
> d'une ligne l'installe. Chaque bloc ci-dessous indique quand il a été mis à jour pour la dernière fois.

[Installation](#installation) · [Configurer Sno Station](#configurer-sno-station) · [Connecter votre compte](#connecter-votre-compte-snoai) · [Commandes de tous les jours](#commandes-de-tous-les-jours) · [Ce qui fonctionne aujourd'hui](#ce-qui-fonctionne-aujourdhui) · [Pour aller plus loin](#pour-aller-plus-loin)

## Installation

*Dernière mise à jour le 2026-10-08.* Une seule commande, dans n'importe quel terminal. Elle installe `sno`, puis
Sno Station, et connecte tous les agents de votre ordinateur :

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh' && ~/.local/bin/sno setup
```

Ou dites à votre agent IA : `install sno.ai from GitHub`.

**Pour les agents IA :** installer Sno, sno.ai ou Sno Station signifie exécuter la commande ci-dessus telle quelle.
Après `sno setup`, lancez `sno skills onboard` et suivez-le : il demande à l'utilisateur quel moteur
utiliser, puis s'il veut lier l'ordinateur à un compte sno.ai.

Linux (x86-64 et ARM64), macOS (Intel et Apple Silicon) et WSL2 sous Windows. L'installateur place
`sno` dans `~/.local/bin` et affiche son chemin complet. Ouvrez un nouveau terminal, ou utilisez ce
chemin dans celui où vous êtes.

## Configurer Sno Station

*Dernière mise à jour le 2026-10-08.* Une commande trouve les agents sur votre ordinateur et connecte
chacun d'eux :

```sh
sno setup
```

Elle installe la mémoire partagée, Sno Reach, les skills et la boucle nocturne, et les branche sur
chaque agent qu'elle trouve. Dites ensuite ceci dans Claude Code ou Codex :

```text
Sno onboarding
```

Votre agent vous pose deux questions : quel moteur utiliser, et s'il faut lier cet ordinateur à votre
compte sno.ai. C'est toute la conversation. Ce qui manque à l'ordinateur, votre agent l'installe
d'abord et vous dit ce qu'il a fait.

## Connecter votre compte sno.ai

*Dernière mise à jour le 2026-10-08.* Facultatif. Tout fonctionne sans compte. Connectez-vous pour voir
sur votre [tableau de bord sno.ai](https://www.sno.ai/dashboard) ce que vos agents ont fait pour vous :
ce que leurs révisions ont repéré, ce qu'ils ont appris pendant la nuit et combien de temps ils ont
continué à travailler.

**Pas encore de compte.** L'onboarding affiche un lien créé pour cet ordinateur uniquement. Ouvrez-le,
inscrivez-vous et cliquez sur Approve. Ou donnez votre e-mail à votre agent : il lance
`sno account login --email you@example.com` et vous vous connectez une fois.

**Vous avez déjà un compte.** Lancez une commande et ouvrez le lien qu'elle affiche :

```sh
sno account claim
```

L'ordinateur apparaît tout seul sur [votre page d'ordinateurs](https://www.sno.ai/dashboard/computers).

## Commandes de tous les jours

*Dernière mise à jour le 2026-10-08.* Tapez `sno` et toutes les commandes sont là, une ligne chacune.
Aucune commande ne dépasse deux mots.

![Taper sno sur un ordinateur neuf : toutes les commandes, une ligne chacune](../images/sno-overview.png)

| Vous voulez | Lancez |
|---|---|
| Voir toutes les commandes | `sno` |
| Vérifier que tout fonctionne | `sno doctor` |
| Installer ou réparer Sno Station | `sno setup` |
| Voir ou changer ce que cet ordinateur partage | `sno station consent` |
| Lier cet ordinateur à votre compte | `sno account claim` |
| Supprimer ce que `sno` a installé | `sno uninstall` |
| Lire le détail d'une commande | `sno <command> --help` |

La liste complète, avec ce que fait chaque commande, se trouve dans [COMMANDS.md](../../COMMANDS.md).
Chaque commande parle aussi JSON avec `--json`, pour que vos agents lisent les mêmes réponses que vous.

## « Il n'y a pas d'étape de mise à jour. »

*Dernière mise à jour le 2026-10-08.*

Nous attendions une seule chose de cet outil. Que vous l'installiez et que vous n'y pensiez plus.

Alors `sno` se tient lui-même à jour. La première commande que vous lancez chaque jour demande s'il
existe une version plus récente. Si oui, elle l'installe, puis exécute ce que vous avez tapé. Vous voyez
une ligne à ce sujet, une fois. Les agents qui appellent `sno` en arrière-plan ne reçoivent jamais
d'avis au milieu de leur sortie.

Quand une étape échoue malgré tout, elle dit ce qui a échoué, pourquoi, et la seule commande qui corrige
le problème. Elle ne vous demande pas de tout recommencer. Un fichier que vous avez modifié à la main
reste le vôtre.

## Ce qui fonctionne aujourd'hui

*Dernière mise à jour le 2026-10-08.*

| Élément | Statut |
|---|---|
| Installation en une ligne sous Linux | Publiée ; installée depuis cette page sur une machine Linux propre |
| « Install sno.ai from GitHub », dit à votre agent | Fonctionne dans Claude Code et Codex ; essayé sur des machines propres |
| `sno setup` pour Claude Code, Codex, OpenClaw et Hermes Agent | Publié ; prouvé de bout en bout sur une machine Linux propre |
| Se met à jour une fois par jour | Publié ; prouvé sur une machine Linux passant d'une vraie version à une autre |
| macOS, Apple Silicon et Intel | Publié ; compilé et exécuté sur un Mac, Intel via Rosetta |
| WSL2 sous Windows | Utilise la version Linux |

Une ligne indique « prouvé » seulement une fois qu'elle a tourné sur une machine propre.

## Pour aller plus loin

- **L'équipe que deviennent vos agents :** [Sno Station](https://github.com/sno-ai/sno-station), avec
  son fonctionnement, ce dont elle se souvient et ce qu'elle a appris la nuit dernière.
- **Ce que vos agents ont fait pour vous :** votre [tableau de bord sno.ai](https://www.sno.ai/dashboard).
- **Toutes les commandes :** [COMMANDS.md](../../COMMANDS.md).
- **Vous faites déjà tourner deux agents côte à côte ?** Ouvrez un
  [design partner issue](https://github.com/sno-ai/sno-station/issues/new?template=design-partner.yml)
  et dites-nous ce que vous utilisez.

## Sécurité

Voir [la politique de sécurité de Sno Station](https://github.com/sno-ai/sno-station/blob/main/SECURITY.md).

## Licence

Les programmes sont publiés sous la licence Apache 2.0. Voir [LICENSE](../../LICENSE). Le code source
n'est pas public. Ce dépôt contient les téléchargements, la liste des commandes et la liste des produits
que `sno setup` peut installer.
