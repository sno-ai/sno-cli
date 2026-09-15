#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
use sno::assemble::{Artifact, InstallError, ReleaseSet, ReleaseSource};
use tempfile::TempDir;

// Synthetic releases exercise the real parser and filesystem. They are not upstream proof.
const CONTRACT: &[u8] = include_bytes!("fixtures/assemble/requirements-contract.json");
const REQUIREMENTS_FIXTURE: &str = "scripts/fixtures/s-category-requirements.json";
const USER_HOOK: &str =
    r#"{ "matcher" : "", "hooks" : [ { "type": "command", "command": "echo user-owned-hook" } ] }"#;

struct FileSource(PathBuf);

impl ReleaseSource for FileSource {
    fn resolve(
        &self,
        reach_version: Option<&str>,
        skills_version: Option<&str>,
    ) -> Result<ReleaseSet, InstallError> {
        let data: Value = serde_json::from_slice(&fs::read(&self.0).unwrap()).unwrap();
        let artifact = |v: &Value| Artifact {
            name: v["name"].as_str().unwrap().to_owned(),
            version: v["version"].as_str().unwrap().to_owned(),
            url: v["url"].as_str().unwrap().to_owned(),
            sha256: v["sha256"].as_str().unwrap().to_owned(),
            entry_point: v["entry_point"].as_str().unwrap().to_owned(),
        };
        let programs: Vec<_> = data["programs"]
            .as_array()
            .unwrap()
            .iter()
            .map(artifact)
            .collect();
        let skills = artifact(&data["skills"]);
        if let Some(version) = reach_version {
            assert_eq!(
                programs.iter().find(|p| p.name == "reach").unwrap().version,
                version
            );
        }
        if let Some(version) = skills_version {
            assert_eq!(skills.version, version);
        }
        Ok(ReleaseSet {
            programs,
            skills,
            contract_sha256: hex::encode(Sha256::digest(CONTRACT)),
        })
    }

    fn fetch(&self, url: &str) -> Result<Vec<u8>, InstallError> {
        let path = url::Url::parse(url).unwrap().to_file_path().unwrap();
        Ok(fs::read(path).unwrap())
    }

    fn checkpoint(&self, phase: &str) -> Result<(), InstallError> {
        if std::env::var("SNO_TEST_CHECKPOINT").ok().as_deref() != Some(phase) {
            return Ok(());
        }
        if std::env::var("SNO_TEST_CHECKPOINT_MODE").as_deref() == Ok("fail") {
            return Err(InstallError::state(
                "synthetic write failure after activation",
            ));
        }
        fs::write(
            std::env::var_os("SNO_TEST_CHECKPOINT_MARKER").unwrap(),
            phase,
        )
        .unwrap();
        std::thread::sleep(Duration::from_secs(20));
        Err(InstallError::state("test checkpoint was not released"))
    }
}

#[test]
fn fixture_dispatch() {
    let Some(source) = std::env::var_os("SNO_TEST_ASSEMBLE_SOURCE") else {
        return;
    };
    let args: Vec<String> =
        serde_json::from_str(&std::env::var("SNO_TEST_ASSEMBLE_ARGS").unwrap()).unwrap();
    std::process::exit(sno::cli::run_with_source(args, &FileSource(source.into())));
}

struct Fixture {
    root: TempDir,
    home: PathBuf,
    source: PathBuf,
    restrictive_umask: bool,
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        let home = root.path().join("home");
        fs::create_dir(&home).unwrap();
        let home = home.canonicalize().unwrap();
        let mut fixture = Self {
            source: root.path().join("source.json"),
            root,
            home,
            restrictive_umask: false,
        };
        fixture.release("2.0");
        fixture
    }

    fn release(&mut self, reach_version: &str) {
        let mut programs = Vec::new();
        for (name, command, version) in [
            ("reach", "sno-reach", reach_version),
            ("heartbeat", "heartbeat", "1.0"),
            (
                "subscription-quota-check",
                "subscription-quota-check",
                "1.0",
            ),
        ] {
            let reminder = if name == "reach" {
                "if [ \"$1\" = remind ]; then\n  printf '%s\\n' \"$$\" > \"$HOME/remind-pid\"\n  case \"${SNO_TEST_REMIND_MODE:-}\" in\n    fail) echo 'synthetic Reach remind failed' >&2; exit 7 ;;\n    hang) exec /bin/sleep 30 ;;\n  esac\nfi\n"
            } else {
                ""
            };
            let script = format!("#!/bin/sh\n{reminder}printf '%s\\n' '{version}'\n");
            let entry = format!("bin/{command}");
            let release_manifest = serde_json::to_vec(&json!({"program": name, "version": version, "requirements_contract_sha256": hex::encode(Sha256::digest(CONTRACT)), "dependencies": []})).unwrap();
            let mut files: Vec<(&str, &[u8], u32)> = vec![
                (&entry, script.as_bytes(), 0o755),
                ("VERSION", version.as_bytes(), 0o644),
                ("lib/resource.txt", b"synthetic resource\n", 0o644),
                ("vendor/resource.txt", b"synthetic vendor\n", 0o644),
                ("spec/schema.json", b"{}\n", 0o644),
                ("guide/README.md", b"Synthetic guide.\n", 0o644),
                ("NOTICE", b"Synthetic test fixture\n", 0o644),
                ("LICENSE", b"Synthetic test fixture\n", 0o644),
            ];
            if name != "reach" {
                files.push(("release.json", release_manifest.as_slice(), 0o644));
                files.push(("requirements-contract.json", CONTRACT, 0o644));
            }
            let bytes = archive(&files);
            programs.push(self.artifact(name, version, &entry, &bytes));
        }
        let mut skill_files = vec![(
            "scripts/requirements-contract.json".to_owned(),
            CONTRACT.to_vec(),
            0o644,
        )];
        let mut declarations = Vec::new();
        for (name, programs, slots) in [
            (
                "reach",
                vec![("reach", "2.0")],
                vec![
                    ("4.shell", "required"),
                    ("2.pre-turn-context-injection", "preferred"),
                ],
            ),
            (
                "handoff",
                vec![("reach", "2.0")],
                vec![("4.shell", "required"), ("4.file-read-write", "required")],
            ),
            (
                "heartbeat",
                vec![("heartbeat", "1.0")],
                vec![
                    ("4.shell", "required"),
                    ("4.background-processes", "required"),
                    ("3.reader-to-agent-delivery", "required"),
                ],
            ),
            (
                "subscription-quota-check",
                vec![("subscription-quota-check", "1.0"), ("heartbeat", "1.0")],
                vec![
                    ("4.shell", "required"),
                    ("4.background-processes", "required"),
                    ("3.reader-to-agent-delivery", "required"),
                ],
            ),
        ] {
            let programs: Vec<_> = programs
                .iter()
                .map(|(name, version)| json!({"name":name,"min_version":version}))
                .collect();
            let harness: Vec<_> = slots
                .iter()
                .map(|(slot, need)| json!({"slot":slot,"need":need}))
                .collect();
            let requirements = json!({"programs": programs, "harness": harness});
            let skill = format!(
                "---\nname: {name}\ndescription: Synthetic installer fixture\nrequires: {requirements}\n---\nRead references/usage.md.\n"
            );
            let prefix = format!("skills/S-communication-and-handoff/{name}");
            declarations.push(json!({"unit": name, "skill_path": format!("{prefix}/skill/SKILL.md"), "skill_sha256": hex::encode(Sha256::digest(skill.as_bytes())), "requires": requirements}));
            skill_files.push((
                format!("{prefix}/skill/SKILL.md"),
                skill.into_bytes(),
                0o644,
            ));
            skill_files.push((
                format!("{prefix}/skill/references/usage.md"),
                b"Synthetic instructions.\n".to_vec(),
                0o644,
            ));
            let stamp = json!({"unit":name,"category":"S","tier":"shipped","split":{},"source":{"repo":"sno-skills","commit":"0000000000000000000000000000000000000000","unit_tree_clean":true},"selftests_run":1,"contract_sha256":hex::encode(Sha256::digest(CONTRACT)),"payload_sha256":payload_digest(&skill_files, &prefix),"promoted_at":"2026-09-13T00:00:00Z"})
                .to_string();
            skill_files.push((format!("{prefix}/PROMOTED.json"), stamp.into_bytes(), 0o644));
        }
        let declaration_fixture = json!({"schema_version": 1, "contract_sha256": hex::encode(Sha256::digest(CONTRACT)), "source": {"repo": "sno-skills", "commit": "0000000000000000000000000000000000000000"}, "units": declarations});
        skill_files.push((
            REQUIREMENTS_FIXTURE.into(),
            serde_json::to_vec(&declaration_fixture).unwrap(),
            0o644,
        ));
        let refs: Vec<_> = skill_files
            .iter()
            .map(|(path, bytes, mode)| (path.as_str(), bytes.as_slice(), *mode))
            .collect();
        let bytes = archive(&refs);
        let skills = self.artifact("skills", "1.0", "", &bytes);
        fs::write(
            &self.source,
            serde_json::to_vec(&json!({"programs": programs, "skills": skills})).unwrap(),
        )
        .unwrap();
    }

    fn artifact(&self, name: &str, version: &str, entry: &str, bytes: &[u8]) -> Value {
        let path = self.root.path().join(format!("{name}-{version}.tar.gz"));
        fs::write(&path, bytes).unwrap();
        json!({"name":name,"version":version,"url":url::Url::from_file_path(path).unwrap().as_str(),"sha256":hex::encode(Sha256::digest(bytes)),"entry_point":entry})
    }

    fn large_reach_resource(&self, resource: &[u8]) {
        let mut source: Value = serde_json::from_slice(&fs::read(&self.source).unwrap()).unwrap();
        let artifact = &source["programs"][0];
        assert_eq!(artifact["name"], "reach");
        let version = artifact["version"].as_str().unwrap().to_owned();
        let path = url::Url::parse(artifact["url"].as_str().unwrap())
            .unwrap()
            .to_file_path()
            .unwrap();
        let bytes = fs::read(path).unwrap();
        let mut original = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for entry in original.entries().unwrap() {
            let mut entry = entry.unwrap();
            let header = entry.header().clone();
            tar.append(&header, &mut entry).unwrap();
        }
        let mut header = tar::Header::new_gnu();
        header.set_size(resource.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "lib/large.dat", resource)
            .unwrap();
        let bytes = tar.into_inner().unwrap().finish().unwrap();
        source["programs"][0] = self.artifact("reach", &version, "bin/sno-reach", &bytes);
        fs::write(&self.source, serde_json::to_vec(&source).unwrap()).unwrap();
    }

    fn edit_program(&self, name: &str, edit: impl FnOnce(&mut Vec<(String, Vec<u8>, u32)>)) {
        let mut source: Value = serde_json::from_slice(&fs::read(&self.source).unwrap()).unwrap();
        let index = source["programs"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| a["name"] == name)
            .unwrap();
        let old = source["programs"][index].clone();
        let path = url::Url::parse(old["url"].as_str().unwrap())
            .unwrap()
            .to_file_path()
            .unwrap();
        let bytes = fs::read(path).unwrap();
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
        let mut files = Vec::new();
        for entry in tar.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path = entry.path().unwrap().to_str().unwrap().to_owned();
            let mode = entry.header().mode().unwrap();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            files.push((path, bytes, mode));
        }
        edit(&mut files);
        let refs: Vec<_> = files
            .iter()
            .map(|(path, bytes, mode)| (path.as_str(), bytes.as_slice(), *mode))
            .collect();
        source["programs"][index] = self.artifact(
            name,
            old["version"].as_str().unwrap(),
            old["entry_point"].as_str().unwrap(),
            &archive(&refs),
        );
        fs::write(&self.source, serde_json::to_vec(&source).unwrap()).unwrap();
    }

    fn edit_skills(&self, edit: impl FnOnce(&mut Vec<(String, Vec<u8>, u32)>)) {
        let mut source: Value = serde_json::from_slice(&fs::read(&self.source).unwrap()).unwrap();
        let path = url::Url::parse(source["skills"]["url"].as_str().unwrap())
            .unwrap()
            .to_file_path()
            .unwrap();
        let bytes = fs::read(path).unwrap();
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
        let mut files = Vec::new();
        for entry in tar.entries().unwrap() {
            let mut entry = entry.unwrap();
            let name = entry.path().unwrap().to_str().unwrap().to_owned();
            let mode = entry.header().mode().unwrap();
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).unwrap();
            files.push((name, contents, mode));
        }
        edit(&mut files);
        let refs: Vec<_> = files
            .iter()
            .map(|(path, bytes, mode)| (path.as_str(), bytes.as_slice(), *mode))
            .collect();
        source["skills"] = self.artifact("skills", "1.0", "", &archive(&refs));
        fs::write(&self.source, serde_json::to_vec(&source).unwrap()).unwrap();
    }

    fn reach_declaration(&self, declaration: &str) {
        self.edit_skills(|files| {
            let skill = format!("---\nname: reach\ndescription: Synthetic declaration case\n{declaration}\n---\nSynthetic text.\n").into_bytes();
            let skill_sha = hex::encode(Sha256::digest(&skill));
            for (name, contents, _) in files.iter_mut() {
                if name.ends_with("/reach/skill/SKILL.md") { *contents = skill.clone(); }
                if name == REQUIREMENTS_FIXTURE {
                    let mut fixture: Value = serde_json::from_slice(contents).unwrap();
                    let entry = fixture["units"].as_array_mut().unwrap().iter_mut().find(|entry| entry["unit"] == "reach").unwrap();
                    entry["skill_sha256"] = json!(skill_sha);
                    if let Ok(parsed) = serde_yaml::from_str::<Value>(declaration) {
                        if let Some(requirements) = parsed.get("requires") { entry["requires"] = requirements.clone(); }
                    }
                    *contents = serde_json::to_vec(&fixture).unwrap();
                }
            }
            let digest = payload_digest(files, "skills/S-communication-and-handoff/reach");
            for (name, contents, _) in files.iter_mut() {
                if name.ends_with("/reach/PROMOTED.json") {
                    let mut stamp: Value = serde_json::from_slice(contents).unwrap();
                    stamp["payload_sha256"] = Value::String(digest.clone());
                    *contents = serde_json::to_vec(&stamp).unwrap();
                }
            }
        });
    }

    fn harness(&self, name: &str) {
        use std::os::unix::fs::PermissionsExt;
        let executable = write_file(
            &self.home,
            &format!(".local/bin/{name}"),
            b"#!/bin/sh\nexit 0\n",
        );
        fs::set_permissions(executable, fs::Permissions::from_mode(0o755)).unwrap();
        fs::create_dir_all(self.home.join(format!(".{name}/skills"))).unwrap();
    }

    #[cfg(target_os = "linux")]
    fn scheduler(&self) {
        use std::os::unix::fs::PermissionsExt;
        let executable = write_file(
            &self.home,
            ".local/bin/systemctl",
            include_bytes!("fixtures/assemble/systemctl"),
        );
        fs::set_permissions(executable, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = if self.restrictive_umask {
            let mut command = isolated_command("/bin/sh", &self.home);
            command
                .args(["-c", "umask 077; exec \"$@\"", "fixture-shell"])
                .arg(std::env::current_exe().unwrap());
            command
        } else {
            isolated_command(std::env::current_exe().unwrap(), &self.home)
        };
        command
            .args(["--exact", "fixture_dispatch", "--nocapture"])
            .env("SNO_TEST_ASSEMBLE_SOURCE", &self.source)
            .env(
                "SNO_TEST_ASSEMBLE_ARGS",
                serde_json::to_string(
                    &std::iter::once("sno")
                        .chain(args.iter().copied())
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            );
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        bounded_output(&mut self.command(args))
    }

    fn activation_checkpoint(&self) -> String {
        format!(
            "after-write:{}",
            self.home.join(".local/lib/sno-reach/current").display()
        )
    }

    fn pause(&self, args: &[&str], phase: &str) -> std::process::Child {
        self.pause_with_timeout(args, phase, Duration::from_secs(10))
    }

    fn pause_with_timeout(
        &self,
        args: &[&str],
        phase: &str,
        timeout: Duration,
    ) -> std::process::Child {
        let marker = self.root.path().join("checkpoint");
        let mut child = self
            .command(args)
            .env("SNO_TEST_CHECKPOINT", phase)
            .env("SNO_TEST_CHECKPOINT_MODE", "block")
            .env("SNO_TEST_CHECKPOINT_MARKER", &marker)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + timeout;
        while !marker.exists() {
            if child.try_wait().unwrap().is_some() {
                let output = child.wait_with_output().unwrap();
                panic!("checkpoint {phase} not reached: {}", diagnostic(&output));
            }
            if Instant::now() > deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("checkpoint {phase} timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        child
    }

    fn manifest(&self) -> PathBuf {
        self.home.join(".config/sno/assemble.json")
    }

    fn owned_snapshot(
        &self,
        mtimes: bool,
    ) -> BTreeMap<PathBuf, (String, u32, Option<std::time::SystemTime>)> {
        use std::os::unix::fs::PermissionsExt;
        let manifest: Value = serde_json::from_slice(&fs::read(self.manifest()).unwrap()).unwrap();
        let mut paths: Vec<PathBuf> = manifest["files"]
            .as_object()
            .unwrap()
            .keys()
            .map(PathBuf::from)
            .collect();
        paths.extend(
            manifest["hooks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|hook| PathBuf::from(hook["path"].as_str().unwrap())),
        );
        paths.push(self.manifest());
        paths
            .into_iter()
            .map(|path| {
                let metadata = fs::symlink_metadata(&path).unwrap();
                let content = if metadata.file_type().is_symlink() {
                    format!("link:{}", fs::read_link(&path).unwrap().display())
                } else {
                    hex::encode(Sha256::digest(fs::read(&path).unwrap()))
                };
                let modified = mtimes.then(|| metadata.modified().unwrap());
                (
                    path,
                    (content, metadata.permissions().mode() & 0o777, modified),
                )
            })
            .collect()
    }

    fn real_shell_dependencies(&self) {
        for name in ["sh", "jq", "mktemp", "sleep", "cat", "rm", "timeout"] {
            let candidates: &[&str] = if name == "timeout" {
                &["timeout", "gtimeout"]
            } else {
                &[name]
            };
            let command = candidates
                .iter()
                .find_map(|candidate| {
                    std::env::split_paths(&std::env::var_os("PATH").unwrap())
                        .map(|root| root.join(candidate))
                        .find(|path| path.is_file())
                })
                .unwrap_or_else(|| {
                    panic!("missing host shell dependency: {name} ({candidates:?})")
                });
            let path = self.home.join(".local/bin").join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(command, path).unwrap();
        }
    }

    fn driver_table(&self) {
        let source: Value = serde_json::from_slice(&fs::read(&self.source).unwrap()).unwrap();
        let mut artifacts = source["programs"].as_array().unwrap().clone();
        artifacts.push(source["skills"].clone());
        let text = artifacts
            .iter()
            .map(|artifact| {
                ["name", "version", "url", "sha256", "entry_point"]
                    .iter()
                    .map(|key| artifact[*key].as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join("\t")
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(self.home.join(".fixture-releases"), text + "\n").unwrap();
    }

    fn compile_driver(&self) -> PathBuf {
        let dependency_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let library = fs::read_dir(&dependency_dir)
            .unwrap()
            .map(Result::unwrap)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("libsno-")
                    && path.extension().is_some_and(|s| s == "rlib")
            })
            .max_by_key(|path| fs::metadata(path).unwrap().modified().unwrap())
            .expect("compiled sno library");
        let driver = self.root.path().join("fixture-sno");
        let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .arg("--edition=2024")
            .arg("--crate-name=fixture_sno")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/assemble/driver.rs"))
            .arg("-L")
            .arg(format!("dependency={}", dependency_dir.display()))
            .arg("--extern")
            .arg(format!("sno={}", library.display()))
            .arg("-o")
            .arg(&driver)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", diagnostic(&output));
        driver
    }

    fn success(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}: {}",
            diagnostic(&output)
        );
        output
    }
}

fn isolated_command(executable: impl AsRef<std::ffi::OsStr>, home: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .env("HOME", home)
        .env("PATH", home.join(".local/bin"))
        .env("SNO_PROFILE_DIR", home.join("station-profile"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("CODEX_HOME")
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("OPENCLAW_STATE_DIR")
        .env_remove("SNO_REACH_ADDR");
    command
}

fn bounded_output(command: &mut Command) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "installer exceeded 20 second test bound: {}",
                diagnostic(&output)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

fn payload_digest(files: &[(String, Vec<u8>, u32)], prefix: &str) -> String {
    let prefix = format!("{prefix}/");
    let mut entries: Vec<_> = files.iter().filter_map(|(name, bytes, mode)| {
        let path = name.strip_prefix(&prefix)?;
        if path == "PROMOTED.json" { return None; }
        Some((path, json!({"mode": mode, "path": path, "sha256": hex::encode(Sha256::digest(bytes)), "type": "file"})))
    }).collect();
    entries.sort_by_key(|(path, _)| *path);
    let entries: Vec<_> = entries.into_iter().map(|(_, entry)| entry).collect();
    hex::encode(Sha256::digest(serde_json::to_vec(&entries).unwrap()))
}

fn archive(files: &[(&str, &[u8], u32)]) -> Vec<u8> {
    let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut tar = tar::Builder::new(gzip);
    for (path, contents, mode) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(*mode);
        header.set_cksum();
        tar.append_data(&mut header, path, *contents).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap()
}

fn write_file(home: &Path, path: &str, bytes: &[u8]) -> PathBuf {
    let path = home.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

fn sno(home: &Path, args: &[&str]) -> Output {
    bounded_output(isolated_command(env!("CARGO_BIN_EXE_sno"), home).args(args))
}

fn diagnostic(output: &Output) -> String {
    format!(
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn tree_snapshot(root: &Path) -> BTreeMap<PathBuf, (String, u32)> {
    use std::os::unix::fs::PermissionsExt;
    fn collect(root: &Path, result: &mut BTreeMap<PathBuf, (String, u32)>) {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let content = if metadata.file_type().is_symlink() {
                format!("link:{}", fs::read_link(&path).unwrap().display())
            } else if metadata.is_dir() {
                "directory".into()
            } else {
                hex::encode(Sha256::digest(fs::read(&path).unwrap()))
            };
            result.insert(
                path.clone(),
                (content, metadata.permissions().mode() & 0o777),
            );
            if metadata.is_dir() {
                collect(&path, result);
            }
        }
    }
    let mut result = BTreeMap::new();
    collect(root, &mut result);
    result
}

fn assert_installer_help(verb: &str) {
    let home = TempDir::new().unwrap();
    let output = sno(home.path(), &[verb, "--help"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{verb}: {}",
        diagnostic(&output)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("--reach-version"), "{text}");
    assert!(text.contains("--skills-version"), "{text}");
}

#[test]
fn assemble_has_real_help() {
    assert_installer_help("assemble");
    let home = TempDir::new().unwrap();
    let retired = sno(home.path(), &["starport"]);
    assert_eq!(retired.status.code(), Some(2));
    assert_eq!(
        retired.stderr,
        b"error: 'starport' is not a top-level command; run 'sno --help'\n"
    );
    assert!(!String::from_utf8_lossy(&sno(home.path(), &["--help"]).stdout).contains("starport"));
}

#[test]
fn update_has_real_help() {
    assert_installer_help("update");
}

#[test]
fn doctor_reports_installation_in_empty_home() {
    let home = TempDir::new().unwrap();
    let output = sno(home.path(), &["doctor", "--json"]);
    assert_ne!(
        output.status.code(),
        Some(0),
        "empty station must not be healthy"
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    assert!(
        report.get("error").is_none(),
        "doctor must dispatch rather than reject: {report}"
    );
    for section in ["reach", "skills", "hooks", "acp", "timer", "station"] {
        assert!(report.get(section).is_some(), "missing {section}: {report}");
    }
    assert_eq!(report["station"]["identity"]["status"], "warn");
    assert_eq!(report["station"]["buffer"]["status"], "warn");
}

#[test]
fn remove_empty_home_is_a_usage_result_without_installation_changes() {
    let home = TempDir::new().unwrap();
    let output = sno(home.path(), &["remove", "--json"]);
    assert_eq!(output.status.code(), Some(2), "{}", diagnostic(&output));
    let report: Value = serde_json::from_slice(&output.stdout).expect("remove JSON");
    assert!(report.to_string().contains("nothing assembled"), "{report}");
    assert!(!home.path().join(".config/sno/assemble.json").exists());
    assert!(!home.path().join(".local/lib/sno-reach").exists());
    assert!(
        !fs::read_dir(home.path())
            .unwrap()
            .any(|entry| entry.unwrap().file_name() == "station-profile")
    );
}

#[test]
fn install_repeat_update_remove_preserves_user_content() {
    let mut fixture = Fixture::new();
    fixture.harness("claude");
    let settings = write_file(
        &fixture.home,
        ".claude/settings.json",
        format!("{{\"hooks\":{{\"UserPromptSubmit\":[{USER_HOOK}]}},\"userSetting\":true}}\n")
            .as_bytes(),
    );
    fixture.success(&[
        "assemble",
        "--reach-version",
        "2.0",
        "--skills-version",
        "1.0",
    ]);
    let installed = fixture.home.join(".local/bin/sno-reach");
    let version = Command::new(&installed).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"2.0\n");
    for (path, expected) in [
        ("VERSION", b"2.0".as_slice()),
        ("lib/resource.txt", b"synthetic resource\n".as_slice()),
        ("vendor/resource.txt", b"synthetic vendor\n".as_slice()),
        ("spec/schema.json", b"{}\n".as_slice()),
        ("guide/README.md", b"Synthetic guide.\n".as_slice()),
        ("NOTICE", b"Synthetic test fixture\n".as_slice()),
        ("LICENSE", b"Synthetic test fixture\n".as_slice()),
    ] {
        assert_eq!(
            fs::read(
                fixture
                    .home
                    .join(".local/lib/sno-reach/releases/2.0")
                    .join(path)
            )
            .unwrap(),
            expected,
            "{path}"
        );
    }
    let agents: Value = serde_json::from_slice(
        &fs::read(fixture.home.join(".config/sno-reach/agents.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        agents
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["claude"]
    );
    assert!(
        agents["claude"]["acpx_agent"]
            .as_str()
            .is_some_and(|name| !name.is_empty())
    );
    for command in ["heartbeat", "subscription-quota-check"] {
        let output = Command::new(fixture.home.join(".local/bin").join(command))
            .arg("--help")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{command}: {}",
            diagnostic(&output)
        );
    }
    let skill = fixture.home.join(".claude/skills/reach");
    assert!(skill.join("SKILL.md").is_file());
    assert_eq!(
        fs::read(skill.join("references/usage.md")).unwrap(),
        b"Synthetic instructions.\n"
    );
    assert!(skill.join("PROMOTED.json").is_file());
    assert!(
        fixture
            .home
            .join(".claude/skills/handoff/SKILL.md")
            .is_file()
    );
    for name in ["heartbeat", "subscription-quota-check"] {
        assert!(
            !fixture
                .home
                .join(format!(".claude/skills/{name}/SKILL.md"))
                .exists(),
            "unverified reader delivery cannot satisfy {name}"
        );
    }
    let hooks = fs::read_to_string(&settings).unwrap();
    assert!(
        hooks.contains(USER_HOOK),
        "unrelated hook bytes changed: {hooks}"
    );
    let hook_config: Value = serde_json::from_str(&hooks).unwrap();
    let prompt_hooks = hook_config["hooks"]["UserPromptSubmit"].as_array().unwrap();
    assert_eq!(prompt_hooks.len(), 2, "{hooks}");
    assert_eq!(
        prompt_hooks
            .iter()
            .filter(|entry| entry.to_string().contains("remind"))
            .count(),
        1,
        "{hooks}"
    );
    let manifest = fs::read(fixture.manifest()).unwrap();
    let entire_installation = fixture.owned_snapshot(true);
    let modified = fs::metadata(fixture.manifest())
        .unwrap()
        .modified()
        .unwrap();
    let repeat = fixture.success(&["assemble"]);
    assert!(String::from_utf8_lossy(&repeat.stdout).contains("current"));
    assert_eq!(fs::read(fixture.manifest()).unwrap(), manifest);
    assert_eq!(
        fs::metadata(fixture.manifest())
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
    assert_eq!(fs::read_to_string(&settings).unwrap(), hooks);
    assert_eq!(
        fixture.owned_snapshot(true),
        entire_installation,
        "repeat changed an installed file, link, mode or mtime"
    );
    fixture.release("2.1");
    let update = fixture.success(&["update"]);
    let update_text = String::from_utf8_lossy(&update.stdout);
    assert!(
        update_text
            .lines()
            .any(|line| line.trim_end() == "update reach updated 2.0 -> 2.1"),
        "{update_text}"
    );
    assert!(
        update_text
            .lines()
            .any(|line| line.starts_with("update skill reach claude current")),
        "{update_text}"
    );
    assert_eq!(
        Command::new(&installed)
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
        b"2.1\n"
    );
    write_file(
        &fixture.home,
        ".claude/skills/reach/user-notes.md",
        b"keep inside\n",
    );
    write_file(
        &fixture.home,
        ".claude/skills/user-other/SKILL.md",
        b"keep beside\n",
    );
    write_file(
        &fixture.home,
        ".local/state/sno-reach/user-mail",
        b"keep state\n",
    );
    let owned: Value = serde_json::from_slice(&fs::read(fixture.manifest()).unwrap()).unwrap();
    fixture.success(&["remove"]);
    for path in owned["files"].as_object().unwrap().keys() {
        assert!(
            fs::symlink_metadata(path).is_err(),
            "owned path survived remove: {path}"
        );
    }
    assert!(!fixture.manifest().exists());
    for command in ["sno-reach", "heartbeat", "subscription-quota-check"] {
        assert!(fs::symlink_metadata(fixture.home.join(".local/bin").join(command)).is_err());
    }
    assert!(!skill.join("SKILL.md").exists());
    assert_eq!(
        fs::read(skill.join("user-notes.md")).unwrap(),
        b"keep inside\n"
    );
    assert_eq!(
        fs::read(fixture.home.join(".claude/skills/user-other/SKILL.md")).unwrap(),
        b"keep beside\n"
    );
    assert_eq!(
        fs::read(fixture.home.join(".local/state/sno-reach/user-mail")).unwrap(),
        b"keep state\n"
    );
    assert!(fs::read_to_string(&settings).unwrap().contains(USER_HOOK));
    assert!(!fs::read_to_string(&settings).unwrap().contains("sno-reach"));
    assert!(!fixture.home.join(".config/sno-reach/agents.json").exists());
    let second = fixture.run(&["remove"]);
    assert_eq!(second.status.code(), Some(2), "{}", diagnostic(&second));
}

#[test]
fn checksum_failure_preserves_the_committed_installation() {
    for already_installed in [false, true] {
        let mut fixture = Fixture::new();
        let previous = if already_installed {
            fixture.success(&["assemble"]);
            let snapshot = fixture.owned_snapshot(false);
            fixture.release("2.1");
            Some(snapshot)
        } else {
            None
        };
        let mut source: Value =
            serde_json::from_slice(&fs::read(&fixture.source).unwrap()).unwrap();
        source["programs"][0]["sha256"] = Value::String("0".repeat(64));
        fs::write(&fixture.source, serde_json::to_vec(&source).unwrap()).unwrap();
        let output = fixture.run(&[if already_installed {
            "update"
        } else {
            "assemble"
        }]);
        assert_eq!(output.status.code(), Some(3), "{}", diagnostic(&output));
        if let Some(snapshot) = previous {
            assert_eq!(fixture.owned_snapshot(false), snapshot);
        } else {
            assert!(!fixture.manifest().exists());
            assert!(!fixture.home.join(".local/lib/sno-reach").exists());
        }
        assert!(
            !fixture
                .home
                .join(".local/lib/sno-reach/releases/2.1")
                .exists()
        );
    }
}

#[test]
fn unowned_command_and_skill_are_never_adopted() {
    for conflict in [".local/bin/sno-reach", ".claude/skills/reach/SKILL.md"] {
        let fixture = Fixture::new();
        fixture.harness("claude");
        let owned_by_user = if conflict == ".local/bin/sno-reach" {
            let target = write_file(&fixture.home, "user-command-target", b"user owned\n");
            let link = fixture.home.join(conflict);
            std::os::unix::fs::symlink(&target, &link).unwrap();
            link
        } else {
            write_file(&fixture.home, conflict, b"user owned\n")
        };
        let output = fixture.run(&["assemble"]);
        assert_eq!(
            output.status.code(),
            Some(4),
            "{conflict}: {}",
            diagnostic(&output)
        );
        assert_eq!(fs::read(&owned_by_user).unwrap(), b"user owned\n");
        if conflict == ".local/bin/sno-reach" {
            assert_eq!(
                fs::read_link(owned_by_user).unwrap(),
                fixture.home.join("user-command-target")
            );
        }
        assert!(!fixture.manifest().exists());
        assert!(!fixture.home.join(".local/lib/sno-reach/current").exists());
    }
}

#[test]
fn user_changes_refuse_update_and_remove() {
    let mut fixture = Fixture::new();
    fixture.harness("claude");
    fixture.success(&["assemble"]);
    let manifest = fs::read(fixture.manifest()).unwrap();
    let changed = write_file(
        &fixture.home,
        ".claude/skills/reach/SKILL.md",
        b"user edits\n",
    );
    fixture.release("2.1");
    for verb in ["update", "remove"] {
        let output = fixture.run(&[verb]);
        assert_eq!(
            output.status.code(),
            Some(4),
            "{verb}: {}",
            diagnostic(&output)
        );
        assert_eq!(fs::read(&changed).unwrap(), b"user edits\n");
        assert_eq!(fs::read(fixture.manifest()).unwrap(), manifest);
    }
}

#[test]
fn existing_agents_config_is_preserved_and_invalid_config_refused() {
    for config in [
        b"{\n  \"codex\": {\"acpx_agent\": \"user-codex\"}\n}\n".as_slice(),
        b"not json\n".as_slice(),
    ] {
        let fixture = Fixture::new();
        let path = write_file(&fixture.home, ".config/sno-reach/agents.json", config);
        let output = fixture.run(&["assemble"]);
        let expected = if config.starts_with(b"{") { 0 } else { 4 };
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{}",
            diagnostic(&output)
        );
        assert_eq!(fs::read(&path).unwrap(), config);
        if expected == 0 {
            fixture.success(&["remove"]);
            assert_eq!(fs::read(path).unwrap(), config);
        } else {
            assert!(!fixture.manifest().exists());
            assert!(diagnostic(&output).contains("agents.json"));
            let doctor = fixture.run(&["doctor"]);
            assert!(
                String::from_utf8_lossy(&doctor.stdout)
                    .lines()
                    .any(|line| line.starts_with("doctor acp agents.json fail ")),
                "{}",
                diagnostic(&doctor)
            );
        }
    }
}

#[test]
fn archive_link_escaping_staging_is_refused_before_install() {
    for linked in [true, false] {
        let fixture = Fixture::new();
        let mut source: Value =
            serde_json::from_slice(&fs::read(&fixture.source).unwrap()).unwrap();
        let valid_path = url::Url::parse(source["skills"]["url"].as_str().unwrap())
            .unwrap()
            .to_file_path()
            .unwrap();
        let valid_bytes = fs::read(valid_path).unwrap();
        let mut valid = tar::Archive::new(flate2::read::GzDecoder::new(valid_bytes.as_slice()));
        let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut tar = tar::Builder::new(gzip);
        for entry in valid.entries().unwrap() {
            let mut entry = entry.unwrap();
            let header = entry.header().clone();
            tar.append(&header, &mut entry).unwrap();
        }
        let mut header = tar::Header::new_gnu();
        if linked {
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o777);
            tar.append_link(&mut header, "escaping", "../../outside")
                .unwrap();
        } else {
            let name = b"../../outside";
            header.as_mut_bytes()[..name.len()].copy_from_slice(name);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(3);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, b"bad".as_slice()).unwrap();
        }
        let bytes = tar.into_inner().unwrap().finish().unwrap();
        source["skills"] = fixture.artifact("skills", "1.0", "", &bytes);
        fs::write(&fixture.source, serde_json::to_vec(&source).unwrap()).unwrap();
        let output = fixture.run(&["assemble"]);
        assert_eq!(output.status.code(), Some(3), "{}", diagnostic(&output));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("../../outside"),
            "archive must be rejected for the escape, not missing payload: {error}"
        );
        assert!(!fixture.manifest().exists());
        assert!(!fixture.home.join(".local/lib/sno-reach/current").exists());
    }
}

#[test]
fn invalid_skill_declarations_refuse_all_installation_changes() {
    for (case, declaration) in [
        ("missing requires", ""),
        (
            "wrong list type",
            "requires: {programs: wrong, harness: []}",
        ),
        (
            "unknown program",
            "requires: {programs: [{name: typo, min_version: '1.0'}], harness: []}",
        ),
        (
            "unknown slot",
            "requires: {programs: [], harness: [{slot: invented, need: required}]}",
        ),
        (
            "invalid need",
            "requires: {programs: [], harness: [{slot: 4.shell, need: nice}]}",
        ),
        (
            "blank version",
            "requires: {programs: [{name: reach, min_version: ' '}], harness: []}",
        ),
        (
            "expression version",
            "requires: {programs: [{name: reach, min_version: '>=2.0'}], harness: []}",
        ),
        (
            "duplicate key",
            "requires: {programs: [], programs: [], harness: []}",
        ),
        (
            "duplicate entry",
            "requires: {programs: [{name: reach, min_version: '2.0'}, {name: reach, min_version: '2.0'}], harness: []}",
        ),
        (
            "unknown field",
            "requires: {programs: [], harness: [], fallback: true}",
        ),
    ] {
        let fixture = Fixture::new();
        fixture.harness("claude");
        fixture.reach_declaration(declaration);
        let output = fixture.run(&["assemble"]);
        assert_eq!(
            output.status.code(),
            Some(3),
            "{case}: {}",
            diagnostic(&output)
        );
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            !error.contains("fixture"),
            "{case} must fail declaration parsing, not shared fixture comparison: {error}"
        );
        assert!(
            error.contains("requires") || error.contains("version") || error.contains("SKILL.md"),
            "{case}: {error}"
        );
        assert!(!fixture.manifest().exists(), "{case}");
        assert!(
            !fixture.home.join(".local/lib/sno-reach/current").exists(),
            "{case}"
        );
    }
}

#[test]
fn known_unverified_capability_obeys_required_preferred_optional() {
    for (need, expected) in [
        ("required", "skipped"),
        ("preferred", "degraded"),
        ("optional", "installed"),
    ] {
        let fixture = Fixture::new();
        fixture.harness("claude");
        fixture.reach_declaration(&format!("requires: {{programs: [{{name: reach, min_version: '2.0'}}], harness: [{{slot: 4.shell, need: required}}, {{slot: 3.reader-to-agent-delivery, need: {need}}}]}}"));
        let output = fixture.success(&["assemble"]);
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(
            text.lines().any(|line| line.contains("reach")
                && line.contains("claude")
                && line.contains(expected)),
            "{need}: {text}"
        );
        assert_eq!(
            fixture.home.join(".claude/skills/reach/SKILL.md").exists(),
            need != "required"
        );
    }
}

#[test]
fn program_versions_compare_numerically_per_program() {
    let mut fixture = Fixture::new();
    fixture.release("2.10");
    fixture.harness("claude");
    fixture.reach_declaration("requires: {programs: [{name: reach, min_version: '2.9'}], harness: [{slot: 4.shell, need: required}]}");
    fixture.success(&["assemble"]);
    assert!(fixture.home.join(".claude/skills/reach/SKILL.md").is_file());
    let independent = Fixture::new();
    independent.harness("claude");
    independent.reach_declaration("requires: {programs: [{name: heartbeat, min_version: '2.0'}], harness: [{slot: 4.shell, need: required}]}");
    let output = independent.success(&["assemble"]);
    assert!(
        !independent
            .home
            .join(".claude/skills/reach/SKILL.md")
            .exists()
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.lines().any(|line| line.contains("reach")
            && line.contains("heartbeat")
            && line.contains("skipped")
            && line.contains("2.0")
            && line.contains("1.0")),
        "{text}"
    );
}

#[test]
fn handled_activation_failure_restores_first_install_and_update() {
    for updating in [false, true] {
        let mut fixture = Fixture::new();
        fixture.harness("claude");
        let previous = if updating {
            fixture.success(&["assemble"]);
            fixture.real_shell_dependencies();
            let snapshot = tree_snapshot(&fixture.home);
            let hook = fs::read(fixture.home.join(".claude/settings.json")).unwrap();
            fixture.release("2.1");
            Some((snapshot, hook))
        } else {
            None
        };
        let output = bounded_output(
            fixture
                .command(&[if updating { "update" } else { "assemble" }])
                .env("SNO_TEST_CHECKPOINT", fixture.activation_checkpoint())
                .env("SNO_TEST_CHECKPOINT_MODE", "fail"),
        );
        assert_eq!(output.status.code(), Some(4), "{}", diagnostic(&output));
        assert!(String::from_utf8_lossy(&output.stderr).contains("synthetic write failure"));
        assert!(
            !fixture
                .home
                .join(".config/sno/assemble.pending.json")
                .exists()
        );
        match previous {
            Some((snapshot, hook)) => {
                assert_eq!(
                    tree_snapshot(&fixture.home),
                    snapshot,
                    "rollback changed the prior complete tree"
                );
                let output = Command::new(fixture.home.join(".local/bin/sno-reach"))
                    .arg("--version")
                    .output()
                    .unwrap();
                assert_eq!(output.stdout, b"2.0\n");
                fixture.success(&["update"]);
                assert_ne!(
                    fs::read(fixture.home.join(".claude/settings.json")).unwrap(),
                    hook,
                    "update must include a changed owned hook after jq becomes available"
                );
            }
            None => {
                assert!(!fixture.manifest().exists());
                assert!(!fixture.home.join(".local/lib/sno-reach/current").exists());
                assert!(!fixture.home.join(".claude/settings.json").exists());
                for program in ["reach", "heartbeat", "subscription-quota-check"] {
                    assert!(
                        !fixture
                            .home
                            .join(format!(".local/lib/sno-{program}"))
                            .exists()
                    );
                }
                assert!(!fixture.home.join(".config/sno-reach/agents.json").exists());
            }
        }
    }
}

#[test]
fn killed_install_recovers_before_remove_and_refuses_concurrent_mutation() {
    let fixture = Fixture::new();
    fixture.harness("claude");
    let mut child = fixture.pause(&["assemble"], &fixture.activation_checkpoint());
    let pending_path = fixture.home.join(".config/sno/assemble.pending.json");
    let pending = fs::read(&pending_path).unwrap();
    let before = tree_snapshot(&fixture.home);
    let competing = fixture.run(&["remove"]);
    let competition_code = competing.status.code();
    let pending_after = fs::read(&pending_path).unwrap();
    let after = tree_snapshot(&fixture.home);
    child.kill().unwrap();
    child.wait().unwrap();
    assert_eq!(competition_code, Some(4), "{}", diagnostic(&competing));
    assert_eq!(pending_after, pending);
    assert_eq!(
        after, before,
        "competing mutation changed the complete tree"
    );
    let doctor = fixture.run(&["doctor"]);
    assert_ne!(doctor.status.code(), Some(0), "{}", diagnostic(&doctor));
    assert!(diagnostic(&doctor).contains("pending") || diagnostic(&doctor).contains("recover"));
    let remove = fixture.run(&["remove"]);
    assert_eq!(remove.status.code(), Some(2), "{}", diagnostic(&remove));
    assert!(!pending_path.exists());
    assert!(!fixture.manifest().exists());
    assert!(fs::symlink_metadata(fixture.home.join(".local/bin/sno-reach")).is_err());
    assert!(!fixture.home.join(".local/lib/sno-reach").exists());
}

#[test]
fn killed_after_manifest_commit_keeps_the_new_generation() {
    let mut fixture = Fixture::new();
    fixture.success(&["assemble"]);
    fixture.release("2.1");
    let mut child = fixture.pause(&["update"], "after-manifest");
    let committed = fs::read(fixture.manifest()).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
    fixture.success(&["assemble"]);
    assert_eq!(fs::read(fixture.manifest()).unwrap(), committed);
    assert!(
        !fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
    assert_eq!(
        Command::new(fixture.home.join(".local/bin/sno-reach"))
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
        b"2.1\n"
    );
}

#[test]
fn recovery_conflict_keeps_pending_record_and_user_bytes() {
    let fixture = Fixture::new();
    let mut child = fixture.pause(&["assemble"], &fixture.activation_checkpoint());
    child.kill().unwrap();
    child.wait().unwrap();
    let current = fixture.home.join(".local/lib/sno-reach/current");
    fs::remove_file(&current).unwrap();
    fs::write(&current, b"user replacement\n").unwrap();
    let pending = fixture.home.join(".config/sno/assemble.pending.json");
    let bytes = fs::read(&pending).unwrap();
    let output = fixture.run(&["remove"]);
    assert_eq!(output.status.code(), Some(4), "{}", diagnostic(&output));
    assert_eq!(fs::read(&pending).unwrap(), bytes);
    assert_eq!(fs::read(current).unwrap(), b"user replacement\n");
}

#[test]
fn disabling_auto_update_does_not_require_release_access() {
    let fixture = Fixture::new();
    fixture.success(&["assemble"]);
    for path in [
        ".claude/settings.json",
        ".codex/hooks.json",
        ".hermes/skills",
        ".openclaw/skills",
    ] {
        assert!(
            !fixture.home.join(path).exists(),
            "absent harness received {path}"
        );
    }
    fs::remove_file(&fixture.source).unwrap();
    let output = fixture.success(&["update", "--auto", "off"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("disabled"));
    assert!(fixture.manifest().exists());
}

#[test]
#[cfg(target_os = "linux")]
fn scheduler_enable_disable_is_owned_and_disable_works_offline() {
    let fixture = Fixture::new();
    fixture.scheduler();
    fixture.success(&["assemble"]);
    fixture.success(&["update", "--auto", "on"]);
    assert!(fixture.home.join("scheduler-active").exists());
    assert!(fixture.home.join("scheduler-enabled").exists());
    let service = fixture.home.join(".config/systemd/user/sno-update.service");
    let timer = fixture.home.join(".config/systemd/user/sno-update.timer");
    let text = fs::read_to_string(&service).unwrap();
    assert!(text.contains("update --quiet"), "{text}");
    assert!(
        text.contains(&format!("HOME={}", fixture.home.display())),
        "{text}"
    );
    assert!(
        fs::read_to_string(&timer)
            .unwrap()
            .contains("OnCalendar=daily")
    );
    let committed = fs::read(fixture.manifest()).unwrap();
    fixture.success(&["update", "--auto", "on"]);
    let repeated = fs::read(fixture.manifest()).unwrap();
    let old: Value = serde_json::from_slice(&committed).unwrap();
    let new: Value = serde_json::from_slice(&repeated).unwrap();
    let changed: Vec<_> = new
        .as_object()
        .unwrap()
        .iter()
        .filter(|(key, value)| old.get(*key) != Some(*value))
        .map(|(key, _)| key)
        .collect();
    assert!(
        repeated == committed,
        "repeat auto-on changed manifest fields: {changed:?}"
    );
    fs::remove_file(&fixture.source).unwrap();
    fixture.success(&["update", "--auto", "off"]);
    assert!(!fixture.home.join("scheduler-active").exists());
    assert!(!fixture.home.join("scheduler-enabled").exists());
    assert!(!service.exists());
    assert!(!timer.exists());
    assert!(fixture.manifest().exists());
}

#[test]
#[cfg(target_os = "linux")]
fn scheduler_enable_failure_restores_files_and_manifest() {
    let fixture = Fixture::new();
    fixture.scheduler();
    fixture.success(&["assemble"]);
    let manifest = fs::read(fixture.manifest()).unwrap();
    write_file(&fixture.home, "scheduler-fail-enable", b"fail");
    let output = fixture.run(&["update", "--auto", "on"]);
    assert_eq!(output.status.code(), Some(4), "{}", diagnostic(&output));
    assert!(!fixture.home.join("scheduler-active").exists());
    assert!(!fixture.home.join("scheduler-enabled").exists());
    assert!(
        fs::read(fixture.manifest()).unwrap() == manifest,
        "scheduler failure changed the committed manifest"
    );
    assert!(
        !fixture
            .home
            .join(".config/systemd/user/sno-update.service")
            .exists()
    );
    assert!(
        !fixture
            .home
            .join(".config/systemd/user/sno-update.timer")
            .exists()
    );
    assert!(
        !fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
}

#[test]
#[cfg(target_os = "linux")]
fn remove_disables_active_timer_and_removes_owned_units() {
    let fixture = Fixture::new();
    fixture.scheduler();
    fixture.success(&["assemble"]);
    fixture.success(&["update", "--auto", "on"]);
    assert!(fixture.home.join("scheduler-active").exists());
    assert!(fixture.home.join("scheduler-enabled").exists());
    fixture.success(&["remove"]);
    assert!(!fixture.home.join("scheduler-active").exists());
    assert!(!fixture.home.join("scheduler-enabled").exists());
    assert!(!fixture.manifest().exists());
    assert!(
        !fixture
            .home
            .join(".config/systemd/user/sno-update.service")
            .exists()
    );
    assert!(
        !fixture
            .home
            .join(".config/systemd/user/sno-update.timer")
            .exists()
    );
}

#[test]
fn externally_owned_exact_reminder_survives_remove() {
    let fixture = Fixture::new();
    fixture.harness("claude");
    fixture.success(&["assemble"]);
    let path = fixture.home.join(".claude/settings.json");
    let canonical = fs::read(&path).unwrap();
    fixture.success(&["remove"]);
    fs::write(&path, &canonical).unwrap();
    fixture.success(&["assemble"]);
    assert_eq!(fs::read(&path).unwrap(), canonical);
    fixture.success(&["remove"]);
    assert_eq!(fs::read(&path).unwrap(), canonical);
}

#[test]
fn conflicting_external_reminder_is_preserved_on_refusal() {
    let fixture = Fixture::new();
    fixture.harness("claude");
    let original = b"{\"hooks\":{\"UserPromptSubmit\":[{\"hooks\":[{\"type\":\"command\",\"command\":\"sno reach remind --as user-seat\"}]}]}}\n";
    let path = write_file(&fixture.home, ".claude/settings.json", original);
    let output = fixture.run(&["assemble"]);
    assert_eq!(output.status.code(), Some(4), "{}", diagnostic(&output));
    assert_eq!(fs::read(path).unwrap(), original);
    assert!(!fixture.manifest().exists());
}

#[test]
fn codex_untrusted_hook_requires_action_and_no_address_returns_no_context() {
    let fixture = Fixture::new();
    fixture.harness("codex");
    let hook_path = write_file(
        &fixture.home,
        ".codex/hooks.json",
        format!("{{\"hooks\":{{\"UserPromptSubmit\":[{USER_HOOK}]}}}}\n").as_bytes(),
    );
    let output = fixture.success(&["assemble"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("press t"));
    assert!(
        !fixture.home.join(".codex/config.toml").exists(),
        "installer must not create trust records"
    );
    let hook_bytes = fs::read(&hook_path).unwrap();
    assert!(String::from_utf8_lossy(&hook_bytes).contains(USER_HOOK));
    fixture.success(&["assemble"]);
    assert_eq!(fs::read(&hook_path).unwrap(), hook_bytes);
    let hooks: Value = serde_json::from_slice(&hook_bytes).unwrap();
    assert_eq!(
        hooks["hooks"]["UserPromptSubmit"].as_array().unwrap().len(),
        2
    );
    let command = hooks["hooks"]["UserPromptSubmit"][1]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    let hook = bounded_output(isolated_command("/bin/sh", &fixture.home).args(["-c", command]));
    assert!(hook.status.success(), "{}", diagnostic(&hook));
    assert!(hook.stdout.is_empty(), "{}", diagnostic(&hook));
    assert!(hook.stderr.is_empty(), "{}", diagnostic(&hook));
    let doctor = fixture.run(&["doctor"]);
    assert_ne!(doctor.status.code(), Some(0));
    let text = String::from_utf8_lossy(&doctor.stdout);
    assert!(
        text.lines().any(|line| line.contains("codex")
            && line.contains("trust")
            && line.contains("missing")),
        "{text}"
    );
    assert!(
        !text
            .lines()
            .any(|line| line.contains("execution") && line.contains(" ok ")),
        "{text}"
    );
}

#[test]
fn codex_trust_is_read_only_and_bound_to_the_current_command() {
    let fixture = Fixture::new();
    fixture.harness("codex");
    fixture.harness("acpx");
    fixture.real_shell_dependencies();
    fixture.success(&["assemble"]);
    let hook_path = fixture.home.join(".codex/hooks.json");
    let mut hooks: Value = serde_json::from_slice(&fs::read(&hook_path).unwrap()).unwrap();
    let command = hooks["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap()
        .to_owned();
    let definition = BTreeMap::from([
        ("async", json!(false)),
        ("command", json!(command)),
        ("timeout", json!(600)),
        ("type", json!("command")),
    ]);
    let group = BTreeMap::from([
        ("event_name", json!("user_prompt_submit")),
        ("hooks", json!([definition])),
    ]);
    let hash = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(serde_json::to_vec(&group).unwrap()))
    );
    let key = format!("{}:user_prompt_submit:0:0", hook_path.display());
    let config = format!(
        "[hooks.state.{}]\ntrusted_hash = '{}'\n",
        serde_json::to_string(&key).unwrap(),
        hash
    );
    let config_path = write_file(&fixture.home, ".codex/config.toml", config.as_bytes());
    let initialized = sno(
        &fixture.home,
        &["station", "telemetry", "consent", "set", "full"],
    );
    assert_eq!(
        initialized.status.code(),
        Some(0),
        "{}",
        diagnostic(&initialized)
    );
    assert!(fixture.home.join("station-profile/buffer.db").is_file());
    let station = sno(&fixture.home, &["station", "doctor", "--json"]);
    assert_eq!(station.status.code(), Some(0), "{}", diagnostic(&station));
    let repeated = fixture.success(&["assemble"]);
    assert!(
        !String::from_utf8_lossy(&repeated.stdout).contains("press t"),
        "{}",
        diagnostic(&repeated)
    );
    assert_eq!(fs::read(&config_path).unwrap(), config.as_bytes());
    let doctor = fixture.run(&["doctor"]);
    assert_eq!(doctor.status.code(), Some(0), "{}", diagnostic(&doctor));
    let text = String::from_utf8_lossy(&doctor.stdout);
    assert!(
        text.lines()
            .any(|line| line.contains("codex trust") && line.contains(" ok ")),
        "{text}"
    );
    assert!(
        !text
            .lines()
            .any(|line| line.contains("execution") && line.contains(" ok ")),
        "{text}"
    );
    let structured = fixture.run(&["doctor", "--json"]);
    assert_eq!(
        structured.status.code(),
        Some(0),
        "{}",
        diagnostic(&structured)
    );
    let report: Value = String::from_utf8_lossy(&structured.stdout)
        .lines()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("doctor JSON object");
    assert_eq!(
        report["station"],
        serde_json::from_slice::<Value>(&station.stdout).unwrap()
    );
    for section in ["reach", "skills", "hooks", "acp", "timer", "station"] {
        assert!(report.get(section).is_some());
    }
    for section in ["reach", "skills", "hooks", "acp", "timer"] {
        for row in report[section].as_array().unwrap() {
            let expected = format!(
                "doctor {section} {} {} {}",
                row["target"].as_str().unwrap(),
                row["result"].as_str().unwrap(),
                row["detail"].as_str().unwrap()
            );
            assert!(
                text.lines()
                    .any(|line| line.trim_end() == expected.trim_end()),
                "JSON/human mismatch: {expected}\n{text}"
            );
        }
    }
    assert!(
        report["hooks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["target"] == "codex execution" && row["result"] == "warn")
    );
    hooks["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"] = json!(format!("{command}; :"));
    fs::write(&hook_path, serde_json::to_vec(&hooks).unwrap()).unwrap();
    let stale = fixture.run(&["doctor"]);
    assert_ne!(stale.status.code(), Some(0));
    let text = String::from_utf8_lossy(&stale.stdout);
    assert!(
        text.lines()
            .any(|line| line.contains("codex trust") && line.contains("missing")),
        "{text}"
    );
    assert_eq!(fs::read(config_path).unwrap(), config.as_bytes());
}

#[test]
#[cfg(target_os = "linux")]
fn interrupted_scheduler_change_restores_external_state_and_manifest() {
    for enabling in [true, false] {
        let fixture = Fixture::new();
        fixture.scheduler();
        fixture.success(&["assemble"]);
        if !enabling {
            fixture.success(&["update", "--auto", "on"]);
        }
        let previous = fs::read(fixture.manifest()).unwrap();
        let mut child = fixture.pause(
            &["update", "--auto", if enabling { "on" } else { "off" }],
            "after-scheduler",
        );
        let changed_state = fixture.home.join("scheduler-active").exists();
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(
            changed_state, enabling,
            "scheduler checkpoint must observe the changed external state"
        );
        assert!(
            fixture
                .home
                .join(".config/sno/assemble.pending.json")
                .exists()
        );
        fixture.success(&["assemble"]);
        assert_eq!(fixture.home.join("scheduler-active").exists(), !enabling);
        assert_eq!(fixture.home.join("scheduler-enabled").exists(), !enabling);
        assert!(
            fs::read(fixture.manifest()).unwrap() == previous,
            "recovery changed the previous manifest"
        );
        assert!(
            !fixture
                .home
                .join(".config/sno/assemble.pending.json")
                .exists()
        );
        assert_eq!(
            fixture
                .home
                .join(".config/systemd/user/sno-update.timer")
                .exists(),
            !enabling
        );
    }
}

#[test]
#[cfg(target_os = "linux")]
fn interrupted_remove_recovers_then_finishes_with_timer_disabled() {
    let fixture = Fixture::new();
    fixture.scheduler();
    fixture.success(&["assemble"]);
    fixture.success(&["update", "--auto", "on"]);
    let mut child = fixture.pause(&["remove"], &fixture.activation_checkpoint());
    let manifest_still_present = fixture.manifest().exists();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        manifest_still_present,
        "manifest disappeared before removal completed"
    );
    assert!(
        fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
    fixture.success(&["remove"]);
    assert!(!fixture.home.join("scheduler-active").exists());
    assert!(!fixture.home.join("scheduler-enabled").exists());
    assert!(!fixture.manifest().exists());
    assert!(
        !fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
    assert!(fs::symlink_metadata(fixture.home.join(".local/bin/sno-reach")).is_err());
    assert!(
        !fixture
            .home
            .join(".config/systemd/user/sno-update.timer")
            .exists()
    );
}

#[test]
fn shared_fixture_must_match_contract_skill_bytes_and_declarations() {
    for case in [
        "contract digest",
        "skill digest",
        "declaration",
        "missing fixture",
    ] {
        let fixture = Fixture::new();
        fixture.edit_skills(|files| {
            if case == "missing fixture" {
                files.retain(|(path, _, _)| path != REQUIREMENTS_FIXTURE);
                return;
            }
            let (_, bytes, _) = files
                .iter_mut()
                .find(|(path, _, _)| path == REQUIREMENTS_FIXTURE)
                .unwrap();
            let mut value: Value = serde_json::from_slice(bytes).unwrap();
            match case {
                "contract digest" => value["contract_sha256"] = json!("0".repeat(64)),
                "skill digest" => value["units"][0]["skill_sha256"] = json!("0".repeat(64)),
                "declaration" => {
                    value["units"][0]["requires"]["programs"][0]["min_version"] = json!("2.1")
                }
                _ => unreachable!(),
            }
            *bytes = serde_json::to_vec(&value).unwrap();
        });
        let output = fixture.run(&["assemble"]);
        assert_eq!(
            output.status.code(),
            Some(3),
            "{case}: {}",
            diagnostic(&output)
        );
        assert!(!fixture.manifest().exists());
        assert!(!fixture.home.join(".local/lib/sno-reach/current").exists());
    }
}

#[test]
fn manifest_records_the_exact_shared_fixture_digest() {
    let fixture = Fixture::new();
    let mut expected = String::new();
    fixture.edit_skills(|files| {
        let (_, bytes, _) = files
            .iter()
            .find(|(path, _, _)| path == REQUIREMENTS_FIXTURE)
            .unwrap();
        expected = hex::encode(Sha256::digest(bytes));
    });
    fixture.success(&["assemble"]);
    let manifest: Value = serde_json::from_slice(&fs::read(fixture.manifest()).unwrap()).unwrap();
    assert_eq!(manifest["requirements_fixture_sha256"], expected);
    assert_eq!(
        manifest["contract_sha256"],
        hex::encode(Sha256::digest(CONTRACT))
    );
}

#[test]
fn interrupted_explicit_purge_resumes_after_manifest_removal() {
    let fixture = Fixture::new();
    fixture.success(&["assemble"]);
    let state = write_file(
        &fixture.home,
        ".local/state/sno-reach/user-mail",
        b"explicitly purged\n",
    );
    let other = write_file(
        &fixture.home,
        ".local/state/other-application/mail",
        b"keep adjacent state\n",
    );
    let mut child = fixture.pause(&["remove", "--purge-state"], "before-purge");
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(!fixture.manifest().exists());
    assert!(state.exists());
    let retry = fixture.run(&["remove"]);
    assert_eq!(retry.status.code(), Some(0), "{}", diagnostic(&retry));
    assert!(!state.exists());
    assert_eq!(fs::read(other).unwrap(), b"keep adjacent state\n");
    assert_eq!(fixture.run(&["remove"]).status.code(), Some(2));
    assert!(
        !fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
}

#[test]
fn doctor_detects_missing_program_without_changing_station_checks() {
    let fixture = Fixture::new();
    fixture.success(&["assemble"]);
    let station_before = sno(&fixture.home, &["station", "doctor", "--json"]);
    let doctor_before = fixture.run(&["doctor"]);
    assert_eq!(
        doctor_before.status.code(),
        Some(1),
        "uninitialised station warnings must remain nonzero"
    );
    let before = String::from_utf8_lossy(&doctor_before.stdout);
    assert!(
        before
            .lines()
            .any(|line| line.starts_with("doctor reach reach ok ")),
        "Reach must be healthy before deleting current: {before}"
    );
    fs::remove_file(fixture.home.join(".local/lib/sno-reach/current")).unwrap();
    let output = fixture.run(&["doctor"]);
    assert_ne!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.lines()
            .any(|line| line.starts_with("doctor reach reach fail ")),
        "{text}"
    );
    assert!(
        text.lines()
            .any(|line| line.contains("timer") && line.contains("disabled")),
        "{text}"
    );
    let station_after = sno(&fixture.home, &["station", "doctor", "--json"]);
    assert_eq!(station_after.status.code(), station_before.status.code());
    assert_eq!(station_after.stdout, station_before.stdout);
    let structured = fixture.run(&["doctor", "--json"]);
    assert_eq!(structured.status.code(), output.status.code());
    let report: Value = String::from_utf8_lossy(&structured.stdout)
        .lines()
        .find_map(|line| serde_json::from_str(line).ok())
        .unwrap();
    assert!(
        report["reach"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["target"] == "reach" && row["result"] == "fail")
    );
}

#[test]
fn restrictive_umask_preserves_modes_across_update_and_rollback() {
    use std::os::unix::fs::PermissionsExt;
    let mut fixture = Fixture::new();
    fixture.restrictive_umask = true;
    fixture.harness("claude");
    fixture.success(&["assemble"]);
    let executable = fixture.home.join(".local/bin/sno-reach");
    assert_eq!(
        fs::metadata(&executable).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert_eq!(
        fs::metadata(
            fixture
                .home
                .join(".local/lib/sno-reach/releases/2.0/lib/resource.txt")
        )
        .unwrap()
        .permissions()
        .mode()
            & 0o777,
        0o644
    );
    let manifest = fs::read(fixture.manifest()).unwrap();
    fixture.success(&["assemble"]);
    assert!(fs::read(fixture.manifest()).unwrap() == manifest);
    fixture.release("2.1");
    let failed = bounded_output(
        fixture
            .command(&["update"])
            .env("SNO_TEST_CHECKPOINT", fixture.activation_checkpoint())
            .env("SNO_TEST_CHECKPOINT_MODE", "fail"),
    );
    assert_eq!(failed.status.code(), Some(4), "{}", diagnostic(&failed));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("synthetic write failure"));
    assert!(fs::read(fixture.manifest()).unwrap() == manifest);
    assert_eq!(
        fs::metadata(&executable).unwrap().permissions().mode() & 0o777,
        0o755
    );
    fixture.success(&["update"]);
    assert_eq!(
        Command::new(&executable)
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
        b"2.1\n"
    );
    assert_eq!(
        fs::metadata(&executable).unwrap().permissions().mode() & 0o777,
        0o755
    );
    fixture.success(&["remove"]);
    assert!(!fixture.manifest().exists());
}

#[test]
fn committed_remove_cleans_owned_directories_before_reassembly() {
    let fixture = Fixture::new();
    fixture.harness("claude");
    fixture.success(&["assemble"]);
    let manifest: Value = serde_json::from_slice(&fs::read(fixture.manifest()).unwrap()).unwrap();
    let directories: Vec<PathBuf> = manifest["directories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().into())
        .collect();
    assert!(!directories.is_empty());
    let mut child = fixture.pause(&["remove"], "after-manifest");
    let payload_removed = manifest["files"]
        .as_object()
        .unwrap()
        .keys()
        .all(|path| fs::symlink_metadata(path).is_err());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(payload_removed, "manifest was removed before owned payload");
    assert!(!fixture.manifest().exists());
    assert_eq!(fixture.run(&["remove"]).status.code(), Some(2));
    for directory in directories {
        assert!(
            !directory.exists(),
            "orphan owned directory: {}",
            directory.display()
        );
    }
    assert!(
        !fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
    fixture.success(&["assemble"]);
}

#[test]
fn large_program_update_and_recovery_keep_payload_out_of_journal_metadata() {
    use rand::{RngCore, SeedableRng};
    let mut fixture = Fixture::new();
    let mut body = vec![0u8; 20 * 1024 * 1024];
    rand::rngs::StdRng::seed_from_u64(937).fill_bytes(&mut body);
    fixture.large_reach_resource(&body);
    fixture.success(&["assemble"]);
    let original = fixture
        .home
        .join(".local/lib/sno-reach/releases/2.0/lib/large.dat");
    assert_eq!(
        Sha256::digest(fs::read(&original).unwrap()),
        Sha256::digest(&body)
    );
    assert!(fs::metadata(fixture.manifest()).unwrap().len() < 1024 * 1024);
    fixture.release("2.1");
    rand::rngs::StdRng::seed_from_u64(938).fill_bytes(&mut body);
    fixture.large_reach_resource(&body);
    let mut child = fixture.pause_with_timeout(
        &["update"],
        &fixture.activation_checkpoint(),
        Duration::from_secs(120),
    );
    let pending = fs::read(fixture.home.join(".config/sno/assemble.pending.json")).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        pending.len() < 1024 * 1024,
        "journal duplicates artifact bodies: {} bytes",
        pending.len()
    );
    let blob = fixture
        .home
        .join(".config/sno/assemble-blobs")
        .join(hex::encode(Sha256::digest(&body)));
    assert_eq!(
        Sha256::digest(fs::read(blob).unwrap()),
        Sha256::digest(&body)
    );
    fixture.success(&["update"]);
    let updated = fixture
        .home
        .join(".local/lib/sno-reach/releases/2.1/lib/large.dat");
    assert_eq!(
        Sha256::digest(fs::read(updated).unwrap()),
        Sha256::digest(&body)
    );
    assert!(!original.exists(), "obsolete program payload was retained");
    assert!(
        !fixture
            .home
            .join(".config/sno/assemble.pending.json")
            .exists()
    );
}

#[test]
#[cfg(target_os = "linux")]
fn generated_scheduler_command_runs_the_actual_cli_and_updates_the_selected_home() {
    let mut fixture = Fixture::new();
    fixture.scheduler();
    let driver = fixture.compile_driver();
    fixture.driver_table();
    let output = bounded_output(isolated_command(&driver, &fixture.home).args(["assemble"]));
    assert_eq!(output.status.code(), Some(0), "{}", diagnostic(&output));
    let output =
        bounded_output(isolated_command(&driver, &fixture.home).args(["update", "--auto", "on"]));
    assert_eq!(output.status.code(), Some(0), "{}", diagnostic(&output));
    let service =
        fs::read_to_string(fixture.home.join(".config/systemd/user/sno-update.service")).unwrap();
    let exec = service
        .lines()
        .find_map(|line| line.strip_prefix("ExecStart="))
        .unwrap();
    assert_eq!(exec, format!("\"{}\" update --quiet", driver.display()));
    let mut environment = BTreeMap::new();
    for line in service
        .lines()
        .filter_map(|line| line.strip_prefix("Environment="))
    {
        let value: String =
            serde_json::from_str(line).expect("generated quoted systemd environment");
        let (name, value) = value.split_once('=').unwrap();
        environment.insert(name.to_owned(), value.to_owned());
    }
    assert_eq!(
        environment.get("HOME").unwrap(),
        fixture.home.to_str().unwrap()
    );
    fixture.release("2.1");
    fixture.driver_table();
    let output = bounded_output(
        Command::new("/bin/sh")
            .args(["-c", exec])
            .env_clear()
            .envs(&environment),
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "scheduled invocation: {}",
        diagnostic(&output)
    );
    assert_eq!(
        Command::new(fixture.home.join(".local/bin/sno-reach"))
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
        b"2.1\n"
    );
    let manifest: Value = serde_json::from_slice(&fs::read(fixture.manifest()).unwrap()).unwrap();
    assert_eq!(manifest["programs"]["reach"]["version"], "2.1");
}

#[test]
fn addressed_reminder_failure_is_visible_and_timeout_stops_the_program() {
    let fixture = Fixture::new();
    fixture.harness("claude");
    fixture.real_shell_dependencies();
    let driver = fixture.compile_driver();
    fixture.driver_table();
    let assembled = bounded_output(isolated_command(&driver, &fixture.home).args(["assemble"]));
    assert_eq!(
        assembled.status.code(),
        Some(0),
        "{}",
        diagnostic(&assembled)
    );
    let hooks: Value =
        serde_json::from_slice(&fs::read(fixture.home.join(".claude/settings.json")).unwrap())
            .unwrap();
    let command = hooks["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    let no_address =
        bounded_output(isolated_command("/bin/sh", &fixture.home).args(["-c", command]));
    assert!(no_address.status.success());
    assert!(no_address.stdout.is_empty());
    assert!(
        !fixture.home.join("remind-pid").exists(),
        "Reach ran without a seat address"
    );
    let failed = bounded_output(
        isolated_command("/bin/sh", &fixture.home)
            .args(["-c", command])
            .env("SNO_REACH_ADDR", "fixture@local")
            .env("SNO_TEST_REMIND_MODE", "fail"),
    );
    assert!(
        failed.status.success(),
        "hook failure must not block prompt"
    );
    assert!(failed.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("failed"),
        "{}",
        diagnostic(&failed)
    );
    let stdout = fixture.root.path().join("hook-stdout");
    let stderr = fixture.root.path().join("hook-stderr");
    let started = Instant::now();
    let mut child = isolated_command("/bin/sh", &fixture.home)
        .args(["-c", command])
        .env("SNO_REACH_ADDR", "fixture@local")
        .env("SNO_TEST_REMIND_MODE", "hang")
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let mut status = None;
    while started.elapsed() < Duration::from_secs(8) {
        status = child.try_wait().unwrap();
        if status.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    if status.is_none() {
        child.kill().unwrap();
    }
    child.wait().unwrap();
    let pid = fs::read_to_string(fixture.home.join("remind-pid")).unwrap();
    let running = Command::new("/bin/kill")
        .args(["-0", pid.trim()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success();
    if running {
        let _ = Command::new("/bin/kill")
            .args(["-KILL", pid.trim()])
            .status();
    }
    assert!(
        status.is_some_and(|s| s.success()),
        "hook exceeded eight-second test bound"
    );
    assert!(
        !running,
        "timed-out Reach process {pid} survived its hook wrapper"
    );
    assert!(fs::read(stdout).unwrap().is_empty());
    assert!(fs::read_to_string(stderr).unwrap().contains("timed out"));
}

#[test]
fn utility_release_manifest_must_match_program_version_and_contract() {
    for case in [
        "missing",
        "version",
        "program",
        "contract",
        "contract bytes",
    ] {
        let fixture = Fixture::new();
        fixture.edit_program("heartbeat", |files| {
            if case == "missing" {
                files.retain(|(path, _, _)| path != "release.json");
                return;
            }
            for (path, bytes, _) in files {
                if path == "requirements-contract.json" && case == "contract bytes" {
                    bytes.push(b'\n');
                }
                if path != "release.json" {
                    continue;
                }
                let mut manifest: Value = serde_json::from_slice(bytes).unwrap();
                match case {
                    "version" => manifest["version"] = json!("9.9"),
                    "program" => manifest["program"] = json!("reach"),
                    "contract" => manifest["requirements_contract_sha256"] = json!("0".repeat(64)),
                    _ => (),
                }
                *bytes = serde_json::to_vec(&manifest).unwrap();
            }
        });
        let output = fixture.run(&["assemble"]);
        assert_eq!(
            output.status.code(),
            Some(3),
            "{case}: {}",
            diagnostic(&output)
        );
        assert!(!fixture.manifest().exists());
        assert!(!fixture.home.join(".local/lib/sno-reach").exists());
    }
}

#[test]
#[ignore = "requires explicitly supplied accepted local core utility archives; not published-release proof"]
fn accepted_core_utilities_install_with_synthetic_reach_and_skills() {
    let fixture = Fixture::new();
    let mut source: Value = serde_json::from_slice(&fs::read(&fixture.source).unwrap()).unwrap();
    for (program, variable) in [
        ("heartbeat", "SNO_TEST_HEARTBEAT_ARCHIVE"),
        ("subscription-quota-check", "SNO_TEST_QUOTA_ARCHIVE"),
    ] {
        let path =
            PathBuf::from(std::env::var_os(variable).expect("explicit accepted archive path"));
        let bytes = fs::read(&path).unwrap();
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
        let mut release = None;
        for entry in tar.entries().unwrap() {
            let mut entry = entry.unwrap();
            if entry.path().unwrap() == Path::new("release.json") {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).unwrap();
                release = Some(serde_json::from_slice::<Value>(&bytes).unwrap());
            }
        }
        let release = release.expect("accepted utility manifest");
        assert_eq!(release["program"], program);
        let version = release["version"].as_str().unwrap();
        let index = source["programs"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| a["name"] == program)
            .unwrap();
        source["programs"][index] = json!({"name":program,"version":version,"url":url::Url::from_file_path(&path).unwrap().as_str(),"sha256":hex::encode(Sha256::digest(&bytes)),"entry_point":format!("bin/{program}")});
        println!(
            "mixed fixture source {program} path={} sha256={}",
            path.display(),
            hex::encode(Sha256::digest(&bytes))
        );
    }
    fs::write(&fixture.source, serde_json::to_vec(&source).unwrap()).unwrap();
    fixture.success(&["assemble"]);
    let manifest: Value = serde_json::from_slice(&fs::read(fixture.manifest()).unwrap()).unwrap();
    for program in ["heartbeat", "subscription-quota-check"] {
        assert!(
            fs::metadata(fixture.home.join(".local/bin").join(program))
                .unwrap()
                .is_file()
        );
        assert!(
            !manifest["programs"][program]["dependencies"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
    let doctor = fixture.run(&["doctor"]);
    assert_ne!(doctor.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&doctor.stdout).contains("dependency"),
        "{}",
        diagnostic(&doctor)
    );
    fixture.success(&["remove"]);
}
