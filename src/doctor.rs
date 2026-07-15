use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::cli::print_json;
use crate::error::CliError;
use crate::service::normalize_base_url;
use crate::state::{ConsentValue, Identity, SnoPaths, is_valid_identity};

#[derive(Serialize)]
struct DoctorReport {
    identity: DoctorCheck,
    buffer: DoctorCheck,
    consent: DoctorCheck,
    last_ship: DoctorCheck,
    lockfile: DoctorCheck,
}

#[derive(Serialize)]
struct DoctorCheck {
    name: &'static str,
    status: CheckStatus,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

pub fn run(json_enabled: bool) -> Result<i32, CliError> {
    let paths = SnoPaths::from_environment()?;
    let (buffer, shipped_count) = check_buffer(&paths.buffer_path);
    let report = DoctorReport {
        identity: check_identity(&paths.identity_path),
        buffer,
        consent: check_consent(&paths.consent_path),
        last_ship: check_last_ship(shipped_count),
        lockfile: check_lockfile(&paths.identity_lock_path),
    };
    let has_issue = report.identity.status != CheckStatus::Ok
        || report.buffer.status != CheckStatus::Ok
        || report.consent.status != CheckStatus::Ok
        || report.last_ship.status != CheckStatus::Ok
        || report.lockfile.status != CheckStatus::Ok;
    if json_enabled {
        print_json(&serde_json::to_value(&report)?)?;
    } else {
        for check in [
            &report.identity,
            &report.buffer,
            &report.consent,
            &report.last_ship,
            &report.lockfile,
        ] {
            let badge = match check.status {
                CheckStatus::Ok => "[ok]",
                CheckStatus::Warn => "[warn]",
                CheckStatus::Fail => "[fail]",
            };
            println!("{badge} {}", check.detail);
        }
    }
    Ok(if has_issue { 1 } else { 0 })
}

fn check_identity(path: &Path) -> DoctorCheck {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return DoctorCheck {
                name: "identity",
                status: CheckStatus::Warn,
                detail: "identity not bootstrapped - first SDK use or `sno account machine register` will create it automatically".to_owned(),
                path: Some(path.display().to_string()),
            };
        }
        Err(error) => return failed_path("identity", path, error.to_string()),
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return failed_path("identity", path, error.to_string()),
    };
    let parsed = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(parsed) => parsed,
        Err(_) => {
            return DoctorCheck {
                name: "identity",
                status: CheckStatus::Fail,
                detail: format!("identity is not readable JSON at {}", path.display()),
                path: Some(path.display().to_string()),
            };
        }
    };
    let identity = match serde_json::from_value::<Identity>(parsed) {
        Ok(identity) => identity,
        Err(_) => {
            return DoctorCheck {
                name: "identity",
                status: CheckStatus::Fail,
                detail: format!("identity is malformed at {}", path.display()),
                path: Some(path.display().to_string()),
            };
        }
    };
    if !is_valid_identity(&identity) {
        return DoctorCheck {
            name: "identity",
            status: CheckStatus::Fail,
            detail: format!("identity is malformed at {}", path.display()),
            path: Some(path.display().to_string()),
        };
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions().mode() & 0o777;
        if permissions & 0o077 != 0 {
            return DoctorCheck {
                name: "identity",
                status: CheckStatus::Warn,
                detail: format!(
                    "identity present but {} mode {:o} is permissive",
                    path.display(),
                    permissions
                ),
                path: Some(path.display().to_string()),
            };
        }
    }
    DoctorCheck {
        name: "identity",
        status: CheckStatus::Ok,
        detail: "identity present (anonymous machine)".to_owned(),
        path: Some(path.display().to_string()),
    }
}

fn check_buffer(path: &Path) -> (DoctorCheck, i64) {
    if !path.exists() {
        return (
            DoctorCheck {
                name: "buffer",
                status: CheckStatus::Warn,
                detail: format!("buffer not initialized at {}", path.display()),
                path: Some(path.display().to_string()),
            },
            0,
        );
    }
    let result = (|| -> Result<(i64, i64, i64), rusqlite::Error> {
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let pending = connection.query_row(
            "SELECT COUNT(*) FROM events WHERE shipped = 0 AND terminal = 0",
            [],
            |row| row.get(0),
        )?;
        let total = connection.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        let shipped =
            connection.query_row("SELECT COUNT(*) FROM events WHERE shipped = 1", [], |row| {
                row.get(0)
            })?;
        Ok((pending, total, shipped))
    })();
    match result {
        Ok((pending, total, shipped)) => {
            let wal = if sidecar_path(path, "-wal").exists() {
                "wal sidecar present"
            } else {
                "wal sidecar absent"
            };
            (
                DoctorCheck {
                    name: "buffer",
                    status: CheckStatus::Ok,
                    detail: format!("buffer reachable ({pending}/{total} pending; {wal})"),
                    path: Some(path.display().to_string()),
                },
                shipped,
            )
        }
        Err(error) => (
            DoctorCheck {
                name: "buffer",
                status: CheckStatus::Fail,
                detail: format!("buffer not reachable at {}: {error}", path.display()),
                path: Some(path.display().to_string()),
            },
            0,
        ),
    }
}

fn check_consent(path: &Path) -> DoctorCheck {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return DoctorCheck {
                name: "consent",
                status: CheckStatus::Ok,
                detail: "consent metadata-only (default)".to_owned(),
                path: Some(path.display().to_string()),
            };
        }
        Err(error) => return failed_path("consent", path, error.to_string()),
    };
    let parsed = serde_json::from_slice::<serde_json::Value>(&bytes).ok();
    let consent = parsed.as_ref().and_then(|value| {
        (value.get("version").and_then(serde_json::Value::as_u64) == Some(1))
            .then(|| value.get("value")?.as_str())
            .flatten()
            .and_then(|value| ConsentValue::parse_cli(value).ok())
    });
    match consent {
        Some(consent) => DoctorCheck {
            name: "consent",
            status: CheckStatus::Ok,
            detail: format!("consent {consent}"),
            path: Some(path.display().to_string()),
        },
        None => DoctorCheck {
            name: "consent",
            status: CheckStatus::Fail,
            detail: format!("consent is malformed at {}", path.display()),
            path: Some(path.display().to_string()),
        },
    }
}

fn check_last_ship(shipped_count: i64) -> DoctorCheck {
    let configured =
        env::var("SNO_OBSERVE_BASE_URL").unwrap_or_else(|_| "https://www.sno.ai".to_owned());
    let base_url = normalize_base_url(&configured)
        .map(|url| url.as_str().trim_end_matches('/').to_owned())
        .unwrap_or_else(|_| configured.trim_end_matches('/').to_owned());
    let suffix = if shipped_count > 0 {
        "shipped events present"
    } else {
        "never"
    };
    DoctorCheck {
        name: "last_ship",
        status: CheckStatus::Ok,
        detail: format!("last successful POST to {base_url}: {suffix}"),
        path: None,
    }
}

fn check_lockfile(path: &Path) -> DoctorCheck {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return DoctorCheck {
                name: "lockfile",
                status: CheckStatus::Ok,
                detail: "no identity lockfile present".to_owned(),
                path: Some(path.display().to_string()),
            };
        }
        Err(_) => String::new(),
    };
    let pid = contents
        .split(|character: char| !character.is_ascii_digit())
        .find(|part| !part.is_empty())
        .and_then(|part| part.parse::<u32>().ok())
        .filter(|pid| *pid > 0);
    let Some(pid) = pid else {
        return DoctorCheck {
            name: "lockfile",
            status: CheckStatus::Warn,
            detail: format!(
                "identity lockfile present at {} without a PID",
                path.display()
            ),
            path: Some(path.display().to_string()),
        };
    };
    if pid_is_running(pid) {
        return DoctorCheck {
            name: "lockfile",
            status: CheckStatus::Ok,
            detail: format!("identity lockfile held by pid {pid}"),
            path: Some(path.display().to_string()),
        };
    }
    DoctorCheck {
        name: "lockfile",
        status: CheckStatus::Warn,
        detail: format!(
            "stale lockfile detected at {} - safe to remove if no `sno` process is running",
            path.display()
        ),
        path: Some(path.display().to_string()),
    }
}

#[cfg(unix)]
fn pid_is_running(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(not(unix))]
fn pid_is_running(_pid: u32) -> bool {
    true
}

fn sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", path.display()))
}

fn failed_path(name: &'static str, path: &Path, error: String) -> DoctorCheck {
    DoctorCheck {
        name,
        status: CheckStatus::Fail,
        detail: format!("{name} not reachable at {}: {error}", path.display()),
        path: Some(path.display().to_string()),
    }
}
