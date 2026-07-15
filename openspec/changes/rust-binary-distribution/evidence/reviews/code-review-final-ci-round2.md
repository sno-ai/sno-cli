# Codex Adversarial Review

Target: `.github/workflows/ci.yml`, `policy/test-substitutes.json`, `scripts/check-test-substitutes.sh`, `scripts/check-release-surfaces.sh`, `tests/support/sno_service_server.rs`
Verdict: needs-attention

Do not ship until the test service fixture cannot silently suppress failed request assertions; it can currently produce green CI despite an external-service contract test failing inside its handler.

Findings:
- [high] Detached fixture suppresses handler and protocol assertion failures (`tests/support/sno_service_server.rs`:90-96, confidence 0.98)
  Trigger: A test starts the service server, places request assertions in a handler, then lets the server drop rather than calling `finish()`. If the handler panics—or the server rejects a malformed request—the worker thread finishes; `Drop` joins it but discards the `Err`.
  Impact: The test function returns successfully and CI passes while the client sent an invalid request or violated the external service contract. Regressions can ship undetected.
  Recommendation: Make an unfinalized server fail the test on drop, or join the worker and propagate its panic when the current thread is not already panicking; retain `finish()` as the explicit successful completion path.

Next steps:
- Add a regression test proving a handler assertion failure fails the enclosing test even when the fixture is dropped.