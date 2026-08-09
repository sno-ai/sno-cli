#[path = "support/sno_service_server.rs"]
mod sno_service_server;

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use fs2::FileExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sno_service_server::{ServiceResponse, SnoServiceServer};
use tempfile::TempDir;

fn sno(profile: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(arguments)
        .env("SNO_PROFILE_DIR", profile)
        .env("OPENCLAW_STATE_DIR", profile)
        .output()
        .expect("run sno")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr")
}

#[test]
fn root_help_version_and_missing_command_are_stable() {
    let profile = TempDir::new().expect("profile");
    let version = sno(profile.path(), &["--version"]);
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        stdout(&version),
        format!("sno {}\n", env!("CARGO_PKG_VERSION"))
    );

    let help = sno(profile.path(), &["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let help_text = stdout(&help);
    for expected in ["account", "station", "starport", "external subcommands"] {
        assert!(
            help_text.contains(expected),
            "missing {expected} in {help_text}"
        );
    }

    let missing = sno(profile.path(), &[]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(stdout(&missing).contains("Usage:"));
    assert!(stderr(&missing).is_empty());

    let missing_json = sno(profile.path(), &["--json"]);
    assert_eq!(missing_json.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing_json.stdout).expect("JSON error"),
        json!({"error":"usage_error","message":"missing command"})
    );
    assert!(stderr(&missing_json).is_empty());
}

#[test]
fn legacy_root_commands_are_rejected() {
    let profile = TempDir::new().expect("profile");
    for arguments in [
        vec!["consent", "get"],
        vec!["observe", "pause"],
        vec!["register"],
        vec!["claim"],
        vec!["audit", "verify", "evt_1"],
        vec!["doctor"],
    ] {
        let output = sno(profile.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
    }
}

#[test]
fn parser_and_export_usage_errors_match_the_migrated_contract() {
    let profile = TempDir::new().expect("profile");
    let unknown = sno(profile.path(), &["--bogus"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert_eq!(stderr(&unknown), "error: unknown option '--bogus'\n");

    let missing_event = sno(profile.path(), &["station", "audit", "verify", "--json"]);
    assert_eq!(missing_event.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing_event.stdout).expect("missing event JSON"),
        json!({"error":"usage_error","message":"missing required argument 'event_id'"})
    );

    let invalid_format = sno(
        profile.path(),
        &[
            "station",
            "telemetry",
            "export",
            "--format",
            "xml",
            "--json",
        ],
    );
    assert_eq!(invalid_format.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&invalid_format.stdout).expect("invalid format JSON"),
        json!({
            "error":"usage_error",
            "message":"invalid export format: expected one of tarball, jsonl, csv"
        })
    );

    let path_conflict = sno(
        profile.path(),
        &[
            "station",
            "telemetry",
            "export",
            "one.jsonl",
            "--out",
            "two.jsonl",
            "--json",
        ],
    );
    assert_eq!(path_conflict.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&path_conflict.stdout).expect("path conflict JSON"),
        json!({
            "error":"usage_error",
            "message":"provide either a positional path or --out, not both"
        })
    );

    for format in ["jsonl", "csv"] {
        let missing_output = sno(
            profile.path(),
            &[
                "station",
                "telemetry",
                "export",
                "--format",
                format,
                "--json",
            ],
        );
        assert_eq!(missing_output.status.code(), Some(2));
        assert_eq!(
            serde_json::from_slice::<Value>(&missing_output.stdout).expect("missing output JSON"),
            json!({
                "error":"usage_error",
                "message":"JSON mode requires an output path for jsonl or csv export"
            })
        );
    }
}

#[test]
fn consent_pause_resume_and_export_use_real_state() {
    let profile = TempDir::new().expect("profile");
    let get = sno(
        profile.path(),
        &["station", "telemetry", "consent", "get", "--json"],
    );
    assert_eq!(stdout(&get), "{\"consent\":\"metadata-only\"}\n");

    let set = sno(
        profile.path(),
        &["station", "telemetry", "consent", "set", "full", "--json"],
    );
    assert_eq!(set.status.code(), Some(0));
    let pause = sno(profile.path(), &["station", "telemetry", "pause", "--json"]);
    assert_eq!(pause.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&pause.stdout).expect("pause JSON"),
        json!({"consent":"off","paused":true,"already_paused":false})
    );
    let resume = sno(
        profile.path(),
        &["station", "telemetry", "resume", "--json"],
    );
    assert_eq!(stdout(&resume), "{\"consent\":\"full\"}\n");

    let export = sno(
        profile.path(),
        &["station", "telemetry", "export", "--format", "jsonl"],
    );
    assert_eq!(export.status.code(), Some(0));
    assert!(stdout(&export).lines().count() >= 7);

    let archive_path = profile.path().join("events.tar.gz");
    let tarball = sno(
        profile.path(),
        &[
            "station",
            "telemetry",
            "export",
            "--format",
            "tarball",
            "--out",
            archive_path.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(tarball.status.code(), Some(0), "{}", stderr(&tarball));
    let report: Value = serde_json::from_slice(&tarball.stdout).expect("tarball report");
    let archive_bytes = fs::read(&archive_path).expect("tarball");
    assert_eq!(report["bytes"], archive_bytes.len());
    assert_eq!(
        report["tarball_sha256"],
        hex::encode(Sha256::digest(&archive_bytes))
    );
    let decoder = flate2::read::GzDecoder::new(archive_bytes.as_slice());
    let mut archive = tar::Archive::new(decoder);
    let mut paths = Vec::new();
    for entry in archive.entries().expect("tar entries") {
        let mut entry = entry.expect("tar entry");
        paths.push(entry.path().unwrap().into_owned());
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).unwrap();
        assert!(!contents.is_empty());
    }
    assert_eq!(
        paths,
        [Path::new("events.jsonl"), Path::new("MANIFEST.json")]
    );
    assert!(profile.path().join("identity.json").is_file());
    assert!(profile.path().join("buffer.db").is_file());
    assert!(profile.path().join("state/consent.json").is_file());
}

#[test]
fn explicit_consent_change_invalidates_pause_and_failed_resume_stays_off() {
    let profile = TempDir::new().expect("profile");
    for arguments in [
        vec!["station", "telemetry", "consent", "set", "full", "--json"],
        vec!["station", "telemetry", "pause", "--json"],
        vec![
            "station",
            "telemetry",
            "consent",
            "set",
            "metadata-only",
            "--json",
        ],
    ] {
        assert_eq!(sno(profile.path(), &arguments).status.code(), Some(0));
    }
    let resume = sno(
        profile.path(),
        &["station", "telemetry", "resume", "--json"],
    );
    assert_eq!(stdout(&resume), "{\"consent\":\"metadata-only\"}\n");

    assert_eq!(
        sno(
            profile.path(),
            &["station", "telemetry", "consent", "set", "full", "--json",],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        sno(profile.path(), &["station", "telemetry", "pause", "--json"],)
            .status
            .code(),
        Some(0)
    );
    for suffix in ["", "-wal", "-shm"] {
        let _ = fs::remove_file(profile.path().join(format!("buffer.db{suffix}")));
    }
    fs::create_dir(profile.path().join("buffer.db")).expect("blocking buffer path");
    let failed_resume = sno(
        profile.path(),
        &["station", "telemetry", "resume", "--json"],
    );
    assert_eq!(failed_resume.status.code(), Some(1));
    let consent: Value = serde_json::from_str(
        &fs::read_to_string(profile.path().join("state/consent.json")).expect("consent state"),
    )
    .expect("consent JSON");
    assert_eq!(consent["value"], "off");
    assert!(
        !profile
            .path()
            .join("state/consent-transition.json")
            .exists()
    );
}

#[test]
fn explicit_opt_out_cannot_be_resumed_without_a_pause_record() {
    let profile = TempDir::new().expect("profile");
    assert_eq!(
        sno(
            profile.path(),
            &["station", "telemetry", "consent", "set", "off", "--json"],
        )
        .status
        .code(),
        Some(0)
    );
    let resume = sno(
        profile.path(),
        &["station", "telemetry", "resume", "--json"],
    );
    assert_eq!(resume.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&resume.stdout).expect("resume error JSON"),
        json!({
            "error": "telemetry_not_paused",
            "message": "telemetry is off by explicit consent; no paused setting can be resumed"
        })
    );
    let consent: Value = serde_json::from_str(
        &fs::read_to_string(profile.path().join("state/consent.json")).expect("consent state"),
    )
    .expect("consent JSON");
    assert_eq!(consent["value"], "off");
}

#[test]
fn concurrent_consent_commands_leave_file_at_latest_committed_value() {
    let profile = TempDir::new().expect("profile");
    let handles = (0..12)
        .map(|index| {
            let path = profile.path().to_path_buf();
            thread::spawn(move || {
                let value = if index % 2 == 0 { "full" } else { "off" };
                sno(
                    &path,
                    &["station", "telemetry", "consent", "set", value, "--json"],
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        let output = handle.join().expect("consent command");
        assert_eq!(
            output.status.code(),
            Some(0),
            "stdout: {}\nstderr: {}",
            stdout(&output),
            stderr(&output)
        );
    }

    let consent: Value = serde_json::from_str(
        &fs::read_to_string(profile.path().join("state/consent.json")).expect("consent state"),
    )
    .expect("consent JSON");
    let connection = rusqlite::Connection::open(profile.path().join("buffer.db")).unwrap();
    let committed: String = connection
        .query_row(
            "SELECT next_consent FROM consent_transitions ORDER BY rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(consent["value"], committed);
    assert!(
        !profile
            .path()
            .join("state/consent-transition.json")
            .exists()
    );
    assert!(profile.path().join("state/consent.lock").is_file());
}

#[test]
fn malformed_identity_fails_closed_without_replacement() {
    let profile = TempDir::new().expect("profile");
    fs::write(profile.path().join("identity.json"), "null\n").expect("identity fixture");
    let output = sno(
        profile.path(),
        &["station", "telemetry", "consent", "set", "full", "--json"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("identity error JSON")["error"],
        "invalid_identity"
    );
    assert_eq!(
        fs::read_to_string(profile.path().join("identity.json")).unwrap(),
        "null\n"
    );
}

#[test]
fn register_uses_anonymous_machine_identity_request() {
    let server = SnoServiceServer::start(vec![Box::new(|request| {
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/api/v1/identity/register-machine");
        assert!(!request.headers.contains_key("authorization"));
        let body: Value = serde_json::from_str(&request.body).expect("registration body");
        assert!(body.get("machine_secret").is_none());
        ServiceResponse::json(
            200,
            json!({
                "user_cuid": body["user_cuid"],
                "machine_uuid": body["machine_uuid"],
                "claimed": false,
            }),
        )
    })]);
    let profile = TempDir::new().expect("profile");
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["account", "machine", "register", "--json"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("SNO_OBSERVE_BASE_URL", server.base_url())
        .output()
        .expect("register");
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value: Value = serde_json::from_slice(&output.stdout).expect("register JSON");
    assert_eq!(value["registered"], true);
    assert_eq!(value["claimed"], false);
    let requests = server.finish();
    assert_eq!(requests.len(), 1);

    let identity = fs::read_to_string(profile.path().join("identity.json")).expect("identity");
    assert!(
        !stdout(&output).contains(
            serde_json::from_str::<Value>(&identity).unwrap()["machine_secret"]
                .as_str()
                .unwrap()
        )
    );
}

#[test]
fn registration_preserves_server_error_code() {
    let server = SnoServiceServer::start(vec![Box::new(|_| {
        ServiceResponse::json(
            409,
            json!({
                "error":"machine_secret_mismatch",
                "message":"machine secret mismatch"
            }),
        )
    })]);
    let profile = TempDir::new().expect("profile");
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["account", "machine", "register", "--json"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("SNO_OBSERVE_BASE_URL", server.base_url())
        .output()
        .expect("register");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("registration error JSON"),
        json!({
            "error":"machine_secret_mismatch",
            "message":"machine registration failed with HTTP 409: machine secret mismatch"
        })
    );
    assert!(stderr(&output).is_empty());
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn claim_uses_device_flow_without_authorization_headers() {
    let account_id = "a23456789012345678901234";
    let server = SnoServiceServer::start(vec![
        Box::new(|request| {
            let body: Value = serde_json::from_str(&request.body).expect("registration body");
            ServiceResponse::json(
                200,
                json!({
                    "user_cuid": body["user_cuid"],
                    "machine_uuid": body["machine_uuid"],
                    "claimed": false,
                }),
            )
        }),
        Box::new(|request| {
            assert_eq!(request.target, "/api/v1/device/code");
            ServiceResponse::json(
                200,
                json!({
                    "device_code":"device-secret",
                    "user_code":"SNO-CODE",
                    "verification_uri":"https://www.sno.ai/cli/connect",
                    "verification_uri_complete":"https://www.sno.ai/cli/connect?code=SNO-CODE",
                    "expires_in":1800,
                    "interval":1
                }),
            )
        }),
        Box::new(move |request| {
            assert_eq!(request.target, "/api/v1/device/token");
            ServiceResponse::json(
                200,
                json!({"user_account_id":account_id,"status":"claimed"}),
            )
        }),
    ]);
    let profile = TempDir::new().expect("profile");
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["account", "machine", "claim", "--json"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("SNO_OBSERVE_BASE_URL", server.base_url())
        .output()
        .expect("claim");
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("claim JSON line"))
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["type"], "authorization");
    assert_eq!(lines[0]["user_code"], "SNO-CODE");
    assert_eq!(lines[1]["type"], "result");
    assert_eq!(lines[1]["claimed"], true);
    assert_eq!(lines[1]["user_account_id"], account_id);
    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .all(|request| !request.headers.contains_key("authorization"))
    );
}

#[test]
fn claim_retries_transient_http_status() {
    let account_id = "a23456789012345678901234";
    let server = SnoServiceServer::start(vec![
        Box::new(|request| {
            let body: Value = serde_json::from_str(&request.body).expect("registration body");
            ServiceResponse::json(
                200,
                json!({
                    "user_cuid": body["user_cuid"],
                    "machine_uuid": body["machine_uuid"],
                    "claimed": false,
                }),
            )
        }),
        Box::new(|_| {
            ServiceResponse::json(
                200,
                json!({
                    "device_code":"device-secret",
                    "user_code":"SNO-CODE",
                    "verification_uri":"https://www.sno.ai/cli/connect",
                    "expires_in":1800,
                    "interval":1
                }),
            )
        }),
        Box::new(|_| ServiceResponse {
            status: 503,
            body: "temporary outage".to_owned(),
        }),
        Box::new(move |_| {
            ServiceResponse::json(
                200,
                json!({"user_account_id":account_id,"status":"claimed"}),
            )
        }),
    ]);
    let profile = TempDir::new().expect("profile");
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["account", "machine", "claim", "--json"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("SNO_OBSERVE_BASE_URL", server.base_url())
        .output()
        .expect("claim");
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("claim JSON line"))
        .collect::<Vec<_>>();
    assert_eq!(lines[1]["user_account_id"], account_id);
    assert_eq!(server.finish().len(), 4);
}

#[test]
fn json_claim_flushes_authorization_before_browser_approval() {
    let server = SnoServiceServer::start(vec![
        Box::new(|request| {
            let body: Value = serde_json::from_str(&request.body).expect("registration body");
            ServiceResponse::json(
                200,
                json!({
                    "user_cuid": body["user_cuid"],
                    "machine_uuid": body["machine_uuid"],
                    "claimed": false,
                }),
            )
        }),
        Box::new(|_| {
            ServiceResponse::json(
                200,
                json!({
                    "device_code":"device-secret",
                    "user_code":"SNO-CODE",
                    "verification_uri":"https://www.sno.ai/cli/connect",
                    "expires_in":1800,
                    "interval":30
                }),
            )
        }),
    ]);
    let profile = TempDir::new().expect("profile");
    let mut child = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["account", "machine", "claim", "--json"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("SNO_OBSERVE_BASE_URL", server.base_url())
        .stdout(Stdio::piped())
        .spawn()
        .expect("claim process");
    let output = child.stdout.take().expect("claim stdout");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(output)
            .read_line(&mut line)
            .expect("read authorization line");
        sender.send(line).expect("send authorization line");
    });
    let line = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("authorization JSON was not flushed before polling");
    let value: Value = serde_json::from_str(&line).expect("authorization JSON");
    assert_eq!(value["type"], "authorization");
    assert_eq!(value["user_code"], "SNO-CODE");
    child.kill().expect("stop pending claim");
    child.wait().expect("reap pending claim");
    assert_eq!(server.finish().len(), 2);
}

#[test]
fn audit_verify_registers_then_uses_machine_bearer() {
    let server = SnoServiceServer::start(vec![
        Box::new(|request| {
            let body: Value = serde_json::from_str(&request.body).expect("registration body");
            ServiceResponse::json(
                200,
                json!({
                    "user_cuid": body["user_cuid"],
                    "machine_uuid": body["machine_uuid"],
                    "claimed": false,
                }),
            )
        }),
        Box::new(|request| {
            assert_eq!(request.method, "GET");
            assert_eq!(request.target, "/api/v1/audit/verify?event_id=evt%2F1");
            assert!(
                request
                    .headers
                    .get("authorization")
                    .is_some_and(|value| value.starts_with("Bearer "))
            );
            ServiceResponse::json(200, json!({"verified":true,"anchor_id":"anchor_1"}))
        }),
    ]);
    let profile = TempDir::new().expect("profile");
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["station", "audit", "verify", "evt/1", "--json"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("SNO_OBSERVE_BASE_URL", server.base_url())
        .output()
        .expect("audit verify");
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("audit JSON")["verified"],
        true
    );
    assert_eq!(server.finish().len(), 2);
}

#[test]
fn doctor_reports_stable_order_and_malformed_identity_without_mutation() {
    let profile = TempDir::new().expect("profile");
    fs::write(profile.path().join("identity.json"), "null\n").expect("identity fixture");
    let output = sno(profile.path(), &["station", "doctor", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let text = stdout(&output);
    let value: Value = serde_json::from_str(&text).expect("doctor JSON");
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["identity", "buffer", "consent", "last_ship", "lockfile"]
    );
    assert_eq!(value["identity"]["status"], "fail");
    assert!(
        value["identity"]["detail"]
            .as_str()
            .unwrap()
            .contains("identity is malformed")
    );
    assert_eq!(
        fs::read_to_string(profile.path().join("identity.json")).unwrap(),
        "null\n"
    );
}

#[cfg(unix)]
#[test]
fn external_subcommand_preserves_literal_arguments_and_exit_code() {
    use std::os::unix::fs::PermissionsExt;

    let profile = TempDir::new().expect("profile");
    let bin = TempDir::new().expect("bin");
    let executable = bin.path().join("sno-demo");
    fs::write(&executable, "#!/bin/sh\nprintf '%s\\n' \"$@\"\nexit 7\n").expect("script");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).expect("executable");
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(["demo", "literal;$HOME", "$(false)"])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("PATH", bin.path())
        .output()
        .expect("external command");
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(stdout(&output), "literal;$HOME\n$(false)\n");
}

#[test]
fn rem_start_posts_type_scope_and_boot_token() {
    let profile = TempDir::new().expect("profile");
    let trace_path = profile.path().join("mem-claw").join("rem-trace.jsonl");
    let ignored_override = profile.path().join("unauthorized-third-trace.jsonl");
    let server = SnoServiceServer::start(vec![Box::new(|request| {
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/rem/run");
        assert_eq!(
            request.headers.get("x-sidecar-token").map(String::as_str),
            Some("boot-token-1")
        );
        assert_eq!(
            request
                .headers
                .get("x-rem-correlation-id")
                .map(String::as_str),
            Some("corr-start-019f8da3")
        );
        assert_eq!(
            serde_json::from_str::<Value>(&request.body).expect("REM start body"),
            json!({"type":"noop","scope":"persona:test-68a19d8c"})
        );
        ServiceResponse::json(202, json!({"job_id":"job-019f8da3"}))
    })]);
    write_rem_discovery(profile.path(), service_port(&server), "boot-token-1");

    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args([
            "station",
            "rem-start",
            "--type",
            "noop",
            "--scope",
            "persona:test-68a19d8c",
            "--json",
        ])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("OPENCLAW_STATE_DIR", profile.path())
        .env("SNO_REM_CORRELATION_ID", "corr-start-019f8da3")
        .env("SNO_REM_TRACE_FILE", &ignored_override)
        .output()
        .expect("REM start");

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("REM start JSON"),
        json!({"job_id":"job-019f8da3","correlation_id":"corr-start-019f8da3"})
    );
    assert_eq!(server.finish().len(), 1);
    let trace = fs::read_to_string(trace_path).expect("REM trace");
    for event in [
        "command_received",
        "discovery_resolved",
        "http_request_sent",
        "http_response_received",
    ] {
        assert!(trace.contains(&format!(r#""event":"{event}""#)), "{trace}");
    }
    assert!(trace.contains(r#""correlation_id":"corr-start-019f8da3""#));
    assert!(!trace.contains("boot-token-1"));
    assert!(!ignored_override.exists());
}

#[test]
fn rem_start_surfaces_failed_allocated_job() {
    let profile = TempDir::new().expect("profile");
    let server = SnoServiceServer::start(vec![Box::new(|_| {
        ServiceResponse::json(
            400,
            json!({"job_id":"job-unsupported","error":"unsupported_rem_type"}),
        )
    })]);
    write_rem_discovery(profile.path(), service_port(&server), "boot-token-2");

    let output = sno(
        profile.path(),
        &[
            "station",
            "rem-start",
            "--type",
            "rem-review",
            "--scope",
            "persona:test-68a19d8c",
            "--json",
        ],
    );

    assert_eq!(output.status.code(), Some(1));
    let error = serde_json::from_slice::<Value>(&output.stdout).expect("allocated failure JSON");
    assert_eq!(error["error"], "unsupported_rem_type");
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| message.contains("job-unsupported"))
    );
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn rem_human_output_keeps_job_id_actionable_on_success_and_failure() {
    let profile = TempDir::new().expect("profile");
    let server = SnoServiceServer::start(vec![
        Box::new(|_| ServiceResponse::json(202, json!({"job_id":"job-human-start"}))),
        Box::new(|_| {
            ServiceResponse::json(
                200,
                json!({
                    "state":"running",
                    "type":"noop",
                    "scope":"persona:human",
                    "started_at":"2026-07-23T06:00:00Z",
                    "finished_at":null,
                    "stats":null,
                    "error":null,
                    "correlation_id":null
                }),
            )
        }),
        Box::new(|_| {
            ServiceResponse::json(
                200,
                json!({
                    "state":"failed",
                    "type":"noop",
                    "scope":"persona:human",
                    "started_at":"2026-07-23T06:00:00Z",
                    "finished_at":"2026-07-23T06:00:01Z",
                    "stats":null,
                    "error":"sidecar_restart",
                    "correlation_id":null
                }),
            )
        }),
        Box::new(|_| {
            ServiceResponse::json(
                400,
                json!({"job_id":"job-human-rejected","error":"unsupported_rem_type"}),
            )
        }),
    ]);
    write_rem_discovery(profile.path(), service_port(&server), "boot-token-human");

    let started = sno(
        profile.path(),
        &[
            "station",
            "rem-start",
            "--type",
            "noop",
            "--scope",
            "persona:human",
        ],
    );
    assert_eq!(started.status.code(), Some(0), "{}", stderr(&started));
    assert!(stdout(&started).contains("job-human-start"));

    let running = sno(
        profile.path(),
        &["station", "rem-status", "job-human-start"],
    );
    assert_eq!(running.status.code(), Some(0), "{}", stderr(&running));
    assert!(stdout(&running).contains("job-human-start"));
    assert!(stdout(&running).contains("running"));

    let failed = sno(
        profile.path(),
        &["station", "rem-status", "job-human-start"],
    );
    assert_eq!(failed.status.code(), Some(3));
    assert!(stderr(&failed).contains("job-human-start"));
    assert!(stderr(&failed).contains("sidecar_restart"));

    let rejected = sno(
        profile.path(),
        &[
            "station",
            "rem-start",
            "--type",
            "rem-review",
            "--scope",
            "persona:human",
        ],
    );
    assert_eq!(rejected.status.code(), Some(1));
    assert!(stderr(&rejected).contains("job-human-rejected"));
    assert!(stderr(&rejected).contains("unsupported_rem_type"));
    assert_eq!(server.finish().len(), 4);
}

#[test]
fn rem_one_shot_status_reads_running_then_stable_done() {
    let profile = TempDir::new().expect("profile");
    let server = SnoServiceServer::start(vec![
        Box::new(|_| {
            ServiceResponse::json(
                200,
                json!({
                    "state":"running",
                    "type":"noop",
                    "scope":"persona:one-shot",
                    "started_at":"2026-07-23T06:00:00Z",
                    "finished_at":null,
                    "stats":null,
                    "error":null,
                    "correlation_id":null
                }),
            )
        }),
        Box::new(|_| ServiceResponse::json(200, rem_done_job())),
        Box::new(|_| ServiceResponse::json(200, rem_done_job())),
    ]);
    write_rem_discovery(profile.path(), service_port(&server), "boot-token-one-shot");

    for expected in ["running", "done", "done"] {
        let output = sno(
            profile.path(),
            &["station", "rem-status", "job-one-shot", "--json"],
        );
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).expect("REM status JSON")["state"],
            expected
        );
    }
    assert_eq!(server.finish().len(), 3);
}

#[test]
fn rem_exit_codes_are_stable() {
    let profile = TempDir::new().expect("profile");
    let server = SnoServiceServer::start(vec![
        Box::new(|_| ServiceResponse::json(200, rem_done_job())),
        Box::new(|_| {
            ServiceResponse::json(
                200,
                json!({
                    "state":"failed",
                    "type":"noop",
                    "scope":"persona:test-68a19d8c",
                    "started_at":"2026-07-23T06:00:00Z",
                    "finished_at":"2026-07-23T06:00:01Z",
                    "stats":null,
                    "error":"sidecar_restart",
                    "correlation_id":null
                }),
            )
        }),
    ]);
    write_rem_discovery(profile.path(), service_port(&server), "boot-token-exits");

    let done = sno(
        profile.path(),
        &["station", "rem-status", "job-done", "--json"],
    );
    let failed = sno(
        profile.path(),
        &["station", "rem-status", "job-failed", "--json"],
    );
    assert_eq!(server.finish().len(), 2);
    fs::remove_file(profile.path().join("station/sidecar.json")).expect("remove discovery");
    let timeout = sno(
        profile.path(),
        &[
            "station",
            "rem-status",
            "job-timeout",
            "--wait",
            "--timeout",
            "1",
            "--json",
        ],
    );
    let usage = sno(profile.path(), &["station", "rem-start", "--type", "noop"]);

    for (label, output, expected) in [
        ("done", done, 0),
        ("failed", failed, 3),
        ("timeout", timeout, 4),
        ("invalid usage", usage, 2),
    ] {
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{label}: stdout={} stderr={}",
            stdout(&output),
            stderr(&output),
        );
    }
}

#[test]
fn rem_common_local_errors_are_clean() {
    let missing_profile = TempDir::new().expect("missing profile");
    let missing = sno(
        missing_profile.path(),
        &[
            "station",
            "rem-start",
            "--type",
            "noop",
            "--scope",
            "persona:missing",
        ],
    );
    assert_eq!(missing.status.code(), Some(7));
    assert!(stderr(&missing).contains("sidecar not running"));
    assert!(!stderr(&missing).contains("panicked"));

    let malformed_profile = TempDir::new().expect("malformed profile");
    fs::create_dir_all(malformed_profile.path().join("station")).expect("station directory");
    fs::write(
        malformed_profile.path().join("station/sidecar.json"),
        r#"{"port":"#,
    )
    .expect("malformed discovery");
    let malformed = sno(
        malformed_profile.path(),
        &[
            "station",
            "rem-start",
            "--type",
            "noop",
            "--scope",
            "persona:malformed",
        ],
    );
    assert_eq!(malformed.status.code(), Some(7));
    assert!(stderr(&malformed).contains("discovery is malformed"));
    assert!(!stderr(&malformed).contains("already running"));
    assert!(!stderr(&malformed).contains("panicked"));

    let unknown_profile = TempDir::new().expect("unknown profile");
    let server = SnoServiceServer::start(vec![Box::new(|_| {
        ServiceResponse::json(404, json!({"error":"job_not_found"}))
    })]);
    write_rem_discovery(
        unknown_profile.path(),
        service_port(&server),
        "boot-token-unknown",
    );
    let unknown = sno(
        unknown_profile.path(),
        &["station", "rem-status", "job-does-not-exist"],
    );
    assert_eq!(unknown.status.code(), Some(9));
    assert!(stderr(&unknown).contains("job-does-not-exist"));
    assert!(!stderr(&unknown).contains("panicked"));
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn rem_wait_rereads_discovery_after_sidecar_restart() {
    let profile = TempDir::new().expect("profile");
    let trace_path = profile.path().join("mem-claw").join("rem-trace.jsonl");
    let done_server = SnoServiceServer::start(vec![Box::new(|request| {
        assert_eq!(request.target, "/rem/jobs/job-restarted");
        assert_eq!(
            request
                .headers
                .get("x-rem-correlation-id")
                .map(String::as_str),
            Some("corr-wait-019f8da3")
        );
        ServiceResponse::json(200, rem_done_job())
    })]);
    let done_port = service_port(&done_server);
    let profile_path = profile.path().to_path_buf();
    let old_server = SnoServiceServer::start(vec![Box::new(move |request| {
        assert_eq!(
            request
                .headers
                .get("x-rem-correlation-id")
                .map(String::as_str),
            Some("corr-wait-019f8da3")
        );
        write_rem_discovery(&profile_path, done_port, "new-token");
        ServiceResponse::json(401, json!({"error":"unauthorized"}))
    })]);
    write_rem_discovery(profile.path(), service_port(&old_server), "old-token");

    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args([
            "station",
            "rem-status",
            "job-restarted",
            "--wait",
            "--timeout",
            "2",
            "--json",
        ])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("OPENCLAW_STATE_DIR", profile.path())
        .env("SNO_REM_CORRELATION_ID", "corr-wait-019f8da3")
        .output()
        .expect("REM wait");

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let job = serde_json::from_slice::<Value>(&output.stdout).expect("REM status JSON");
    assert_eq!(job["state"], "done");
    assert_eq!(job["stats"]["operations"], 0);
    assert_eq!(old_server.finish().len(), 1);
    assert_eq!(done_server.finish().len(), 1);
    let trace = fs::read_to_string(trace_path).expect("REM trace");
    assert!(trace.contains(r#""poll_index":1"#), "{trace}");
    assert!(
        trace.contains(r#""transient_class":"unauthorized""#),
        "{trace}"
    );
    assert!(trace.contains(r#""poll_index":2"#), "{trace}");
    assert!(!trace.contains("old-token"));
    assert!(!trace.contains("new-token"));
}

#[test]
fn rem_wait_retries_after_truncated_restart_response() {
    let profile = TempDir::new().expect("profile");
    let done_server = SnoServiceServer::start(vec![Box::new(|_| {
        ServiceResponse::json(200, rem_done_job())
    })]);
    let done_port = service_port(&done_server);
    let profile_path = profile.path().to_path_buf();
    let truncated_server = SnoServiceServer::start(vec![Box::new(move |_| {
        write_rem_discovery(&profile_path, done_port, "new-token");
        ServiceResponse::truncated_json(r#"{"state":"run"#)
    })]);
    write_rem_discovery(profile.path(), service_port(&truncated_server), "old-token");

    let output = sno(
        profile.path(),
        &[
            "station",
            "rem-status",
            "job-restarted",
            "--wait",
            "--timeout",
            "2",
            "--json",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("REM status JSON")["state"],
        "done"
    );
    assert_eq!(truncated_server.finish().len(), 1);
    assert_eq!(done_server.finish().len(), 1);
}

#[test]
fn rem_wait_times_out_when_sidecar_never_appears() {
    let profile = TempDir::new().expect("profile");
    let started = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args([
            "station",
            "rem-status",
            "job-missing",
            "--wait",
            "--timeout",
            "1",
            "--json",
        ])
        .env("SNO_PROFILE_DIR", profile.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("REM status process");
    let patience = Instant::now() + Duration::from_secs(2);
    loop {
        if child.try_wait().expect("poll REM status process").is_some() {
            break;
        }
        if Instant::now() >= patience {
            child.kill().expect("kill hung REM status process");
            child.wait().expect("reap hung REM status process");
            panic!("REM status exceeded the test patience bound");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("collect REM status output");

    assert_eq!(output.status.code(), Some(4));
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("REM timeout JSON")["error"],
        "rem_timeout"
    );
}

#[test]
fn rem_wait_timeout_survives_a_stalled_trace_writer() {
    let profile = TempDir::new().expect("profile");
    let trace_dir = profile.path().join("mem-claw");
    fs::create_dir_all(&trace_dir).expect("trace directory");
    let trace_path = trace_dir.join("rem-trace.jsonl");
    let trace_lock = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path)
        .expect("trace file");
    trace_lock.lock_exclusive().expect("hold trace lock");

    let started = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args([
            "station",
            "rem-status",
            "job-missing",
            "--wait",
            "--timeout",
            "1",
            "--json",
        ])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("OPENCLAW_STATE_DIR", profile.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("REM status process");
    let patience = Instant::now() + Duration::from_secs(2);
    loop {
        if child.try_wait().expect("poll REM status process").is_some() {
            break;
        }
        if Instant::now() >= patience {
            child.kill().expect("kill hung REM status process");
            child.wait().expect("reap hung REM status process");
            panic!("REM status trace blocked past the wait deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("collect REM status output");
    FileExt::unlock(&trace_lock).expect("release trace lock");

    assert_eq!(output.status.code(), Some(4));
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("REM timeout JSON")["error"],
        "rem_timeout"
    );
}

fn service_port(server: &SnoServiceServer) -> u16 {
    server
        .base_url()
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse().ok())
        .expect("loopback service port")
}

fn write_rem_discovery(profile: &Path, port: u16, token: &str) {
    let station = profile.join("station");
    fs::create_dir_all(&station).expect("station directory");
    fs::write(
        station.join("sidecar.json"),
        json!({"port":port,"token":token,"pid":123}).to_string(),
    )
    .expect("REM discovery");
}

fn rem_done_job() -> Value {
    json!({
        "state":"done",
        "type":"noop",
        "scope":"persona:test-68a19d8c",
        "started_at":"2026-07-23T06:00:00Z",
        "finished_at":"2026-07-23T06:00:01Z",
        "stats":{"operations":0}
    })
}
