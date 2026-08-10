# Release contract probe

Date: 2026-08-09

This evidence checks whether the old REM exit behavior reached a published `sno` release.

## Remote release tags

Command:

```sh
timeout 45s git ls-remote --tags origin
```

Result: the remote contains release tags `v0.1.1` through `v0.1.7`; the newest peeled release commit is `05c9f5b6e7e24b08347e9c605b8620eb80974ba9` for `v0.1.7`. No later release tag exists.

## REM introduction ancestry

Commands:

```sh
git tag --contains 5130fee
git merge-base --is-ancestor 05c9f5b6e7e24b08347e9c605b8620eb80974ba9 5130fee
git merge-base --is-ancestor 5130fee 05c9f5b6e7e24b08347e9c605b8620eb80974ba9
git show --stat --oneline 5130fee -- src/rem.rs src/cli.rs README.md
```

Results:

- `git tag --contains 5130fee` returned no tags.
- `v0.1.7` is an ancestor of `5130fee`; the reverse ancestry check failed.
- Commit `5130fee` is `feat(station): add asynchronous REM commands` and added `src/rem.rs` plus the REM CLI and README surface.

Conclusion: published releases exist, but none contained the REM commands or their old collapsed exit behavior. The REM contract can make the PRD-authorized hard cut without a compatibility path.
