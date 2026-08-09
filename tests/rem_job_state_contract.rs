#[allow(dead_code)]
#[path = "support/sno_service_server.rs"]
mod sno_service_server;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use sno_service_server::{ServiceResponse, SnoServiceServer};
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
