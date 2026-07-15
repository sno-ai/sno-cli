use std::env;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{SecondsFormat, Utc};
use fs2::{FileExt, lock_contended_error};
use rand::RngCore;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::CliError;

const DEFAULT_CONSENT: ConsentValue = ConsentValue::MetadataOnly;
const SNO_OBSERVE_VERSION: &str = "0.1.0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsentValue {
    Off,
    MetadataOnly,
    Full,
}

impl ConsentValue {
    pub fn parse_cli(value: &str) -> Result<Self, CliError> {
        match value {
            "off" => Ok(Self::Off),
            "metadata-only" => Ok(Self::MetadataOnly),
            "full" => Ok(Self::Full),
            "metadata" => Err(CliError::usage(
                "invalid consent value: 'metadata' (did you mean 'metadata-only'?)",
            )),
            _ => Err(CliError::usage(
                "invalid consent value: expected one of off, metadata-only, full",
            )),
        }
    }
}

impl Display for ConsentValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Off => "off",
            Self::MetadataOnly => "metadata-only",
            Self::Full => "full",
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Identity {
    pub version: u8,
    pub user_cuid: String,
    pub machine_uuid: String,
    pub machine_secret: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_account_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConsentState {
    version: u8,
    value: ConsentValue,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PauseState {
    version: u8,
    prior: ConsentValue,
    paused_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConsentTransition {
    version: u8,
    transition_id: String,
    from: ConsentValue,
    to: ConsentValue,
    reason: String,
    prepared_at: String,
}

#[derive(Clone, Debug)]
pub struct SnoPaths {
    pub identity_path: PathBuf,
    pub identity_lock_path: PathBuf,
    pub buffer_path: PathBuf,
    pub consent_path: PathBuf,
    pub pause_path: PathBuf,
    pub consent_transition_path: PathBuf,
    pub consent_lock_path: PathBuf,
}

impl SnoPaths {
    pub fn from_environment() -> Result<Self, CliError> {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .ok_or_else(|| CliError::runtime("profile_error", "home directory is unavailable"))?;
        let profile_dir = env::var_os("SNO_PROFILE_DIR")
            .or_else(|| env::var_os("SNO_HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".sno"));
        let identity_path = env::var_os("SNO_IDENTITY_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| profile_dir.join("identity.json"));
        let identity_lock_path = identity_path
            .parent()
            .unwrap_or(&profile_dir)
            .join("identity.lock");
        let buffer_path = env::var_os("SNO_BUFFER_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| profile_dir.join("buffer.db"));
        let consent_path = env::var_os("SNO_CONSENT_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| profile_dir.join("state").join("consent.json"));
        let pause_path = profile_dir.join("state").join("consent-prior.json");
        let consent_transition_path = consent_path
            .parent()
            .unwrap_or(&profile_dir)
            .join("consent-transition.json");
        let consent_lock_path = consent_path
            .parent()
            .unwrap_or(&profile_dir)
            .join("consent.lock");
        Ok(Self {
            identity_path,
            identity_lock_path,
            buffer_path,
            consent_path,
            pause_path,
            consent_transition_path,
            consent_lock_path,
        })
    }
}

pub fn read_consent() -> Result<ConsentValue, CliError> {
    let paths = SnoPaths::from_environment()?;
    with_consent_lock(&paths, || read_consent_locked(&paths))
}

pub fn set_consent(next: ConsentValue, reason: &str) -> Result<(), CliError> {
    let paths = SnoPaths::from_environment()?;
    with_consent_lock(&paths, || {
        let current = read_consent_locked(&paths)?;
        if current == next {
            clear_pause(&paths)?;
            return Ok(());
        }
        apply_consent_transition(&paths, current, next, reason)?;
        clear_pause(&paths)
    })
}

pub fn pause_telemetry() -> Result<(ConsentValue, bool), CliError> {
    let paths = SnoPaths::from_environment()?;
    with_consent_lock(&paths, || {
        let current = read_consent_locked(&paths)?;
        if current == ConsentValue::Off {
            return Ok((current, true));
        }
        atomic_write_json(
            &paths.pause_path,
            &PauseState {
                version: 1,
                prior: current,
                paused_at: now_iso(),
            },
        )?;
        apply_consent_transition(&paths, current, ConsentValue::Off, "observe.pause")?;
        Ok((ConsentValue::Off, false))
    })
}

pub fn resume_telemetry() -> Result<ConsentValue, CliError> {
    let paths = SnoPaths::from_environment()?;
    with_consent_lock(&paths, || {
        let current = read_consent_locked(&paths)?;
        if current != ConsentValue::Off {
            clear_pause(&paths)?;
            return Ok(current);
        }
        let prior = match fs::read(&paths.pause_path) {
            Ok(bytes) => {
                let state = serde_json::from_slice::<PauseState>(&bytes).map_err(|_| {
                    CliError::runtime(
                        "invalid_pause_state",
                        format!("pause state is malformed at {}", paths.pause_path.display()),
                    )
                })?;
                if state.version != 1 || state.prior == ConsentValue::Off {
                    return Err(CliError::runtime(
                        "invalid_pause_state",
                        format!("pause state is malformed at {}", paths.pause_path.display()),
                    ));
                }
                state.prior
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CliError::runtime(
                    "telemetry_not_paused",
                    "telemetry is off by explicit consent; no paused setting can be resumed",
                ));
            }
            Err(error) => return Err(error.into()),
        };
        if current != prior {
            apply_consent_transition(&paths, current, prior, "observe.resume")?;
        }
        clear_pause(&paths)?;
        Ok(prior)
    })
}

pub fn bootstrap_identity() -> Result<Identity, CliError> {
    let paths = SnoPaths::from_environment()?;
    bootstrap_identity_at(&paths)
}

pub fn update_identity_account(
    expected: &Identity,
    user_account_id: &str,
) -> Result<Identity, CliError> {
    if !is_cuid2(user_account_id) {
        return Err(CliError::runtime(
            "invalid_account_id",
            "server returned an invalid account identifier",
        ));
    }
    let paths = SnoPaths::from_environment()?;
    update_identity_account_at(&paths, expected, user_account_id)
}

fn update_identity_account_at(
    paths: &SnoPaths,
    expected: &Identity,
    user_account_id: &str,
) -> Result<Identity, CliError> {
    with_identity_lock(paths, || {
        let mut current = read_valid_identity(&paths.identity_path)?.ok_or_else(|| {
            CliError::runtime(
                "claim_identity_changed",
                "local identity changed before claim could be saved",
            )
        })?;
        if current.user_cuid != expected.user_cuid || current.machine_uuid != expected.machine_uuid
        {
            return Err(CliError::runtime(
                "claim_identity_changed",
                "local identity changed before claim could be saved",
            ));
        }
        if let Some(account_id) = &current.user_account_id {
            if account_id == user_account_id {
                return Ok(current);
            }
            return Err(CliError::runtime(
                "claim_account_conflict",
                "machine is already claimed by a different account",
            ));
        }
        current.user_account_id = Some(user_account_id.to_owned());
        atomic_write_json(&paths.identity_path, &current)?;
        Ok(current)
    })
}

pub fn open_buffer(path: &Path) -> Result<Connection, CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = Connection::open(path)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS events (
            event_id TEXT PRIMARY KEY,
            machine_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            chain_epoch INTEGER NOT NULL,
            seq INTEGER NOT NULL,
            self_hash TEXT NOT NULL,
            prev TEXT NOT NULL,
            payload BLOB NOT NULL,
            shipped INTEGER NOT NULL DEFAULT 0,
            terminal INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0
        );
        CREATE UNIQUE INDEX IF NOT EXISTS events_chain_unique
            ON events(machine_id, agent_id, chain_epoch, seq);
        CREATE INDEX IF NOT EXISTS events_pending_idx ON events(shipped, terminal);
        CREATE TABLE IF NOT EXISTS chain_tail (
            machine_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            chain_epoch INTEGER NOT NULL,
            last_seq INTEGER NOT NULL,
            last_self_hash TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY(machine_id, agent_id, chain_epoch)
        );
        CREATE TABLE IF NOT EXISTS quarantine (
            rowid INTEGER NOT NULL,
            event_id TEXT NOT NULL,
            status INTEGER NOT NULL,
            response_body TEXT NOT NULL,
            quarantined_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS consent_transitions (
            transition_id TEXT PRIMARY KEY,
            next_consent TEXT NOT NULL,
            committed_at INTEGER NOT NULL
        );
        ",
    )?;
    Ok(connection)
}

pub fn is_valid_identity(identity: &Identity) -> bool {
    identity.version == 1
        && is_cuid2(&identity.user_cuid)
        && is_uuid_v7(&identity.machine_uuid)
        && identity.machine_secret.len() == 64
        && identity
            .machine_secret
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && !identity.created_at.is_empty()
        && identity.user_account_id.as_deref().is_none_or(is_cuid2)
}

pub(crate) fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), CliError> {
    atomic_write_with(path, |file| {
        file.write_all(contents)?;
        Ok(())
    })
}

pub(crate) fn atomic_write_with<T>(
    path: &Path,
    action: impl FnOnce(&mut fs::File) -> Result<T, CliError>,
) -> Result<T, CliError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let sync_stop = parent
        .ancestors()
        .find(|candidate| candidate.exists())
        .unwrap_or(parent)
        .to_path_buf();
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    let output = match action(&mut file) {
        Ok(output) => output,
        Err(error) => {
            drop(file);
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
    };
    if let Err(error) = file.sync_all() {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    set_private_file_mode(&temporary)?;
    if let Err(error) = atomic_replace(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    set_private_file_mode(path)?;
    sync_directory_tree(parent, &sync_stop)?;
    Ok(output)
}

fn read_consent_locked(paths: &SnoPaths) -> Result<ConsentValue, CliError> {
    recover_consent_transition(paths)?;
    match fs::read(&paths.consent_path) {
        Ok(bytes) => {
            let state: ConsentState = serde_json::from_slice(&bytes).map_err(|_| {
                CliError::runtime(
                    "invalid_consent_state",
                    format!("consent is malformed at {}", paths.consent_path.display()),
                )
            })?;
            if state.version != 1 {
                return Err(CliError::runtime(
                    "invalid_consent_state",
                    format!("consent is malformed at {}", paths.consent_path.display()),
                ));
            }
            Ok(state.value)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DEFAULT_CONSENT),
        Err(error) => Err(error.into()),
    }
}

fn write_consent(paths: &SnoPaths, value: ConsentValue) -> Result<(), CliError> {
    atomic_write_json(
        &paths.consent_path,
        &ConsentState {
            version: 1,
            value,
            updated_at: now_iso(),
        },
    )
}

fn apply_consent_transition(
    paths: &SnoPaths,
    current: ConsentValue,
    next: ConsentValue,
    reason: &str,
) -> Result<(), CliError> {
    let identity = bootstrap_identity_at(paths)?;
    let transition = ConsentTransition {
        version: 1,
        transition_id: Uuid::now_v7().to_string(),
        from: current,
        to: next,
        reason: reason.to_owned(),
        prepared_at: now_iso(),
    };
    atomic_write_json(&paths.consent_transition_path, &transition)?;
    let database_result = (|| {
        let mut connection = open_buffer(&paths.buffer_path)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let agents = list_agents(&transaction, &identity.machine_uuid)?;
        if current == ConsentValue::Off && next != ConsentValue::Off {
            for agent in &agents {
                let epoch = current_epoch(&transaction, &identity.machine_uuid, agent)? + 1;
                append_agent_identify(&transaction, &identity, agent, epoch, next, false)?;
                append_consent_change(
                    &transaction,
                    &identity,
                    agent,
                    epoch,
                    next,
                    current,
                    next,
                    reason,
                )?;
            }
        } else {
            for agent in &agents {
                let epoch = current_epoch(&transaction, &identity.machine_uuid, agent)?;
                if !has_tail(&transaction, &identity.machine_uuid, agent, epoch)? {
                    append_agent_identify(
                        &transaction,
                        &identity,
                        agent,
                        epoch,
                        current,
                        current == ConsentValue::Off,
                    )?;
                }
                append_consent_change(
                    &transaction,
                    &identity,
                    agent,
                    epoch,
                    current,
                    current,
                    next,
                    reason,
                )?;
            }
            append_post_transition_identities(&transaction, &identity, &agents, next)?;
        }
        transaction.execute(
            "INSERT INTO consent_transitions (transition_id, next_consent, committed_at) VALUES (?1, ?2, ?3)",
            params![
                transition.transition_id,
                transition.to.to_string(),
                Utc::now().timestamp_millis()
            ],
        )?;
        transaction.commit()?;
        Ok::<(), CliError>(())
    })();
    if let Err(error) = database_result {
        let _ = clear_consent_transition(paths);
        return Err(error);
    }
    write_consent(paths, next)?;
    clear_consent_transition(paths)
}

fn append_post_transition_identities(
    connection: &Connection,
    identity: &Identity,
    agents: &[String],
    next: ConsentValue,
) -> Result<(), CliError> {
    for agent in agents {
        let epoch = current_epoch(connection, &identity.machine_uuid, agent)? + 1;
        append_agent_identify(
            connection,
            identity,
            agent,
            epoch,
            next,
            next == ConsentValue::Off,
        )?;
    }
    Ok(())
}

fn clear_pause(paths: &SnoPaths) -> Result<(), CliError> {
    remove_state_file(&paths.pause_path)
}

fn recover_consent_transition(paths: &SnoPaths) -> Result<(), CliError> {
    let bytes = match fs::read(&paths.consent_transition_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let pending = serde_json::from_slice::<ConsentTransition>(&bytes).map_err(|_| {
        CliError::runtime(
            "invalid_consent_transition",
            format!(
                "consent transition is malformed at {}",
                paths.consent_transition_path.display()
            ),
        )
    })?;
    if pending.version != 1 {
        return Err(CliError::runtime(
            "invalid_consent_transition",
            format!(
                "consent transition is malformed at {}",
                paths.consent_transition_path.display()
            ),
        ));
    }
    let connection = open_buffer(&paths.buffer_path)?;
    let committed = connection
        .query_row(
            "SELECT next_consent FROM consent_transitions WHERE transition_id = ?1",
            params![pending.transition_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    match committed {
        Some(next) if next == pending.to.to_string() => write_consent(paths, pending.to)?,
        Some(_) => {
            return Err(CliError::runtime(
                "invalid_consent_transition",
                "consent transition does not match its committed database record",
            ));
        }
        None => {}
    }
    clear_consent_transition(paths)
}

fn clear_consent_transition(paths: &SnoPaths) -> Result<(), CliError> {
    remove_state_file(&paths.consent_transition_path)
}

fn remove_state_file(path: &Path) -> Result<(), CliError> {
    match fs::remove_file(path) {
        Ok(()) => {
            sync_directory_tree(
                path.parent().unwrap_or_else(|| Path::new(".")),
                path.parent().unwrap_or_else(|| Path::new(".")),
            )?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn bootstrap_identity_at(paths: &SnoPaths) -> Result<Identity, CliError> {
    if let Some(parent) = paths.identity_path.parent() {
        let parent_existed = parent.exists();
        fs::create_dir_all(parent)?;
        if !parent_existed {
            set_private_directory_mode(parent)?;
        }
    }
    with_identity_lock(paths, || {
        if let Some(identity) = read_valid_identity(&paths.identity_path)? {
            return Ok(identity);
        }
        let identity = create_identity();
        atomic_write_json(&paths.identity_path, &identity)?;
        Ok(identity)
    })
}

fn read_valid_identity(path: &Path) -> Result<Option<Identity>, CliError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let identity = serde_json::from_slice::<Identity>(&bytes).map_err(|_| {
        CliError::runtime(
            "invalid_identity",
            format!("identity is malformed at {}", path.display()),
        )
    })?;
    if !is_valid_identity(&identity) {
        return Err(CliError::runtime(
            "invalid_identity",
            format!("identity is malformed at {}", path.display()),
        ));
    }
    Ok(Some(identity))
}

fn create_identity() -> Identity {
    let mut secret = [0_u8; 32];
    rand::rng().fill_bytes(&mut secret);
    Identity {
        version: 1,
        user_cuid: create_cuid2(),
        machine_uuid: Uuid::now_v7().to_string(),
        machine_secret: hex::encode(secret),
        created_at: now_iso(),
        default_project_id: None,
        user_account_id: None,
    }
}

fn create_cuid2() -> String {
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const ALPHANUMERIC: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0_u8; 24];
    rand::rng().fill_bytes(&mut bytes);
    let mut output = String::with_capacity(24);
    output.push(LETTERS[usize::from(bytes[0]) % LETTERS.len()] as char);
    for byte in &bytes[1..] {
        output.push(ALPHANUMERIC[usize::from(*byte) % ALPHANUMERIC.len()] as char);
    }
    output
}

fn is_cuid2(value: &str) -> bool {
    value.len() == 24
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn is_uuid_v7(value: &str) -> bool {
    value == value.to_ascii_lowercase()
        && Uuid::parse_str(value).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

fn with_identity_lock<T>(
    paths: &SnoPaths,
    action: impl FnOnce() -> Result<T, CliError>,
) -> Result<T, CliError> {
    with_file_lock(&paths.identity_lock_path, action)
}

fn with_consent_lock<T>(
    paths: &SnoPaths,
    action: impl FnOnce() -> Result<T, CliError>,
) -> Result<T, CliError> {
    with_file_lock(&paths.consent_lock_path, action)
}

fn with_file_lock<T>(
    lock_path: &Path,
    action: impl FnOnce() -> Result<T, CliError>,
) -> Result<T, CliError> {
    let parent = lock_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let started = Instant::now();
    let mut lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    loop {
        match lock_file.try_lock_exclusive() {
            Ok(()) => break,
            Err(error) if is_lock_contended(&error) => {
                if started.elapsed() > Duration::from_secs(5) {
                    return Err(error.into());
                }
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error.into()),
        }
    }
    lock_file.set_len(0)?;
    writeln!(lock_file, "{}", std::process::id())?;
    let result = action();
    let _ = lock_file.set_len(0);
    let unlock_result = FileExt::unlock(&lock_file);
    match result {
        Err(error) => Err(error),
        Ok(value) => {
            unlock_result?;
            Ok(value)
        }
    }
}

fn is_lock_contended(error: &std::io::Error) -> bool {
    error.raw_os_error() == lock_contended_error().raw_os_error()
}

fn list_agents(connection: &Connection, machine_id: &str) -> Result<Vec<String>, CliError> {
    let mut statement =
        connection.prepare("SELECT DISTINCT agent_id FROM chain_tail WHERE machine_id = ?1")?;
    let values = statement
        .query_map(params![machine_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(if values.is_empty() {
        vec!["codex".to_owned()]
    } else {
        values
    })
}

fn current_epoch(
    connection: &Connection,
    machine_id: &str,
    agent_id: &str,
) -> Result<i64, CliError> {
    Ok(connection.query_row(
        "SELECT COALESCE(MAX(chain_epoch), 0) FROM chain_tail WHERE machine_id = ?1 AND agent_id = ?2",
        params![machine_id, agent_id],
        |row| row.get(0),
    )?)
}

fn has_tail(
    connection: &Connection,
    machine_id: &str,
    agent_id: &str,
    chain_epoch: i64,
) -> Result<bool, CliError> {
    Ok(connection
        .query_row(
            "SELECT 1 FROM chain_tail WHERE machine_id = ?1 AND agent_id = ?2 AND chain_epoch = ?3",
            params![machine_id, agent_id, chain_epoch],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

fn append_agent_identify(
    connection: &Connection,
    identity: &Identity,
    agent_id: &str,
    chain_epoch: i64,
    consent: ConsentValue,
    terminal: bool,
) -> Result<(), CliError> {
    append_event(
        connection,
        identity,
        agent_id,
        chain_epoch,
        "agent.identify",
        consent,
        json!({
            "agent_id": agent_id,
            "machine_id": identity.machine_uuid,
            "sdk_version": SNO_OBSERVE_VERSION,
        }),
        terminal,
    )
}

#[allow(clippy::too_many_arguments)]
fn append_consent_change(
    connection: &Connection,
    identity: &Identity,
    agent_id: &str,
    chain_epoch: i64,
    consent: ConsentValue,
    from: ConsentValue,
    to: ConsentValue,
    reason: &str,
) -> Result<(), CliError> {
    append_event(
        connection,
        identity,
        agent_id,
        chain_epoch,
        "consent.change",
        consent,
        json!({ "from": from, "to": to, "reason": reason }),
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn append_event(
    connection: &Connection,
    identity: &Identity,
    agent_id: &str,
    chain_epoch: i64,
    event_type: &str,
    consent: ConsentValue,
    payload: Value,
    terminal: bool,
) -> Result<(), CliError> {
    let tail = connection
        .query_row(
            "SELECT last_seq, last_self_hash FROM chain_tail WHERE machine_id = ?1 AND agent_id = ?2 AND chain_epoch = ?3",
            params![identity.machine_uuid, agent_id, chain_epoch],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    let (seq, previous) = tail
        .map(|(last_seq, last_hash)| (last_seq + 1, last_hash))
        .unwrap_or_else(|| (0, "GENESIS".to_owned()));
    let event_id = Uuid::now_v7().to_string();
    let timestamp = Utc::now().timestamp_millis();
    let mut scope = json!({
        "agent_id": agent_id,
        "machine_id": identity.machine_uuid,
        "user_id": identity.user_cuid,
    });
    if let Some(account) = &identity.user_account_id {
        scope["user_account_id"] = Value::String(account.clone());
    }
    let preimage = format!(
        "v1\n{event_id}\n{event_type}\n{timestamp}\n{}\n{chain_epoch}\n{seq}\n{consent}\n0\n{}\n{previous}",
        canonical_json(&scope)?,
        canonical_json(&payload)?,
    );
    let self_hash = hex::encode(Sha256::digest(preimage.as_bytes()));
    let serialized = serialize_envelope(
        &event_id,
        event_type,
        timestamp,
        consent,
        chain_epoch,
        seq,
        &scope,
        &previous,
        &self_hash,
        &payload,
    )?;
    connection.execute(
        "INSERT INTO events (event_id, machine_id, agent_id, chain_epoch, seq, self_hash, prev, payload, shipped, terminal, created_at, attempts) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10, 0)",
        params![
            event_id,
            identity.machine_uuid,
            agent_id,
            chain_epoch,
            seq,
            self_hash,
            previous,
            serialized.as_bytes(),
            if terminal { 1_i64 } else { 0_i64 },
            timestamp,
        ],
    )?;
    connection.execute(
        "INSERT INTO chain_tail (machine_id, agent_id, chain_epoch, last_seq, last_self_hash, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(machine_id, agent_id, chain_epoch) DO UPDATE SET last_seq = excluded.last_seq, last_self_hash = excluded.last_self_hash, updated_at = excluded.updated_at",
        params![identity.machine_uuid, agent_id, chain_epoch, seq, self_hash, timestamp],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn serialize_envelope(
    event_id: &str,
    event_type: &str,
    timestamp: i64,
    consent: ConsentValue,
    chain_epoch: i64,
    seq: i64,
    scope: &Value,
    previous: &str,
    self_hash: &str,
    payload: &Value,
) -> Result<String, CliError> {
    Ok(format!(
        "{{\"schema_version\":\"v1\",\"event_id\":{},\"event_type\":{},\"lane\":\"memory\",\"ts_edge_ms\":{timestamp},\"consent_level\":{},\"redacted\":false,\"chain_epoch\":{chain_epoch},\"seq\":{seq},\"scope\":{},\"hash_chain\":{{\"prev\":{},\"self\":{}}},\"payload\":{}}}",
        serde_json::to_string(event_id)?,
        serde_json::to_string(event_type)?,
        serde_json::to_string(&consent.to_string())?,
        canonical_json(scope)?,
        serde_json::to_string(previous)?,
        serde_json::to_string(self_hash)?,
        canonical_json(payload)?,
    ))
}

fn canonical_json(value: &Value) -> Result<String, CliError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            Ok(serde_json::to_string(value)?)
        }
        Value::Array(values) => {
            let encoded = values
                .iter()
                .map(canonical_json)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("[{}]", encoded.join(",")))
        }
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            let fields = keys
                .into_iter()
                .map(|key| {
                    Ok(format!(
                        "{}:{}",
                        serde_json::to_string(key)?,
                        canonical_json(&values[key])?
                    ))
                })
                .collect::<Result<Vec<String>, CliError>>()?;
            Ok(format!("{{{}}}", fields.join(",")))
        }
    }
}

fn atomic_write_json(path: &Path, value: &impl Serialize) -> Result<(), CliError> {
    let mut contents = serde_json::to_string_pretty(value)?;
    contents.push('\n');
    atomic_write(path, contents.as_bytes())
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn set_private_file_mode(_path: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(_path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn set_private_directory_mode(_path: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(_path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory_tree(path: &Path, stop: &Path) -> Result<(), CliError> {
    let mut current = path;
    loop {
        let directory = current;
        fs::File::open(directory)?.sync_all()?;
        if directory == stop {
            break;
        }
        current = directory.parent().ok_or_else(|| {
            CliError::runtime("runtime_error", "directory durability boundary is invalid")
        })?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory_tree(_path: &Path, _stop: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Barrier, mpsc};
    use std::thread;
    use std::time::Duration;

    use rusqlite::params;
    use serde_json::json;
    use tempfile::TempDir;

    use super::{
        ConsentTransition, ConsentValue, SnoPaths, apply_consent_transition, atomic_write_json,
        bootstrap_identity_at, canonical_json, open_buffer, read_consent_locked,
        read_valid_identity, update_identity_account_at, with_file_lock, write_consent,
    };

    fn test_paths(profile: &TempDir) -> SnoPaths {
        SnoPaths {
            identity_path: profile.path().join("identity.json"),
            identity_lock_path: profile.path().join("identity.lock"),
            buffer_path: profile.path().join("buffer.db"),
            consent_path: profile.path().join("state/consent.json"),
            pause_path: profile.path().join("state/consent-prior.json"),
            consent_transition_path: profile.path().join("state/consent-transition.json"),
            consent_lock_path: profile.path().join("state/consent.lock"),
        }
    }

    #[test]
    fn canonical_json_sorts_nested_object_keys() {
        let value = json!({
            "z": 1,
            "a": {"two": 2, "one": 1},
            "m": [true, {"b": 2, "a": 1}],
        });
        assert_eq!(
            canonical_json(&value).unwrap(),
            r#"{"a":{"one":1,"two":2},"m":[true,{"a":1,"b":2}],"z":1}"#
        );
    }

    #[test]
    fn consent_events_roll_back_together_on_database_failure() {
        let profile = TempDir::new().unwrap();
        let paths = test_paths(&profile);
        let identity = bootstrap_identity_at(&paths).unwrap();
        let connection = open_buffer(&paths.buffer_path).unwrap();
        for agent in ["alpha", "beta"] {
            connection
                .execute(
                    "INSERT INTO chain_tail (machine_id, agent_id, chain_epoch, last_seq, last_self_hash, updated_at) VALUES (?1, ?2, 0, 0, 'hash', 0)",
                    params![identity.machine_uuid, agent],
                )
                .unwrap();
        }
        connection
            .execute_batch(
                "CREATE TRIGGER fail_beta BEFORE INSERT ON events WHEN NEW.agent_id = 'beta' BEGIN SELECT RAISE(ABORT, 'forced failure'); END;",
            )
            .unwrap();
        drop(connection);

        let result = apply_consent_transition(
            &paths,
            ConsentValue::MetadataOnly,
            ConsentValue::Full,
            "test",
        );
        assert_eq!(result.unwrap_err().code, "runtime_error");
        let connection = open_buffer(&paths.buffer_path).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM events", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert!(!paths.consent_path.exists());
        assert!(!paths.consent_transition_path.exists());
    }

    #[test]
    fn consent_change_ignores_agents_from_a_previous_machine() {
        let profile = TempDir::new().unwrap();
        let paths = test_paths(&profile);
        let identity = bootstrap_identity_at(&paths).unwrap();
        let connection = open_buffer(&paths.buffer_path).unwrap();
        connection
            .execute(
                "INSERT INTO chain_tail (machine_id, agent_id, chain_epoch, last_seq, last_self_hash, updated_at) VALUES ('previous-machine', 'legacy-agent', 0, 0, 'hash', 0)",
                [],
            )
            .unwrap();
        drop(connection);

        apply_consent_transition(
            &paths,
            ConsentValue::MetadataOnly,
            ConsentValue::Full,
            "test",
        )
        .unwrap();
        let connection = open_buffer(&paths.buffer_path).unwrap();
        let agents = connection
            .prepare("SELECT DISTINCT agent_id FROM events WHERE machine_id = ?1")
            .unwrap()
            .query_map(params![identity.machine_uuid], |row| {
                row.get::<_, String>(0)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(agents, ["codex"]);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM events WHERE agent_id = 'legacy-agent'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn committed_consent_transition_recovers_before_reading_state() {
        let profile = TempDir::new().unwrap();
        let paths = test_paths(&profile);
        write_consent(&paths, ConsentValue::Full).unwrap();
        let pending = ConsentTransition {
            version: 1,
            transition_id: "transition-1".to_owned(),
            from: ConsentValue::Full,
            to: ConsentValue::Off,
            reason: "test".to_owned(),
            prepared_at: "2026-07-15T00:00:00.000Z".to_owned(),
        };
        atomic_write_json(&paths.consent_transition_path, &pending).unwrap();
        let connection = open_buffer(&paths.buffer_path).unwrap();
        connection
            .execute(
                "INSERT INTO consent_transitions (transition_id, next_consent, committed_at) VALUES (?1, ?2, 0)",
                params![pending.transition_id, pending.to.to_string()],
            )
            .unwrap();
        drop(connection);

        assert_eq!(read_consent_locked(&paths).unwrap(), ConsentValue::Off);
        assert!(!paths.consent_transition_path.exists());
        let saved = fs::read_to_string(&paths.consent_path).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&saved).unwrap()["value"],
            "off"
        );
    }

    #[test]
    fn concurrent_account_claims_cannot_overwrite_each_other() {
        let profile = TempDir::new().unwrap();
        let paths = test_paths(&profile);
        let identity = bootstrap_identity_at(&paths).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let handles = ["a11111111111111111111111", "b22222222222222222222222"].map(|account_id| {
            let paths = paths.clone();
            let identity = identity.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                update_identity_account_at(&paths, &identity, account_id)
            })
        });
        let results = handles.map(|handle| handle.join().unwrap());
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter_map(|result| result.as_ref().err())
                .next()
                .unwrap()
                .code,
            "claim_account_conflict"
        );
        let saved = read_valid_identity(&paths.identity_path).unwrap().unwrap();
        let winner = results
            .iter()
            .find_map(|result| result.as_ref().ok())
            .unwrap();
        assert_eq!(saved.user_account_id, winner.user_account_id);
    }

    #[test]
    fn operating_system_lock_serializes_forced_overlap() {
        let profile = TempDir::new().unwrap();
        let lock_path = profile.path().join("state.lock");
        let (first_entered_tx, first_entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let first_path = lock_path.clone();
        let first = thread::spawn(move || {
            with_file_lock(&first_path, || {
                first_entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
        });
        first_entered_rx.recv().unwrap();

        let (second_entered_tx, second_entered_rx) = mpsc::channel();
        let second = thread::spawn(move || {
            with_file_lock(&lock_path, || {
                second_entered_tx.send(()).unwrap();
                Ok(())
            })
        });
        assert!(
            second_entered_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err()
        );
        release_tx.send(()).unwrap();
        first.join().unwrap().unwrap();
        second_entered_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        second.join().unwrap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn identity_bootstrap_preserves_existing_parent_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let profile = TempDir::new().unwrap();
        let shared = profile.path().join("shared");
        fs::create_dir(&shared).unwrap();
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o777)).unwrap();
        let mut paths = test_paths(&profile);
        paths.identity_path = shared.join("identity.json");
        paths.identity_lock_path = shared.join("identity.lock");
        bootstrap_identity_at(&paths).unwrap();
        assert_eq!(
            fs::metadata(&shared).unwrap().permissions().mode() & 0o777,
            0o777
        );
        assert_eq!(
            fs::metadata(&paths.identity_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
