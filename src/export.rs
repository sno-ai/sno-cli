use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;

use chrono::Utc;
use flate2::Compression;
use flate2::write::GzEncoder;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tar::{Builder, Header};

use crate::cli::print_json;
use crate::error::CliError;
use crate::state::{SnoPaths, atomic_write, open_buffer};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Tarball,
    Jsonl,
    Csv,
}

struct EventRow {
    machine_id: String,
    agent_id: String,
    chain_epoch: i64,
    seq: i64,
    self_hash: String,
    payload: Vec<u8>,
    shipped: i64,
}

#[derive(Serialize)]
struct Manifest {
    manifest_version: u8,
    exported_at_ms: i64,
    sdk_version: &'static str,
    event_count: usize,
    events_jsonl_sha256: String,
    chains: Vec<ManifestChain>,
    verification: ManifestVerification,
}

#[derive(Serialize)]
struct ManifestChain {
    machine_id: String,
    agent_id: String,
    chain_epoch: i64,
    first_seq: i64,
    last_seq: i64,
    first_self: String,
    last_self: String,
    row_count: usize,
    shipped_count: usize,
    off_period: bool,
}

#[derive(Serialize)]
struct ManifestVerification {
    rule: &'static str,
    notes: &'static str,
}

pub fn run(
    requested_path: Option<String>,
    requested_format: Option<ExportFormat>,
    json_enabled: bool,
) -> Result<i32, CliError> {
    let format = requested_format.unwrap_or_else(|| infer_format(requested_path.as_deref()));
    let output_path = requested_path.or_else(|| {
        (format == ExportFormat::Tarball)
            .then(|| format!("./sno-export-{}.tar.gz", Utc::now().timestamp()))
    });
    let paths = SnoPaths::from_environment()?;
    let connection = open_buffer(&paths.buffer_path)?;
    let rows = read_rows(&connection)?;
    let (data, tarball_hash) = build_export(format, &rows)?;
    if let Some(path) = &output_path {
        atomic_write(Path::new(path), &data)?;
    }
    if json_enabled {
        let mut output = json!({
            "format": format,
            "path": output_path,
            "event_count": rows.len(),
            "bytes": data.len(),
        });
        if let Some(hash) = tarball_hash {
            output["tarball_sha256"] = Value::String(hash);
        }
        print_json(&output)?;
    } else if let Some(path) = &output_path {
        if format == ExportFormat::Tarball {
            println!("exported {} events to {path}", rows.len());
        }
    } else {
        io::stdout().lock().write_all(&data)?;
    }
    Ok(0)
}

fn infer_format(path: Option<&str>) -> ExportFormat {
    match path {
        Some(path) if path.ends_with(".csv") => ExportFormat::Csv,
        Some(path) if path.ends_with(".jsonl") => ExportFormat::Jsonl,
        _ => ExportFormat::Tarball,
    }
}

fn read_rows(connection: &Connection) -> Result<Vec<EventRow>, CliError> {
    let mut statement = connection.prepare(
        "SELECT machine_id, agent_id, chain_epoch, seq, self_hash, payload, shipped FROM events ORDER BY rowid ASC",
    )?;
    Ok(statement
        .query_map([], |row| {
            Ok(EventRow {
                machine_id: row.get(0)?,
                agent_id: row.get(1)?,
                chain_epoch: row.get(2)?,
                seq: row.get(3)?,
                self_hash: row.get(4)?,
                payload: row.get(5)?,
                shipped: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

fn build_export(
    format: ExportFormat,
    rows: &[EventRow],
) -> Result<(Vec<u8>, Option<String>), CliError> {
    match format {
        ExportFormat::Jsonl => Ok((build_jsonl(rows), None)),
        ExportFormat::Csv => Ok((build_csv(rows)?.into_bytes(), None)),
        ExportFormat::Tarball => {
            let bytes = build_tarball(rows)?;
            let hash = sha256(&bytes);
            Ok((bytes, Some(hash)))
        }
    }
}

fn build_jsonl(rows: &[EventRow]) -> Vec<u8> {
    let mut output = Vec::new();
    for row in rows {
        output.extend_from_slice(&row.payload);
        output.push(b'\n');
    }
    output
}

fn build_csv(rows: &[EventRow]) -> Result<String, CliError> {
    let mut lines = vec![
        "event_id,event_type,ts_edge_ms,consent_level,redacted,agent_id,chain_epoch,seq,self,prev"
            .to_owned(),
    ];
    for row in rows {
        let envelope: Value = serde_json::from_slice(&row.payload)?;
        let fields = [
            string_field(&envelope, "event_id")?,
            string_field(&envelope, "event_type")?,
            integer_field(&envelope, "ts_edge_ms")?.to_string(),
            string_field(&envelope, "consent_level")?,
            boolean_field(&envelope, "redacted")?.to_string(),
            envelope
                .pointer("/scope/agent_id")
                .and_then(Value::as_str)
                .ok_or_else(invalid_envelope)?
                .to_owned(),
            integer_field(&envelope, "chain_epoch")?.to_string(),
            integer_field(&envelope, "seq")?.to_string(),
            envelope
                .pointer("/hash_chain/self")
                .and_then(Value::as_str)
                .ok_or_else(invalid_envelope)?
                .to_owned(),
            envelope
                .pointer("/hash_chain/prev")
                .and_then(Value::as_str)
                .ok_or_else(invalid_envelope)?
                .to_owned(),
        ];
        lines.push(fields.map(|field| csv_cell(&field)).join(","));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn build_tarball(rows: &[EventRow]) -> Result<Vec<u8>, CliError> {
    let jsonl = build_jsonl(rows);
    let manifest = Manifest {
        manifest_version: 1,
        exported_at_ms: Utc::now().timestamp_millis(),
        sdk_version: "0.1.0",
        event_count: rows.len(),
        events_jsonl_sha256: sha256(&jsonl),
        chains: build_chains(rows)?,
        verification: ManifestVerification {
            rule: "v1",
            notes: "Re-derive each event self hash and confirm previous-link continuity within each chain; gap-free sequence numbers are required.",
        },
    };
    let mut manifest_bytes = serde_json::to_string_pretty(&manifest)?.into_bytes();
    manifest_bytes.push(b'\n');
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut builder = Builder::new(encoder);
    append_tar_entry(&mut builder, "events.jsonl", &jsonl)?;
    append_tar_entry(&mut builder, "MANIFEST.json", &manifest_bytes)?;
    let encoder = builder.into_inner()?;
    Ok(encoder.finish()?)
}

fn append_tar_entry(
    builder: &mut Builder<GzEncoder<Vec<u8>>>,
    path: &str,
    contents: &[u8],
) -> Result<(), CliError> {
    let mut header = Header::new_ustar();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(Utc::now().timestamp() as u64);
    header.set_cksum();
    builder.append_data(&mut header, path, contents)?;
    Ok(())
}

fn build_chains(rows: &[EventRow]) -> Result<Vec<ManifestChain>, CliError> {
    let mut groups: BTreeMap<(String, String, i64), Vec<&EventRow>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((
                row.machine_id.clone(),
                row.agent_id.clone(),
                row.chain_epoch,
            ))
            .or_default()
            .push(row);
    }
    let mut chains = Vec::new();
    for ((machine_id, agent_id, chain_epoch), mut group) in groups {
        group.sort_by_key(|row| row.seq);
        let first = group.first().expect("group is non-empty");
        let last = group.last().expect("group is non-empty");
        let off_period = group.iter().try_fold(true, |all_off, row| {
            let envelope: Value = serde_json::from_slice(&row.payload)?;
            Ok::<_, serde_json::Error>(
                all_off && envelope.get("consent_level").and_then(Value::as_str) == Some("off"),
            )
        })?;
        chains.push(ManifestChain {
            machine_id,
            agent_id,
            chain_epoch,
            first_seq: first.seq,
            last_seq: last.seq,
            first_self: first.self_hash.clone(),
            last_self: last.self_hash.clone(),
            row_count: group.len(),
            shipped_count: group.iter().filter(|row| row.shipped == 1).count(),
            off_period,
        });
    }
    Ok(chains)
}

fn string_field(value: &Value, key: &str) -> Result<String, CliError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(invalid_envelope)
}

fn integer_field(value: &Value, key: &str) -> Result<i64, CliError> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(invalid_envelope)
}

fn boolean_field(value: &Value, key: &str) -> Result<bool, CliError> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(invalid_envelope)
}

fn invalid_envelope() -> CliError {
    CliError::runtime(
        "invalid_buffer_event",
        "buffer contains an invalid event envelope",
    )
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
