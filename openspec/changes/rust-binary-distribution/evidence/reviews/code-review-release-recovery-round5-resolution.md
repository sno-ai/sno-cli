# Release Recovery Final Review Resolution

## High tag-movement finding

Status: `[Possible FP]`, resolved by external repository enforcement that was outside the review payload.

Repository ruleset `18984000`, `Protect version release tags`, is active for `refs/tags/v*`. Its rules restrict creation, update, deletion, and non-fast-forward changes. The sole bypass actor is `OrganizationAdmin` with `always` mode. `scripts/authorize-release.sh` refuses authorization unless this exact live ruleset, public visibility, immutable releases, remote `main`, and successful CI all pass. The release workflow additionally resolves the tag immediately before draft creation and publication.

## High flow-style-permission finding

Status: fixed after the fifth and final authoritative review iteration.

Permission validation now parses the workflow as YAML through the repository's locked Rust policy tool and inspects every job permission mapping independently of block or flow syntax. A new `permissions: { contents: write }` mutation on the untrusted plan job is rejected. The prior block-style and `write-all` mutations remain covered.

No sixth Codex adversarial-review iteration was run because the skill limit is five. Local schema-aware mutation tests, actionlint, Rust tests, and live repository ruleset evidence are the remaining verification authorities for these fixes.
