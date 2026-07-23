use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use fs2::FileExt;
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
const REM_TRACE_ENV: &str = "SNO_REM_TRACE";
const OPENCLAW_STATE_DIR_ENV: &str = "OPENCLAW_STATE_DIR";

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
    path: Option<PathBuf>,
}

impl RemTrace {
    fn from_environment() -> Result<Self, CliError> {
        let correlation_id = env::var(REM_CORRELATION_ID_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("rem-corr-{}", Uuid::now_v7()));
        let path = if trace_enabled() {
            Some(rem_trace_path()?)
        } else {
            None
        };
        let trace = Self {
            correlation_id,
            path,
        };
        if let Some(path) = &trace.path {
            trace.append(
                "trace_opened",
                json!({ "trace_file": path.display().to_string() }),
            )?;
        }
        Ok(trace)
    }

    fn append(&self, event: &str, fields: Value) -> Result<(), CliError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
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
            .open(path)
            .map_err(|error| {
                CliError::runtime(
                    "rem_trace_error",
                    format!("failed to open {}: {error}", path.display()),
                )
            })?;
        file.lock_exclusive().map_err(|error| {
            CliError::runtime(
                "rem_trace_error",
                format!("failed to lock {}: {error}", path.display()),
            )
        })?;
        let append_result = writeln!(file, "{}", Value::Object(record))
            .and_then(|()| file.sync_all())
            .map_err(|error| {
                CliError::runtime(
                    "rem_trace_error",
                    format!("failed to write and sync {}: {error}", path.display()),
                )
            });
        let unlock_result = FileExt::unlock(&file).map_err(|error| {
            CliError::runtime(
                "rem_trace_error",
                format!("failed to unlock {}: {error}", path.display()),
            )
        });
        append_result?;
        unlock_result
    }
}

fn trace_enabled() -> bool {
    !env::var(REM_TRACE_ENV)
        .ok()
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "0" | "false" | "off"))
}

fn rem_trace_path() -> Result<PathBuf, CliError> {
    let state_dir = if let Some(path) = env::var_os(OPENCLAW_STATE_DIR_ENV) {
        PathBuf::from(path)
    } else {
        env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .map(|home| home.join(".openclaw"))
            .ok_or_else(|| CliError::runtime("rem_trace_error", "home directory is unavailable"))?
    };
    Ok(state_dir.join("mem-claw").join("rem-trace.jsonl"))
}

pub(crate) fn run_start(rem_type: &str, scope: &str, json_enabled: bool) -> Result<i32, CliError> {
    let profile_dir = profile_dir_from_environment()?;
    let trace = RemTrace::from_environment()?;
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
    let trace = RemTrace::from_environment()?;
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
    let response = match client()?
        .post(format!("{endpoint}/rem/run"))
        .header("X-Sidecar-Token", discovery.token)
        .header(REM_CORRELATION_HEADER, &trace.correlation_id)
        .json(&json!({ "type": rem_type, "scope": scope }))
        .send()
    {
        Ok(response) => response,
        Err(error) => {
            trace_request_failure(
                trace,
                "POST",
                "/rem/run",
                1,
                "refused_or_reset",
                &error.to_string(),
            )?;
            return Err(CliError::runtime(
                "sidecar_not_running",
                "sidecar not running",
            ));
        }
    };
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
    let response = match client()?
        .get(format!("{endpoint}{path}"))
        .header("X-Sidecar-Token", discovery.token)
        .header(REM_CORRELATION_HEADER, &trace.correlation_id)
        .send()
    {
        Ok(response) => response,
        Err(error) => {
            trace_request_failure(
                trace,
                "GET",
                &path,
                poll_index,
                "refused_or_reset",
                &error.to_string(),
            )?;
            return Err(CliError::runtime(
                "sidecar_not_running",
                "sidecar not running",
            ));
        }
    };
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

fn trace_request_failure(
    trace: &RemTrace,
    method: &str,
    path: &str,
    poll_index: usize,
    transient_class: &str,
    transport_error: &str,
) -> Result<(), CliError> {
    trace.append(
        "http_request_failed",
        json!({
            "method": method,
            "path": path,
            "poll_index": poll_index,
            "transient_class": transient_class,
            "transport_error": transport_error,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs2::FileExt;
    use std::sync::mpsc;

    #[test]
    fn request_failure_does_not_mask_trace_write_failure() {
        let trace_directory = tempfile::tempdir().expect("trace directory");
        let trace = RemTrace {
            correlation_id: "corr-trace-failure".to_owned(),
            path: Some(trace_directory.path().to_path_buf()),
        };

        let error = trace_request_failure(
            &trace,
            "GET",
            "/rem/jobs/job-trace-failure",
            1,
            "refused_or_reset",
            "connection refused",
        )
        .expect_err("trace write failure must win");

        assert_eq!(error.code, "rem_trace_error");
    }

    #[test]
    fn trace_append_waits_for_the_cross_process_file_lock() {
        let trace_directory = tempfile::tempdir().expect("trace directory");
        let trace_path = trace_directory.path().join("rem-trace.jsonl");
        let lock_holder = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&trace_path)
            .expect("trace lock file");
        lock_holder.lock_exclusive().expect("hold trace lock");
        let trace = RemTrace {
            correlation_id: "corr-concurrent-append".to_owned(),
            path: Some(trace_path),
        };
        let (sender, receiver) = mpsc::channel();
        let writer = thread::spawn(move || {
            sender
                .send(trace.append("command_received", json!({"command":"rem-start"})))
                .expect("send append result");
        });

        assert!(
            receiver.recv_timeout(Duration::from_millis(50)).is_err(),
            "append ignored the held file lock"
        );
        FileExt::unlock(&lock_holder).expect("release trace lock");
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("append completed after lock release")
            .expect("append succeeded");
        writer.join().expect("trace writer");
    }
}
