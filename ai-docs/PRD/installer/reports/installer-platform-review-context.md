# Platform selection patch context

Owner ruling on 2026-09-14 changes Reach archive names to reach-<VERSION>-<os>-<arch>.tar.gz plus .sha256. Literal Rust OS/ARCH values support linux/macos times x86_64/aarch64. Utility names remain unchanged. Unsupported architecture must return usage exit2 naming the pair before ANY release fetch. Native Windows is intentionally refused; other CLI features still support Windows and CI runs native Windows tests.

Review only this delta. Production src/assemble.rs adds three private selection helpers, extracts the existing resolve body into resolve_for_platform, validates before downloads and passes actual std::env::consts values from ReleaseSource::resolve. Checksum lookup is extracted without semantic change. New platform tests are included privately under cfg(test). src/manifest.rs Windows refusal is untouched. No published release or live harness acceptance is claimed. Old fixture archives are intentionally retained for historical local proofs.

The PRD patch touches DEC-3 and the QCG-1 platform assertion, bumps to1.5, and leaves head seal unchanged. Publication/live QCG-9 remains blocked. This is new behavior from a revised owner contract, not a claim that the earlier unqualified contract was incorrectly implemented.

Primary failure costs: fetching the wrong platform or utility name, accepting wrong checksum pair, contacting network before rejecting unsupported architecture, or making normal native CI fail. Existing unrelated installer logic is outside this patch. No full PRD or repository review is requested.
