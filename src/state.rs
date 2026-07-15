use std::env;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use chrono::{SecondsFormat, Utc};
use rand::RngCore;
use rusqlite::{Connection, OptionalExtension, params};
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

#[derive(Clone, Debug)]
pub struct SnoPaths {
    pub identity_path: PathBuf,
    pub identity_lock_path: PathBuf,
    pub buffer_path: PathBuf,
    pub consent_path: PathBuf,
    pub pause_path: PathBuf,
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
        Ok(Self {
            identity_path,
            identity_lock_path,
            buffer_path,
            consent_path,
            pause_path,
        })
    }
}

pub fn read_consent() -> Result<ConsentValue, CliError> {
    read_consent_from(&SnoPaths::from_environment()?)
}

pub fn set_consent(next: ConsentValue, reason: &str) -> Result<(), CliError> {
    let paths = SnoPaths::from_environment()?;
    let current = read_consent_from(&paths)?;
    if current == next {
        return Ok(());
    }
    apply_consent_transition(&paths, current, next, reason)
}

pub fn pause_telemetry() -> Result<(ConsentValue, bool), CliError> {
    let paths = SnoPaths::from_environment()?;
    let current = read_consent_from(&paths)?;
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
}

pub fn resume_telemetry() -> Result<ConsentValue, CliError> {
    let paths = SnoPaths::from_environment()?;
    let prior = match fs::read(&paths.pause_path) {
        Ok(bytes) => serde_json::from_slice::<PauseState>(&bytes)
            .ok()
            .filter(|state| state.version == 1)
            .map(|state| state.prior)
            .unwrap_or(DEFAULT_CONSENT),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => DEFAULT_CONSENT,
        Err(error) => return Err(error.into()),
    };
    let current = read_consent_from(&paths)?;
    if current != prior {
        apply_consent_transition(&paths, current, prior, "observe.resume")?;
    }
    match fs::remove_file(&paths.pause_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(prior)
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
    with_identity_lock(&paths, || {
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
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
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
    file.write_all(contents)?;
    file.sync_all()?;
    set_private_file_mode(&temporary)?;
    if let Err(error) = fs::rename(&temporary, path) {
        #[cfg(windows)]
        if matches!(
            error.kind(),
            std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::PermissionDenied
        ) {
            fs::remove_file(path)?;
            fs::rename(&temporary, path)?;
        } else {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        #[cfg(not(windows))]
        {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
    }
    set_private_file_mode(path)?;
    Ok(())
}

fn read_consent_from(paths: &SnoPaths) -> Result<ConsentValue, CliError> {
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
    let mut connection = open_buffer(&paths.buffer_path)?;
    let agents = list_agents(&connection)?;
    if current == ConsentValue::Off && next != ConsentValue::Off {
        write_consent(paths, next)?;
        for agent in agents {
            let epoch = current_epoch(&connection, &identity.machine_uuid, &agent)? + 1;
            append_agent_identify(&mut connection, &identity, &agent, epoch, next, false)?;
            append_consent_change(
                &mut connection,
                &identity,
                &agent,
                epoch,
                next,
                current,
                next,
                reason,
            )?;
        }
        return Ok(());
    }
    for agent in &agents {
        let epoch = current_epoch(&connection, &identity.machine_uuid, agent)?;
        if !has_tail(&connection, &identity.machine_uuid, agent, epoch)? {
            append_agent_identify(
                &mut connection,
                &identity,
                agent,
                epoch,
                current,
                current == ConsentValue::Off,
            )?;
        }
        append_consent_change(
            &mut connection,
            &identity,
            agent,
            epoch,
            current,
            current,
            next,
            reason,
        )?;
    }
    write_consent(paths, next)?;
    for agent in agents {
        let epoch = current_epoch(&connection, &identity.machine_uuid, &agent)? + 1;
        append_agent_identify(
            &mut connection,
            &identity,
            &agent,
            epoch,
            next,
            next == ConsentValue::Off,
        )?;
    }
    Ok(())
}

fn bootstrap_identity_at(paths: &SnoPaths) -> Result<Identity, CliError> {
    if let Some(parent) = paths.identity_path.parent() {
        fs::create_dir_all(parent)?;
        set_private_directory_mode(parent)?;
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
    let mut identity = match serde_json::from_slice::<Identity>(&bytes) {
        Ok(identity) => identity,
        Err(_) => return Ok(None),
    };
    if identity
        .user_account_id
        .as_deref()
        .is_some_and(|account| !is_cuid2(account))
    {
        identity.user_account_id = None;
    }
    Ok(is_valid_identity(&identity).then_some(identity))
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
    let parent = paths
        .identity_lock_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let started = Instant::now();
    let mut stale_break_attempted = false;
    let mut lock_file = loop {
        let result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&paths.identity_lock_path);
        match result {
            Ok(file) => break file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if started.elapsed() <= Duration::from_secs(5) {
                    thread::sleep(Duration::from_millis(25));
                    continue;
                }
                let stale = fs::metadata(&paths.identity_lock_path)
                    .and_then(|metadata| metadata.modified())
                    .ok()
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_none_or(|age| age > Duration::from_secs(60));
                if !stale_break_attempted && stale {
                    stale_break_attempted = true;
                    let _ = fs::remove_file(&paths.identity_lock_path);
                    continue;
                }
                return Err(error.into());
            }
            Err(error) => return Err(error.into()),
        }
    };
    writeln!(lock_file, "{}", std::process::id())?;
    let result = action();
    drop(lock_file);
    let _ = fs::remove_file(&paths.identity_lock_path);
    result
}

fn list_agents(connection: &Connection) -> Result<Vec<String>, CliError> {
    let mut statement = connection.prepare("SELECT DISTINCT agent_id FROM chain_tail")?;
    let values = statement
        .query_map([], |row| row.get::<_, String>(0))?
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
    connection: &mut Connection,
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
    connection: &mut Connection,
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
    connection: &mut Connection,
    identity: &Identity,
    agent_id: &str,
    chain_epoch: i64,
    event_type: &str,
    consent: ConsentValue,
    payload: Value,
    terminal: bool,
) -> Result<(), CliError> {
    let transaction = connection.transaction()?;
    let tail = transaction
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
    transaction.execute(
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
            i64::from(terminal),
            timestamp,
        ],
    )?;
    transaction.execute(
        "INSERT INTO chain_tail (machine_id, agent_id, chain_epoch, last_seq, last_self_hash, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(machine_id, agent_id, chain_epoch) DO UPDATE SET last_seq = excluded.last_seq, last_self_hash = excluded.last_self_hash, updated_at = excluded.updated_at",
        params![identity.machine_uuid, agent_id, chain_epoch, seq, self_hash, timestamp],
    )?;
    transaction.commit()?;
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

fn set_private_file_mode(path: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn set_private_directory_mode(path: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
