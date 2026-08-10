#[allow(dead_code)]
#[path = "support/sno_service_server.rs"]
mod sno_service_server;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use sno_service_server::{CapturedRequest, ServiceResponse, SnoServiceServer};
use tempfile::TempDir;

#[derive(Clone, Debug)]
struct ContractRow {
    name: &'static str,
    exit: i32,
    errors: Vec<&'static str>,
}

fn expected_rows() -> Vec<ContractRow> {
    vec![
        ContractRow {
            name: "success",
            exit: 0,
            errors: vec![],
        },
        ContractRow {
            name: "unclassified failure",
            exit: 1,
            errors: vec![],
        },
        ContractRow {
            name: "invalid usage",
            exit: 2,
            errors: vec!["usage_error"],
        },
        ContractRow {
            name: "job failed",
            exit: 3,
            errors: vec!["rem_job_failed"],
        },
        ContractRow {
            name: "wait deadline passed",
            exit: 4,
            errors: vec!["rem_timeout"],
        },
        ContractRow {
            name: "state vocabulary mismatch",
            exit: 5,
            errors: vec!["rem_state_unrecognised"],
        },
        ContractRow {
            name: "malformed or truncated response",
            exit: 6,
            errors: vec!["sidecar_response_invalid", "sidecar_response_truncated"],
        },
        ContractRow {
            name: "sidecar failure",
            exit: 7,
            errors: vec![
                "sidecar_not_running",
                "sidecar_unauthorized",
                "sidecar_client_error",
                "sidecar_discovery_error",
                "sidecar_discovery_invalid",
                "sidecar_response_error",
            ],
        },
        ContractRow {
            name: "local environment failure",
            exit: 8,
            errors: vec!["profile_error", "rem_trace_error"],
        },
        ContractRow {
            name: "unknown job identifier",
            exit: 9,
            errors: vec!["rem_job_not_found"],
        },
    ]
}

fn validate_rows(rows: &[ContractRow]) -> Result<(), String> {
    let mut exits = BTreeSet::new();
    let mut errors = BTreeSet::new();
    for row in rows {
        if !exits.insert(row.exit) {
            return Err(format!("duplicate exit {}", row.exit));
        }
        for error in &row.errors {
            if !errors.insert(*error) {
                return Err(format!("duplicate error {error}"));
            }
        }
    }
    Ok(())
}

fn rust_sources() -> Vec<(PathBuf, String)> {
    fn visit(directory: &Path, sources: &mut Vec<(PathBuf, String)>) {
        for entry in fs::read_dir(directory).expect("read Rust source directory") {
            let path = entry.expect("source entry").path();
            if path.is_dir() {
                visit(&path, sources);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("read Rust source");
                sources.push((path, source));
            }
        }
    }

    let mut sources = Vec::new();
    visit(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut sources,
    );
    sources
}

fn declaration_source() -> Result<(PathBuf, String), String> {
    let rows = expected_rows();
    let required = rows
        .iter()
        .flat_map(|row| std::iter::once(row.name).chain(row.errors.iter().copied()))
        .collect::<Vec<_>>();
    let matches = rust_sources()
        .into_iter()
        .filter(|(_, source)| {
            required
                .iter()
                .all(|value| source.contains(&format!("\"{value}\"")))
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [(path, source)] => Ok((path.clone(), source.clone())),
        [] => Err(
            "missing single REM outcome declaration containing the released semantic rows"
                .to_owned(),
        ),
        _ => Err(format!(
            "multiple REM outcome declaration owners: {}",
            matches
                .iter()
                .map(|(path, _)| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn statement_around(source: &str, needle: &str) -> Result<String, String> {
    let position = source
        .find(needle)
        .ok_or_else(|| format!("missing process-exit boundary `{needle}`"))?;
    let before = &source[..position];
    let start = before.rfind([';', '{', '}']).map_or(0, |index| index + 1);
    let after = &source[position..];
    let end = after
        .find(';')
        .map_or(source.len(), |index| position + index + 1);
    Ok(source[start..end].to_owned())
}

fn resolver_name(statement: &str) -> Result<String, String> {
    if statement.contains("error.exit_code") && !statement.contains("error.exit_code(") {
        return Err(format!(
            "process-exit boundary reads error.exit_code directly: {}",
            statement.trim()
        ));
    }

    if let Some(position) = statement.find("error.") {
        let suffix = &statement[position + "error.".len()..];
        let name = suffix
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !name.is_empty() && suffix[name.len()..].trim_start().starts_with('(') {
            return Ok(name);
        }
    }

    let mut candidates = Vec::new();
    for (open, _) in statement.match_indices('(') {
        let prefix = statement[..open].trim_end();
        let name = prefix
            .chars()
            .rev()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        if !matches!(name.as_str(), "Ok" | "Some" | "Err" | "json" | "format")
            && statement[open + 1..].contains("error")
        {
            candidates.push(name);
        }
    }
    candidates
        .into_iter()
        .last()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            format!(
                "no declaration-backed exit resolver in `{}`",
                statement.trim()
            )
        })
}

fn service_exit_statement(source: &str) -> Result<String, String> {
    let branch = source
        .find("Err(error) if json_enabled")
        .ok_or_else(|| "missing JSON service error branch".to_owned())?;
    let suffix = &source[branch..];
    let returned = suffix
        .find("return Ok(")
        .ok_or_else(|| "missing service process-exit return".to_owned())?;
    statement_around(&suffix[returned..], "error")
}

fn write_rem_discovery(profile: &Path, server: &SnoServiceServer) {
    let port = server
        .base_url()
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
        .expect("loopback port");
    let station = profile.join("station");
    fs::create_dir_all(&station).expect("station directory");
    fs::write(
        station.join("sidecar.json"),
        json!({"port":port,"token":"rem-contract-token","pid":123}).to_string(),
    )
    .expect("REM discovery");
}

fn observe_rem_error(code: &'static str) -> (i32, String) {
    let server = SnoServiceServer::start(vec![Box::new(move |_| {
        ServiceResponse::json(
            400,
            json!({"job_id":"job-019f8da3-rem-contract","error":code}),
        )
    })]);
    let profile = TempDir::new().expect("profile");
    write_rem_discovery(profile.path(), &server);
    let output = Command::new(env!("CARGO_BIN_EXE_sno"))
        .args([
            "station",
            "rem-start",
            "--type",
            "noop",
            "--scope",
            "persona:test-rem-contract-019f8da3",
            "--json",
        ])
        .env("SNO_PROFILE_DIR", profile.path())
        .env("OPENCLAW_STATE_DIR", profile.path())
        .env("SNO_REM_TRACE", "0")
        .output()
        .expect("run sno REM error boundary");
    assert_eq!(server.finish().len(), 1, "{code}: request count");
    let value: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{code}: invalid JSON stdout ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    let observed_code = value["error"]
        .as_str()
        .unwrap_or_else(|| panic!("{code}: missing JSON error code in {value}"))
        .to_owned();
    (
        output.status.code().expect("process exit code"),
        observed_code,
    )
}

const SECTION_3_JOB_ID: &str = "job-019f8da3-section-3";

enum FixtureResponse {
    Json(u16, Value),
    Raw(&'static str),
}

impl FixtureResponse {
    fn render(&self) -> ServiceResponse {
        match self {
            Self::Json(status, value) => ServiceResponse::json(*status, value.clone()),
            Self::Raw(body) => ServiceResponse {
                status: 200,
                body: (*body).to_owned(),
            },
        }
    }
}

fn rem_job(state: &str, error: Option<&str>) -> Value {
    json!({
        "state": state,
        "type": "noop",
        "scope": "persona:test-rem-state-019f8da3",
        "started_at": "2026-08-09T20:00:00Z",
        "finished_at": if matches!(state, "done" | "failed") {
            Some("2026-08-09T20:00:01Z")
        } else {
            None
        },
        "stats": if state == "done" { Some(json!({"operations": 0})) } else { None },
        "error": error,
        "correlation_id": null
    })
}

fn rem_job_without_error(state: &str) -> Value {
    let mut value = rem_job(state, None);
    value
        .as_object_mut()
        .expect("REM job object")
        .remove("error");
    value
}

fn run_status(responses: Vec<FixtureResponse>, wait: bool, json_enabled: bool) -> Output {
    let response_count = responses.len();
    let handlers = responses
        .into_iter()
        .map(|response| {
            Box::new(move |_: CapturedRequest| response.render())
                as Box<dyn Fn(CapturedRequest) -> ServiceResponse + Send>
        })
        .collect();
    let server = SnoServiceServer::start(handlers);
    let profile = TempDir::new().expect("profile");
    write_rem_discovery(profile.path(), &server);
    let mut command = Command::new(env!("CARGO_BIN_EXE_sno"));
    command.args(["station", "rem-status", SECTION_3_JOB_ID]);
    if wait {
        command.args(["--wait", "--timeout", "1"]);
    }
    if json_enabled {
        command.arg("--json");
    }
    let output = command
        .env("SNO_PROFILE_DIR", profile.path())
        .env("OPENCLAW_STATE_DIR", profile.path())
        .env("SNO_REM_TRACE", "0")
        .output()
        .expect("run sno REM status");
    assert_eq!(server.finish().len(), response_count, "request count");
    output
}

fn output_text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8 process output")
}

fn last_json_line(output: &Output) -> Value {
    let text = output_text(&output.stdout);
    serde_json::from_str(text.lines().last().expect("JSON output line"))
        .unwrap_or_else(|error| panic!("invalid final JSON line ({error}): {text}"))
}

#[cfg(unix)]
fn run_status_shell(response: Value, script: &str) -> Output {
    let server = SnoServiceServer::start(vec![Box::new(move |_| {
        ServiceResponse::json(200, response.clone())
    })]);
    let profile = TempDir::new().expect("profile");
    write_rem_discovery(profile.path(), &server);
    let output = Command::new("/bin/sh")
        .args(["-c", script])
        .env("SNO_BIN", env!("CARGO_BIN_EXE_sno"))
        .env("JOB_ID", SECTION_3_JOB_ID)
        .env("SNO_PROFILE_DIR", profile.path())
        .env("OPENCLAW_STATE_DIR", profile.path())
        .env("SNO_REM_TRACE", "0")
        .output()
        .expect("run shell REM status caller");
    assert_eq!(server.finish().len(), 1, "shell request count");
    output
}

#[test]
fn qcg_1_single_declaration_owns_codes() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cli = fs::read_to_string(manifest.join("src/cli.rs")).expect("cli source");
    let service = fs::read_to_string(manifest.join("src/service.rs")).expect("service source");
    let cli_statement = statement_around(&cli, "error.exit_code").unwrap_or_else(|_| {
        let body = cli
            .split_once("fn print_cli_error")
            .map(|(_, suffix)| suffix)
            .unwrap_or(&cli);
        statement_around(body, "error").expect("CLI process-exit statement")
    });
    let service_statement =
        service_exit_statement(&service).expect("service process-exit statement");
    let cli_resolver = resolver_name(&cli_statement);
    let service_resolver = resolver_name(&service_statement);
    assert!(
        cli_resolver.is_ok() && service_resolver.is_ok(),
        "REQ-1 missing declaration-backed process exits: cli={cli_resolver:?}; service={service_resolver:?}"
    );
    assert_eq!(
        cli_resolver.unwrap(),
        service_resolver.unwrap(),
        "REQ-1 process exits do not use the same declaration resolver"
    );
    declaration_source().expect("REQ-1 single declaration is missing");
}

#[test]
fn qcg_2_duplicate_exit_and_error_are_rejected() {
    let rows = expected_rows();
    validate_rows(&rows).expect("released rows must be internally valid");

    let mut duplicate_exit = rows.clone();
    duplicate_exit[4].exit = duplicate_exit[3].exit;
    assert_eq!(
        validate_rows(&duplicate_exit),
        Err("duplicate exit 3".to_owned())
    );

    let mut duplicate_error = rows.clone();
    duplicate_error[4].errors = vec!["rem_job_failed"];
    assert_eq!(
        validate_rows(&duplicate_error),
        Err("duplicate error rem_job_failed".to_owned())
    );

    let (path, source) = declaration_source().expect("QCG-2 product declaration is missing");
    for value in rows
        .iter()
        .flat_map(|row| std::iter::once(row.name).chain(row.errors.iter().copied()))
    {
        assert_eq!(
            source.matches(&format!("\"{value}\"")).count(),
            1,
            "QCG-2 duplicate or missing declaration member `{value}` in {}",
            path.display()
        );
    }
}

#[test]
fn qcg_6_all_raisable_rem_codes_are_mapped() {
    let expected = BTreeMap::from([
        ("profile_error", 8),
        ("rem_job_failed", 3),
        ("rem_job_not_found", 9),
        ("rem_timeout", 4),
        ("rem_trace_error", 8),
        ("sidecar_client_error", 7),
        ("sidecar_discovery_error", 7),
        ("sidecar_discovery_invalid", 7),
        ("sidecar_not_running", 7),
        ("sidecar_response_error", 7),
        ("sidecar_response_invalid", 6),
        ("sidecar_response_truncated", 6),
        ("sidecar_unauthorized", 7),
    ]);
    let mut mismatches = Vec::new();
    for (code, expected_exit) in expected {
        let (observed_exit, observed_code) = observe_rem_error(code);
        if observed_code != code || observed_exit != expected_exit {
            mismatches.push(format!(
                "{code}: expected code={code} exit={expected_exit}; observed code={observed_code} exit={observed_exit}"
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "REQ-6 missing or wrong REM mappings:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn qcg_7_exit_one_is_unclassified_only() {
    let (known_exit, known_code) = observe_rem_error("rem_timeout");
    let (unknown_exit, unknown_code) = observe_rem_error("future_rem_error_019f8da3");
    assert!(
        known_code == "rem_timeout"
            && known_exit == 4
            && unknown_code == "future_rem_error_019f8da3"
            && unknown_exit == 1,
        "REQ-7/8 expected mapped rem_timeout exit=4 and absent future_rem_error_019f8da3 exit=1; observed rem_timeout code={known_code} exit={known_exit}, future code={unknown_code} exit={unknown_exit}"
    );

    let rows = expected_rows();
    let exit_one = rows.iter().filter(|row| row.exit == 1).collect::<Vec<_>>();
    assert_eq!(exit_one.len(), 1);
    assert_eq!(exit_one[0].name, "unclassified failure");
    assert!(exit_one[0].errors.is_empty());
    assert!(
        rows.iter()
            .filter(|row| row.name != "unclassified failure")
            .all(|row| row.exit != 1 && !row.errors.is_empty() || row.name == "success")
    );
}

fn record_json_outcome(
    repetition: usize,
    label: &str,
    output: &Output,
    expected_exit: i32,
    expected_key: &str,
    expected_value: &str,
    mismatches: &mut Vec<String>,
) {
    let value = last_json_line(output);
    let observed_exit = output.status.code();
    let observed_value = value[expected_key].as_str();
    if observed_exit != Some(expected_exit) || observed_value != Some(expected_value) {
        mismatches.push(format!(
            "repetition {repetition} {label}: expected exit={expected_exit} {expected_key}={expected_value}; observed exit={observed_exit:?} value={value} stdout={} stderr={}",
            output_text(&output.stdout),
            output_text(&output.stderr),
        ));
    }
}

#[test]
fn qcg_5_section_3_state_outcomes_do_not_interchange_across_ten_repetitions() {
    let unfamiliar_state = "future state/β 019f8da3";
    let mut mismatches = Vec::new();

    for repetition in 1..=10 {
        let done = run_status(
            vec![FixtureResponse::Json(200, rem_job("done", None))],
            false,
            true,
        );
        record_json_outcome(
            repetition,
            "done",
            &done,
            0,
            "state",
            "done",
            &mut mismatches,
        );

        let failed = run_status(
            vec![FixtureResponse::Json(
                200,
                rem_job("failed", Some("sidecar_failure_019f8da3")),
            )],
            false,
            true,
        );
        record_json_outcome(
            repetition,
            "failed",
            &failed,
            3,
            "error",
            "rem_job_failed",
            &mut mismatches,
        );

        let timeout = run_status(
            vec![
                FixtureResponse::Json(200, rem_job("running", None)),
                FixtureResponse::Json(200, rem_job("running", None)),
                FixtureResponse::Json(200, rem_job("running", None)),
            ],
            true,
            true,
        );
        record_json_outcome(
            repetition,
            "timeout",
            &timeout,
            4,
            "error",
            "rem_timeout",
            &mut mismatches,
        );

        let unfamiliar = run_status(
            vec![FixtureResponse::Json(200, rem_job(unfamiliar_state, None))],
            false,
            true,
        );
        record_json_outcome(
            repetition,
            "unfamiliar",
            &unfamiliar,
            5,
            "error",
            "rem_state_unrecognised",
            &mut mismatches,
        );

        let invalid = run_status(vec![FixtureResponse::Raw("not-json")], false, true);
        record_json_outcome(
            repetition,
            "invalid",
            &invalid,
            6,
            "error",
            "sidecar_response_invalid",
            &mut mismatches,
        );
    }

    assert!(
        mismatches.is_empty(),
        "QCG-5 local state outcomes interchanged:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn qcg_8_waiting_and_nonwaiting_known_states_succeed() {
    let queued = run_status(
        vec![FixtureResponse::Json(200, rem_job("queued", None))],
        false,
        true,
    );
    assert_eq!(queued.status.code(), Some(0));
    assert_eq!(last_json_line(&queued)["state"], "queued");

    let waited = run_status(
        vec![
            FixtureResponse::Json(200, rem_job("queued", None)),
            FixtureResponse::Json(200, rem_job("running", None)),
            FixtureResponse::Json(200, rem_job("done", None)),
        ],
        true,
        true,
    );
    assert_eq!(
        waited.status.code(),
        Some(0),
        "stdout={} stderr={}",
        output_text(&waited.stdout),
        output_text(&waited.stderr)
    );
    assert_eq!(last_json_line(&waited)["state"], "done");
}

#[cfg(unix)]
#[test]
fn qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture() {
    let state = "future state/β 019f8da3";
    let direct_nonwait = run_status(
        vec![FixtureResponse::Json(200, rem_job(state, None))],
        false,
        false,
    );
    let direct_wait = run_status(
        vec![FixtureResponse::Json(200, rem_job(state, None))],
        true,
        false,
    );
    let machine = run_status(
        vec![FixtureResponse::Json(200, rem_job(state, None))],
        true,
        true,
    );
    let merged = run_status_shell(
        rem_job(state, None),
        "exec \"$SNO_BIN\" station rem-status \"$JOB_ID\" --wait --timeout 1 2>&1",
    );
    let captured = run_status_shell(
        rem_job(state, None),
        "captured=$(\"$SNO_BIN\" station rem-status \"$JOB_ID\" --wait --timeout 1); status=$?; printf '%s\\n%s\\n' \"$status\" \"$captured\"",
    );

    assert_eq!(captured.status.code(), Some(0));
    let captured_text = output_text(&captured.stdout);
    let mut captured_lines = captured_text.lines();
    assert_eq!(
        captured_lines.next(),
        Some("5"),
        "command substitution observed the wrong sno exit: {captured_text}"
    );
    assert!(
        captured_lines
            .collect::<Vec<_>>()
            .join("\n")
            .contains(state),
        "command substitution lost raw state: {captured_text}"
    );

    let expected_sentence = "sidecar reported a state this build does not know";
    for (mode, output) in [("non-waiting", &direct_nonwait), ("waiting", &direct_wait)] {
        assert_eq!(
            output.status.code(),
            Some(5),
            "{mode}: stdout={} stderr={}",
            output_text(&output.stdout),
            output_text(&output.stderr)
        );
        let stdout = output_text(&output.stdout);
        let stderr = output_text(&output.stderr);
        assert!(
            stdout.contains(state),
            "{mode}: raw state missing: {stdout}"
        );
        for expected in [SECTION_3_JOB_ID, state, expected_sentence] {
            assert!(
                stderr.contains(expected),
                "{mode}: missing `{expected}` in {stderr}"
            );
        }
    }

    let machine_json = last_json_line(&machine);
    assert_eq!(machine.status.code(), Some(5));
    assert_eq!(machine_json["error"], "rem_state_unrecognised");
    assert_ne!(machine_json["error"], "sidecar_response_invalid");

    assert_eq!(merged.status.code(), Some(5));
    let merged_text = output_text(&merged.stdout);
    let state_position = merged_text.find(state).expect("state in merged output");
    let error_position = merged_text.find("error:").expect("error in merged output");
    assert!(
        state_position < error_position,
        "state must precede error: {merged_text}"
    );
}

#[test]
fn qcg_10_invalid_responses_are_distinct_from_unfamiliar_states() {
    for wait in [false, true] {
        for (label, response) in [
            ("invalid JSON", FixtureResponse::Raw("not-json")),
            (
                "missing state",
                FixtureResponse::Json(
                    200,
                    json!({
                        "type": "noop",
                        "scope": "persona:test-rem-state-019f8da3",
                        "started_at": "2026-08-09T20:00:00Z",
                        "finished_at": null,
                        "stats": null,
                        "error": null,
                        "correlation_id": null
                    }),
                ),
            ),
            ("empty state", FixtureResponse::Json(200, rem_job("", None))),
        ] {
            let output = run_status(vec![response], wait, true);
            let value = last_json_line(&output);
            assert_eq!(
                output.status.code(),
                Some(6),
                "{label} wait={wait}: stdout={} stderr={}",
                output_text(&output.stdout),
                output_text(&output.stderr)
            );
            assert_eq!(
                value["error"], "sidecar_response_invalid",
                "{label} wait={wait}: {value}"
            );
        }
    }

    let unfamiliar = run_status(
        vec![FixtureResponse::Json(
            200,
            rem_job("future state/β 019f8da3", None),
        )],
        false,
        true,
    );
    let value = last_json_line(&unfamiliar);
    assert_eq!(unfamiliar.status.code(), Some(5));
    assert_eq!(value["error"], "rem_state_unrecognised");
    assert_ne!(value["error"], "sidecar_response_invalid");
}

#[test]
fn qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel() {
    let sentinel = "sidecar_owned_failure_β_019f8da3";
    let provided = run_status(
        vec![FixtureResponse::Json(
            200,
            rem_job("failed", Some(sentinel)),
        )],
        false,
        true,
    );
    let omitted = run_status(
        vec![FixtureResponse::Json(200, rem_job_without_error("failed"))],
        false,
        true,
    );

    for (label, output) in [("provided", &provided), ("omitted", &omitted)] {
        let value = last_json_line(output);
        assert_eq!(output.status.code(), Some(3), "{label}: {value}");
        assert_eq!(value["error"], "rem_job_failed", "{label}: {value}");
        assert!(
            value["message"]
                .as_str()
                .is_some_and(|message| message.contains(SECTION_3_JOB_ID)),
            "{label}: missing job id in {value}"
        );
    }
    assert!(
        last_json_line(&provided)["message"]
            .as_str()
            .is_some_and(|message| message.contains(sentinel)),
        "provided sidecar sentinel was not preserved"
    );
    assert!(
        last_json_line(&omitted)["message"]
            .as_str()
            .is_some_and(|message| !message.contains(sentinel)),
        "omitted sidecar sentinel appeared in absent-field message"
    );
}
