use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::print_json;
use crate::error::CliError;

const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

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
}

pub(crate) fn run_start(rem_type: &str, scope: &str, json_enabled: bool) -> Result<i32, CliError> {
    let profile_dir = profile_dir_from_environment()?;
    let response = send_start_at(&profile_dir, rem_type, scope)?;
    if json_enabled {
        print_json(&json!({ "job_id": response.job_id }))?;
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
    let timeout = timeout_seconds
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_WAIT_TIMEOUT);
    let job = poll_status_at(&profile_dir, job_id, wait, timeout, POLL_INTERVAL)?;
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

fn read_discovery(profile_dir: &Path) -> Result<SidecarDiscovery, CliError> {
    let path = discovery_path(profile_dir);
    let bytes = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliError::runtime("sidecar_not_running", "sidecar not running")
        } else {
            CliError::runtime(
                "sidecar_discovery_error",
                format!("failed to read {}: {error}", path.display()),
            )
        }
    })?;
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
    Ok(discovery)
}

fn client() -> Result<Client, CliError> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| CliError::runtime("sidecar_client_error", error.to_string()))
}

pub(crate) fn send_start_at(
    profile_dir: &Path,
    rem_type: &str,
    scope: &str,
) -> Result<RemStartResponse, CliError> {
    let discovery = read_discovery(profile_dir)?;
    let response = client()?
        .post(format!("http://127.0.0.1:{}/rem/run", discovery.port))
        .header("X-Sidecar-Token", discovery.token)
        .json(&json!({ "type": rem_type, "scope": scope }))
        .send()
        .map_err(|_| CliError::runtime("sidecar_not_running", "sidecar not running"))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .map_err(|error| CliError::runtime("sidecar_response_error", error.to_string()))?;
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

pub(crate) fn poll_status_at(
    profile_dir: &Path,
    job_id: &str,
    wait: bool,
    timeout: Duration,
    interval: Duration,
) -> Result<RemJob, CliError> {
    let started = Instant::now();
    loop {
        match fetch_status_at(profile_dir, job_id) {
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
            Err(error) if wait && is_transient_wait_error(&error) => {}
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

fn fetch_status_at(profile_dir: &Path, job_id: &str) -> Result<RemJob, CliError> {
    let discovery = read_discovery(profile_dir)?;
    let response = client()?
        .get(format!(
            "http://127.0.0.1:{}/rem/jobs/{job_id}",
            discovery.port
        ))
        .header("X-Sidecar-Token", discovery.token)
        .send()
        .map_err(|_| CliError::runtime("sidecar_not_running", "sidecar not running"))?;
    let status = response.status();
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
        .map_err(|_| CliError::runtime("sidecar_not_running", "sidecar not running"))?;
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
        "sidecar_not_running" | "sidecar_unauthorized"
    )
}

fn status_error_code(status: StatusCode) -> String {
    format!("sidecar_http_{}", status.as_u16())
}
