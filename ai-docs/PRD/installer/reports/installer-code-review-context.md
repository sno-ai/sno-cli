# Installer code review context

Review only source changes in this execution. New files assemble.rs, manifest.rs, harness_slots.rs are entirely in scope. cli.rs adds 4 verbs and source injection; doctor.rs extracts its existing report for root aggregation; lib.rs adds module exports. Existing unrelated code in these files is out of scope. No PRD review.

Ordinary paths: assemble verifies three core program artifacts and category-S declarations; refuses unowned destination; journals modifications; integrates reminder entries preserving user entries; rerun unchanged does not mutate; update then remove; crash pre/post manifest; optional scheduler enable/off/rollback. Source trait is external release transport; checkpoint is observation/failure instrumentation for subprocess interruption fixtures, no production env fault knobs. Shell scheduler substitute exists only in tests.

Actual core utility releases, real skills gate bytes/shared fixture, and live harness discovery/notification are unavailable. Synthetic fixture green is not staged or published-release proof. Do not claim these external boundaries are verified. Host gpt1, repository /home/lh/code/sno-cli. No worktrees.

Existing-file diff:
```diff
diff --git a/Cargo.toml b/Cargo.toml
index 0a91ef3..5b023bb 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -32,9 +32,11 @@ rand = "0.9"
 reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
 rusqlite = { version = "0.32", features = ["bundled"] }
 serde = { version = "1.0", features = ["derive"] }
-serde_json = { version = "1.0", features = ["preserve_order"] }
+serde_json = { version = "1.0", features = ["preserve_order", "raw_value"] }
+serde_yaml = "0.9"
 sha2 = "0.10"
 tar = "0.4"
+toml = "0.8"
 url = "2.5"
 uuid = { version = "1.11", features = ["v7"] }
 
diff --git a/src/cli.rs b/src/cli.rs
index 21f3320..c6347b9 100644
--- a/src/cli.rs
+++ b/src/cli.rs
@@ -15,8 +15,9 @@ use crate::state::{self, ConsentValue};
 #[path = "rem.rs"]
 mod rem;
 
-const RETIRED_ROOT_COMMANDS: &[&str] =
-    &["consent", "observe", "register", "claim", "audit", "doctor", "starport"];
+const RETIRED_ROOT_COMMANDS: &[&str] = &[
+    "consent", "observe", "register", "claim", "audit", "starport",
+];
 
 #[derive(Debug, Parser)]
 #[command(
@@ -35,6 +36,17 @@ struct SnoCli {
 
 #[derive(Debug, Subcommand)]
 enum RootCommand {
+    #[command(about = "Install core programs and agent skills")]
+    Assemble(crate::assemble::InstallOptions),
+    #[command(about = "Update installed programs and skills")]
+    Update(crate::assemble::UpdateOptions),
+    #[command(about = "Check installed programs, skills and station state")]
+    Doctor,
+    #[command(about = "Remove installer-owned files")]
+    Remove {
+        #[arg(long)]
+        purge_state: bool,
+    },
     #[command(about = "Manage account and machine identity")]
     Account {
         #[command(subcommand)]
@@ -144,6 +156,15 @@ enum AuditCommand {
 }
 
 pub fn run<I, T>(arguments: I) -> i32
+where
+    I: IntoIterator<Item = T>,
+    T: Into<OsString> + Clone,
+{
+    run_with_source(arguments, &crate::assemble::GithubSource)
+}
+
+/// Run the same CLI with an explicit release transport, including isolated fixture transports.
+pub fn run_with_source<I, T>(arguments: I, source: &dyn crate::assemble::ReleaseSource) -> i32
 where
     I: IntoIterator<Item = T>,
     T: Into<OsString> + Clone,
@@ -156,7 +177,7 @@ where
     };
     let result = match parsed.command {
         None => return print_missing_command(parsed.json),
-        Some(command) => dispatch(command, parsed.json),
+        Some(command) => dispatch(command, parsed.json, source),
     };
     match result {
         Ok(exit_code) => exit_code,
@@ -164,8 +185,32 @@ where
     }
 }
 
-fn dispatch(command: RootCommand, json_enabled: bool) -> Result<i32, CliError> {
+fn dispatch(
+    command: RootCommand,
+    json_enabled: bool,
+    source: &dyn crate::assemble::ReleaseSource,
+) -> Result<i32, CliError> {
     match command {
+        RootCommand::Assemble(options) => Ok(crate::assemble::run(
+            crate::assemble::Action::Assemble(options),
+            json_enabled,
+            source,
+        )),
+        RootCommand::Update(options) => Ok(crate::assemble::run(
+            crate::assemble::Action::Update(options),
+            json_enabled,
+            source,
+        )),
+        RootCommand::Doctor => Ok(crate::assemble::run(
+            crate::assemble::Action::Doctor,
+            json_enabled,
+            source,
+        )),
+        RootCommand::Remove { purge_state } => Ok(crate::assemble::run(
+            crate::assemble::Action::Remove { purge_state },
+            json_enabled,
+            source,
+        )),
         RootCommand::Account { command } => dispatch_account(command, json_enabled),
         RootCommand::SnoStation { command } => dispatch_sno_station(command, json_enabled),
         RootCommand::External(arguments) => dispatch_external(arguments),
diff --git a/src/doctor.rs b/src/doctor.rs
index 1bd6e9c..eac5598 100644
--- a/src/doctor.rs
+++ b/src/doctor.rs
@@ -35,7 +35,7 @@ enum CheckStatus {
     Fail,
 }
 
-pub fn run(json_enabled: bool) -> Result<i32, CliError> {
+fn inspect() -> Result<(DoctorReport, i32), CliError> {
     let paths = SnoPaths::from_environment()?;
     let (buffer, shipped_count) = check_buffer(&paths.buffer_path);
     let report = DoctorReport {
@@ -50,6 +50,16 @@ pub fn run(json_enabled: bool) -> Result<i32, CliError> {
         || report.consent.status != CheckStatus::Ok
         || report.last_ship.status != CheckStatus::Ok
         || report.lockfile.status != CheckStatus::Ok;
+    Ok((report, if has_issue { 1 } else { 0 }))
+}
+
+pub(crate) fn report() -> Result<(serde_json::Value, i32), CliError> {
+    let (report, exit) = inspect()?;
+    Ok((serde_json::to_value(report)?, exit))
+}
+
+pub fn run(json_enabled: bool) -> Result<i32, CliError> {
+    let (report, exit) = inspect()?;
     if json_enabled {
         print_json(&serde_json::to_value(&report)?)?;
     } else {
@@ -68,7 +78,7 @@ pub fn run(json_enabled: bool) -> Result<i32, CliError> {
             println!("{badge} {}", check.detail);
         }
     }
-    Ok(if has_issue { 1 } else { 0 })
+    Ok(exit)
 }
 
 fn check_identity(path: &Path) -> DoctorCheck {
diff --git a/src/lib.rs b/src/lib.rs
index 0d1997a..79c3df8 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,7 +1,10 @@
-mod cli;
+pub mod assemble;
+pub mod cli;
 mod doctor;
 mod error;
 mod export;
+pub mod harness_slots;
+mod manifest;
 mod rem_outcome;
 mod service;
 mod state;

```
