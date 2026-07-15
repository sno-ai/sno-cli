#[path = "support/sno_service_server.rs"]
mod sno_service_server;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};
use sno_service_server::{ServiceResponse, SnoServiceServer};
use tempfile::TempDir;

fn sno(profile: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sno"))
        .args(arguments)
        .env("SNO_PROFILE_DIR", profile)
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
    assert_eq!(stdout(&version), "sno 0.1.0\n");

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
    assert!(profile.path().join("identity.json").is_file());
    assert!(profile.path().join("buffer.db").is_file());
    assert!(profile.path().join("state/consent.json").is_file());
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
    let value: Value = serde_json::from_slice(&output.stdout).expect("claim JSON");
    assert_eq!(value["claimed"], true);
    assert_eq!(value["user_account_id"], account_id);
    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .all(|request| !request.headers.contains_key("authorization"))
    );
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
