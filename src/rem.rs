use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use super::print_json;
use crate::error::CliError;

const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const REM_CORRELATION_HEADER: &str = "X-Rem-Correlation-Id";
const REM_CORRELATION_ID_ENV: &str = "SNO_REM_CORRELATION_ID";
const REM_TRACE_FILE_ENV: &str = "SNO_REM_TRACE_FILE";

#[derive(Debug, Deserialize)]
struct SidecarDiscovery {
    port: u16,
    token: String,
}

#[derive(Debug, Deserialize)]
struct RemErrorResponse {
    job_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RemStartResponse {
    pub job_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RemJob {
    pub state: String,
    #[serde(rename = "type")]
    pub rem_type: String,
    pub scope: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub stats: Value,
    pub error: Option<String>,
    pub correlation_id: Option<String>,
}

struct RemTrace {
    correlation_id: String,
    path: PathBuf,
}

impl RemTrace {
    fn from_environment(profile_dir: &Path) -> Result<Self, CliError> {
        let correlation_id = env::var(REM_CORRELATION_ID_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("rem-corr-{}", Uuid::now_v7()));
        let path = env::var_os(REM_TRACE_FILE_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| profile_dir.join("station").join("rem-trace.jsonl"));
        let trace = Self {
            correlation_id,
            path,
        };
        trace.append(
            "trace_opened",
            json!({ "trace_file": trace.path.display().to_string() }),
        )?;
        Ok(trace)
    }

    fn append(&self, event: &str, fields: Value) -> Result<(), CliError> {
        let parent = self.path.parent().ok_or_else(|| {
            CliError::runtime("rem_trace_error", "REM trace path has no parent directory")
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            CliError::runtime(
                "rem_trace_error",
                format!("failed to create {}: {error}", parent.display()),
            )
        })?;
        let mut record = serde_json::Map::new();
        record.insert("timestamp".to_owned(), json!(Utc::now().to_rfc3339()));
        record.insert("component".to_owned(), json!("sno_cli"));
        record.insert("event".to_owned(), json!(event));
        record.insert("correlation_id".to_owned(), json!(self.correlation_id));
        if let Value::Object(fields) = fields {
            record.extend(fields);
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| {
                CliError::runtime(
                    "rem_trace_error",
                    format!("failed to open {}: {error}", self.path.display()),
                )
            })?;
        writeln!(file, "{}", Value::Object(record)).map_err(|error| {
            CliError::runtime(
                "rem_trace_error",
                format!("failed to write {}: {error}", self.path.display()),
            )
        })?;
        file.sync_all().map_err(|error| {
            CliError::runtime(
                "rem_trace_error",
                format!("failed to sync {}: {error}", self.path.display()),
            )
        })
    }
}

pub(crate) fn run_start(rem_type: &str, scope: &str, json_enabled: bool) -> Result<i32, CliError> {
    let profile_dir = profile_dir_from_environment()?;
    let trace = RemTrace::from_environment(&profile_dir)?;
    trace.append(
        "command_received",
        json!({
            "command": "rem-start",
            "type": rem_type,
            "scope": scope,
            "json": json_enabled,
        }),
    )?;
    let response = send_start_at(&profile_dir, rem_type, scope, &trace)?;
    trace.append(
        "command_emitted",
        json!({ "command": "rem-start", "job_id": response.job_id }),
    )?;
    if json_enabled {
        print_json(&json!({
            "job_id": response.job_id,
            "correlation_id": trace.correlation_id,
        }))?;
    } else {
        println!("{}", response.job_id);
    }
    Ok(0)
}

pub(crate) fn run_status(
    job_id: &str,
    wait: bool,
    timeout_seconds: Option<u64>,
    json_enabled: bool,
) -> Result<i32, CliError> {
    let profile_dir = profile_dir_from_environment()?;
    let trace = RemTrace::from_environment(&profile_dir)?;
    let timeout = timeout_seconds
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_WAIT_TIMEOUT);
    trace.append(
        "command_received",
        json!({
            "command": "rem-status",
            "job_id": job_id,
            "wait": wait,
            "timeout_seconds": timeout.as_secs(),
            "json": json_enabled,
        }),
    )?;
    let job = poll_status_at(&profile_dir, job_id, wait, timeout, POLL_INTERVAL, &trace)?;
    trace.append(
        "command_emitted",
        json!({ "command": "rem-status", "job_id": job_id, "state": job.state }),
    )?;
    if json_enabled {
        print_json(&serde_json::to_value(&job)?)?;
    } else {
        println!("{}", job.state);
    }
    Ok(0)
}

fn profile_dir_from_environment() -> Result<PathBuf, CliError> {
    if let Some(profile_dir) = env::var_os("SNO_PROFILE_DIR").or_else(|| env::var_os("SNO_HOME")) {
        return Ok(PathBuf::from(profile_dir));
    }
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|home| home.join(".sno"))
        .ok_or_else(|| CliError::runtime("profile_error", "home directory is unavailable"))
}

fn discovery_path(profile_dir: &Path) -> PathBuf {
    profile_dir.join("station").join("sidecar.json")
}

fn read_discovery(
    profile_dir: &Path,
    trace: &RemTrace,
    poll_index: usize,
) -> Result<SidecarDiscovery, CliError> {
    let path = discovery_path(profile_dir);
    trace.append(
        "discovery_read",
        json!({ "discovery_file": path.display().to_string(), "poll_index": poll_index }),
    )?;
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let transient_class = if error.kind() == std::io::ErrorKind::NotFound {
                "missing_discovery"
            } else {
                "discovery_io"
            };
            trace.append(
                "discovery_failed",
                json!({
                    "discovery_file": path.display().to_string(),
                    "poll_index": poll_index,
                    "transient_class": transient_class,
                }),
            )?;
            return Err(if error.kind() == std::io::ErrorKind::NotFound {
                CliError::runtime("sidecar_not_running", "sidecar not running")
            } else {
                CliError::runtime(
                    "sidecar_discovery_error",
                    format!("failed to read {}: {error}", path.display()),
                )
            });
        }
    };
    let discovery = serde_json::from_slice::<SidecarDiscovery>(&bytes).map_err(|_| {
        CliError::runtime(
            "sidecar_discovery_invalid",
            format!("sidecar discovery is malformed at {}", path.display()),
        )
    })?;
    if discovery.token.is_empty() {
        return Err(CliError::runtime(
            "sidecar_discovery_invalid",
            format!("sidecar discovery is malformed at {}", path.display()),
        ));
    }
    trace.append(
        "discovery_resolved",
        json!({
            "discovery_file": path.display().to_string(),
            "poll_index": poll_index,
            "port": discovery.port,
            "endpoint": format!("http://127.0.0.1:{}", discovery.port),
        }),
    )?;
    Ok(discovery)
}

fn client() -> Result<Client, CliError> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| CliError::runtime("sidecar_client_error", error.to_string()))
}

fn send_start_at(
    profile_dir: &Path,
    rem_type: &str,
    scope: &str,
    trace: &RemTrace,
) -> Result<RemStartResponse, CliError> {
    let discovery = read_discovery(profile_dir, trace, 1)?;
    let endpoint = format!("http://127.0.0.1:{}", discovery.port);
    trace.append(
        "http_request_sent",
        json!({
            "method": "POST",
            "path": "/rem/run",
            "endpoint": endpoint,
            "body_shape": ["type", "scope"],
            "poll_index": 1,
        }),
    )?;
    let response = client()?
        .post(format!("{endpoint}/rem/run"))
        .header("X-Sidecar-Token", discovery.token)
        .header(REM_CORRELATION_HEADER, &trace.correlation_id)
        .json(&json!({ "type": rem_type, "scope": scope }))
        .send()
        .map_err(|_| {
            let _ = trace.append(
                "http_request_failed",
                json!({
                    "method": "POST",
                    "path": "/rem/run",
                    "poll_index": 1,
                    "transient_class": "refused_or_reset",
                }),
            );
            CliError::runtime("sidecar_not_running", "sidecar not running")
        })?;
    let status = response.status();
    let bytes = response
        .bytes()
        .map_err(|error| CliError::runtime("sidecar_response_error", error.to_string()))?;
    trace.append(
        "http_response_received",
        json!({
            "method": "POST",
            "path": "/rem/run",
            "poll_index": 1,
            "status": status.as_u16(),
        }),
    )?;
    if status.is_success() {
        return serde_json::from_slice(&bytes).map_err(|_| {
            CliError::runtime(
                "sidecar_response_invalid",
                "sidecar returned an invalid REM start response",
            )
        });
    }
    let error = serde_json::from_slice::<RemErrorResponse>(&bytes).unwrap_or(RemErrorResponse {
        job_id: None,
        error: None,
    });
    let code = error.error.unwrap_or_else(|| status_error_code(status));
    let message = match error.job_id {
        Some(job_id) => format!("REM job {job_id} failed: {code}"),
        None => format!("REM start failed: {code}"),
    };
    Err(CliError::runtime(code, message))
}

fn poll_status_at(
    profile_dir: &Path,
    job_id: &str,
    wait: bool,
    timeout: Duration,
    interval: Duration,
    trace: &RemTrace,
) -> Result<RemJob, CliError> {
    let started = Instant::now();
    let mut poll_index = 0;
    loop {
        poll_index += 1;
        match fetch_status_at(profile_dir, job_id, poll_index, trace) {
            Ok(job) => match job.state.as_str() {
                "done" => return Ok(job),
                "failed" => {
                    let reason = job.error.as_deref().unwrap_or("unknown");
                    return Err(CliError::runtime(
                        "rem_job_failed",
                        format!("REM job {job_id} failed / {reason}"),
                    ));
                }
                "queued" | "running" if wait => {}
                "queued" | "running" => return Ok(job),
                _ => {
                    return Err(CliError::runtime(
                        "sidecar_response_invalid",
                        "sidecar returned an invalid REM job state",
                    ));
                }
            },
            Err(error) if wait && is_transient_wait_error(&error) => {
                trace.append(
                    "poll_transient",
                    json!({
                        "job_id": job_id,
                        "poll_index": poll_index,
                        "transient_class": transient_class(&error),
                    }),
                )?;
            }
            Err(error) => return Err(error),
        }
        if started.elapsed() >= timeout {
            return Err(CliError::runtime(
                "rem_timeout",
                format!("timed out waiting for REM job {job_id}"),
            ));
        }
        thread::sleep(interval);
    }
}

fn fetch_status_at(
    profile_dir: &Path,
    job_id: &str,
    poll_index: usize,
    trace: &RemTrace,
) -> Result<RemJob, CliError> {
    let discovery = read_discovery(profile_dir, trace, poll_index)?;
    let path = format!("/rem/jobs/{job_id}");
    let endpoint = format!("http://127.0.0.1:{}", discovery.port);
    trace.append(
        "http_request_sent",
        json!({
            "method": "GET",
            "path": path,
            "endpoint": endpoint,
            "body_shape": [],
            "poll_index": poll_index,
        }),
    )?;
    let response = client()?
        .get(format!("{endpoint}{path}"))
        .header("X-Sidecar-Token", discovery.token)
        .header(REM_CORRELATION_HEADER, &trace.correlation_id)
        .send()
        .map_err(|_| {
            let _ = trace.append(
                "http_request_failed",
                json!({
                    "method": "GET",
                    "path": path,
                    "poll_index": poll_index,
                    "transient_class": "refused_or_reset",
                }),
            );
            CliError::runtime("sidecar_not_running", "sidecar not running")
        })?;
    let status = response.status();
    trace.append(
        "http_response_received",
        json!({
            "method": "GET",
            "path": path,
            "poll_index": poll_index,
            "status": status.as_u16(),
        }),
    )?;
    if status == StatusCode::UNAUTHORIZED {
        return Err(CliError::runtime(
            "sidecar_unauthorized",
            "sidecar authentication failed",
        ));
    }
    if status == StatusCode::NOT_FOUND {
        return Err(CliError::runtime(
            "rem_job_not_found",
            format!("REM job not found: {job_id}"),
        ));
    }
    if !status.is_success() {
        return Err(CliError::runtime(
            status_error_code(status),
            format!(
                "sidecar status request failed with HTTP {}",
                status.as_u16()
            ),
        ));
    }
    let bytes = response
        .bytes()
        .map_err(|_| CliError::runtime("sidecar_response_truncated", "sidecar not running"))?;
    serde_json::from_slice::<RemJob>(&bytes).map_err(|_| {
        CliError::runtime(
            "sidecar_response_invalid",
            "sidecar returned an invalid REM status response",
        )
    })
}

fn is_transient_wait_error(error: &CliError) -> bool {
    matches!(
        error.code.as_str(),
        "sidecar_not_running" | "sidecar_unauthorized" | "sidecar_response_truncated"
    )
}

fn transient_class(error: &CliError) -> &'static str {
    match error.code.as_str() {
        "sidecar_unauthorized" => "unauthorized",
        "sidecar_response_truncated" => "truncated",
        "sidecar_not_running" => "refused_or_reset",
        _ => "unknown",
    }
}

fn status_error_code(status: StatusCode) -> String {
    format!("sidecar_http_{}", status.as_u16())
}
