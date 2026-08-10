# QCG-5 probe results

Read-only probes run before test implementation.

```text
$ test -x /home/lh/code/sno-station-core-edge-rem-wave/node_modules/.bin/tsx
exit 0

$ ls -l apps/mem-claw/src/sidecar/main.ts apps/mem-claw/dist/sidecar/main.js
source main.ts exists; built main.js exists

$ sha256sum /home/lh/code/sno-cli/target/debug/sno /home/lh/.cargo/bin/sno
907189b03d0453c7975947583bf015948dd59cbb57b33d50bfb1e78276626667  /home/lh/code/sno-cli/target/debug/sno
0d9671a8619c248f63cd227e36e32be4492a88e9f9728b3e445a55a97a769bde  /home/lh/.cargo/bin/sno

$ /home/lh/code/sno-cli/target/debug/sno --version
sno 0.1.7
```

Source facts:

- The production fixture spawns the real source entry with `tsx`, creates a
  temporary real encrypted SQLite store, and writes discovery beneath a
  temporary profile.
- The real sidecar GET route reads the durable store and returns `sendJson(200,
  job)`.
- The durable state schema accepts only `queued`, `running`, `done`, and
  `failed`; an unfamiliar live-store row prevents startup.
- `sendJson` calculates the exact content length and ends with the complete
  payload. There is no response truncation or unfamiliar-state fault switch.
- `SNO_REM_TEST_HOLD_MS` is an existing test-only delay read by the real
  sidecar.
- The current CLI retries a truncated response only in waiting mode. Therefore
  the truncated QCG-5 row must use a non-waiting status call to observe exit 6
  from one injected response.
