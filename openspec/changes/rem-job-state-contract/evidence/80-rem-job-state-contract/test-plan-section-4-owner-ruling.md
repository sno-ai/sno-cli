# Owner final admission ruling — Section 4

Verdict: proceed with QCG-3 and the exact QCG-4 actors.

The owner overrides the QCG-4 rejection in `test-plan-section-4-admission-review.md`. Its
duplication premise is false:

- The existing account test reaches a live loopback server returning HTTP 409; it does not prove
  `account machine register --json` against an unreachable loopback URL.
- The existing external-command test invokes `sno-demo` and exits `7`; it does not prove the real
  external executable selected by `sno service fail-runtime` still exits `1`.

The released QCG-4 requires those exact actors. Implement them as three focused checks rather than
one combined test:

1. unreachable-loopback `account machine register --json` exits `1`;
2. real `sno-service fail-runtime` executable exits `1`;
3. stopped-sidecar `rem-start` and `rem-status` both exit `7`, followed by scoped restoration and
   cleanup of the temporary sidecar state.

QCG-3 retains the reviewer's `ADMIT`. No additional admission review is authorized or required.
