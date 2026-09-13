# Installer fixture boundary

`requirements-contract.json` is the exact JSON example in the category-S PRD section 5.7,
including its final newline. SHA-256:
`5c29a218bd7c3d43003fad6f0e3939914a3f82ede18eb60f2e5f31a3b9a11b92`.

`tests/assemble.rs` constructs synthetic program archives and four synthetic skill units
in each test's temporary directory. Program versions and promotion stamps are test data.
The source provider only substitutes remote release resolution and downloaded bytes.
The generated archive includes a synthetic four-unit `s-category-requirements.json`.
Its declarations and file hashes bind the same generated skill bytes; mutation cases
refresh valid bindings before altering the one value under test.
The CLI parser, installer, archive reader, hook merge, locking, manifest, transactions,
and filesystem are real. A subprocess supplies an isolated HOME and PATH.

The source checkpoint observer returns a test error or pauses after a durable write.
Recovery cases kill that real installer process, then invoke the next real CLI command.
The observer cannot repair, generate, or mutate installer state.
`systemctl` substitutes only the external scheduler. It records arguments and an active
state marker so failed or interrupted changes can prove restoration of both scheduler
state and the real installer manifest. Enabled state and running state are separate;
the substitute checks the exact unit and only starts or stops it with `--now`.

`driver.rs` is compiled against the current test build of `sno`. Its ordinary `main`
calls the public CLI parser with a file-backed source provider. The scheduled-command
test runs the generated `ExecStart` under only its declared HOME/PATH and observes a
real installed-version change. It does not ask the native scheduler to trigger the job.

Run the driver-backed scheduled-command check through the integration harness; it compiles
the driver and creates its isolated source table automatically:

```sh
cargo test --test assemble generated_scheduler_command_runs_the_actual_cli_and_updates_the_selected_home -- --exact
```

The ignored `accepted_core_utilities_install_with_synthetic_reach_and_skills` case must
be selected explicitly with paths to accepted local core archives. It proves those actual
utility bytes cross the installer boundary, while Reach and skill data remain synthetic.
Missing paths fail loudly when this case is selected. It is not publication evidence.

These tests do not prove actual upstream publication, gate-produced declaration fixtures,
live harness skill discovery, adapter readiness, hook injection, or native scheduler triggering.
Those remain separate staged-consumer and published-release acceptance obligations.
The `fixture_dispatch` test is a subprocess entry point, not an acceptance case.
