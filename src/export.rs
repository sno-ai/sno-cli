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
use tar::Header;

use crate::cli::print_json;
use crate::error::CliError;
use crate::state::{SnoPaths, atomic_write_with, open_buffer};

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

struct ExportStats {
    event_count: usize,
    bytes: u64,
    tarball_hash: Option<String>,
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

struct ChainAccumulator {
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
    let mut connection = open_buffer(&paths.buffer_path)?;
    let transaction = connection.transaction()?;
    let stats = if let Some(path) = &output_path {
        atomic_write_with(Path::new(path), |file| {
            write_export(&transaction, format, file)
        })?
    } else if json_enabled {
        write_export(&transaction, format, &mut io::sink())?
    } else {
        write_export(&transaction, format, &mut io::stdout().lock())?
    };
    transaction.commit()?;

    if json_enabled {
        let mut output = json!({
            "format": format,
            "path": output_path,
            "event_count": stats.event_count,
            "bytes": stats.bytes,
        });
        if let Some(hash) = stats.tarball_hash {
            output["tarball_sha256"] = Value::String(hash);
        }
        print_json(&output)?;
    } else if let Some(path) = &output_path
        && format == ExportFormat::Tarball
    {
        println!("exported {} events to {path}", stats.event_count);
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

fn write_export(
    connection: &Connection,
    format: ExportFormat,
    output: &mut dyn Write,
) -> Result<ExportStats, CliError> {
    match format {
        ExportFormat::Jsonl => write_jsonl(connection, output),
        ExportFormat::Csv => write_csv(connection, output),
        ExportFormat::Tarball => write_tarball(connection, output),
    }
}

fn write_jsonl(connection: &Connection, output: &mut dyn Write) -> Result<ExportStats, CliError> {
    let mut counted = CountingWriter::new(output);
    let mut event_count = 0;
    for_each_event(connection, |row| {
        let envelope = parse_envelope(&row.payload)?;
        serde_json::to_writer(&mut counted, &envelope)?;
        counted.write_all(b"\n")?;
        event_count += 1;
        Ok(())
    })?;
    counted.flush()?;
    Ok(ExportStats {
        event_count,
        bytes: counted.bytes,
        tarball_hash: None,
    })
}

fn write_csv(connection: &Connection, output: &mut dyn Write) -> Result<ExportStats, CliError> {
    let mut counted = CountingWriter::new(output);
    counted.write_all(
        b"event_id,event_type,ts_edge_ms,consent_level,redacted,agent_id,chain_epoch,seq,self,prev\n",
    )?;
    let mut event_count = 0;
    for_each_event(connection, |row| {
        let envelope = parse_envelope(&row.payload)?;
        counted.write_all(build_csv_record(&envelope)?.as_bytes())?;
        counted.write_all(b"\n")?;
        event_count += 1;
        Ok(())
    })?;
    counted.flush()?;
    Ok(ExportStats {
        event_count,
        bytes: counted.bytes,
        tarball_hash: None,
    })
}

fn write_tarball(connection: &Connection, output: &mut dyn Write) -> Result<ExportStats, CliError> {
    let mut events_hash = Sha256::new();
    let mut events_bytes = 0_u64;
    let mut event_count = 0_usize;
    let mut groups: BTreeMap<(String, String, i64), ChainAccumulator> = BTreeMap::new();
    for_each_event(connection, |row| {
        let envelope = parse_envelope(&row.payload)?;
        let compact = serde_json::to_vec(&envelope)?;
        events_hash.update(&compact);
        events_hash.update(b"\n");
        events_bytes += compact.len() as u64 + 1;
        event_count += 1;
        update_chain(&mut groups, &row, &envelope)?;
        Ok(())
    })?;

    let manifest = Manifest {
        manifest_version: 1,
        exported_at_ms: Utc::now().timestamp_millis(),
        sdk_version: "0.1.0",
        event_count,
        events_jsonl_sha256: hex::encode(events_hash.finalize()),
        chains: finish_chains(groups),
        verification: ManifestVerification {
            rule: "v1",
            notes: "Re-derive each event self hash and confirm previous-link continuity within each chain; gap-free sequence numbers are required.",
        },
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    manifest_bytes.push(b'\n');

    let mut hashed = HashingWriter::new(output);
    {
        let mut encoder = GzEncoder::new(&mut hashed, Compression::default());
        write_tar_header(&mut encoder, "events.jsonl", events_bytes)?;
        for_each_event(connection, |row| {
            let envelope = parse_envelope(&row.payload)?;
            serde_json::to_writer(&mut encoder, &envelope)?;
            encoder.write_all(b"\n")?;
            Ok(())
        })?;
        write_tar_padding(&mut encoder, events_bytes)?;
        write_tar_header(&mut encoder, "MANIFEST.json", manifest_bytes.len() as u64)?;
        encoder.write_all(&manifest_bytes)?;
        write_tar_padding(&mut encoder, manifest_bytes.len() as u64)?;
        encoder.write_all(&[0_u8; 1024])?;
        encoder.finish()?;
    }
    let (bytes, hash) = hashed.finish();
    Ok(ExportStats {
        event_count,
        bytes,
        tarball_hash: Some(hash),
    })
}

fn for_each_event(
    connection: &Connection,
    mut action: impl FnMut(EventRow) -> Result<(), CliError>,
) -> Result<(), CliError> {
    let mut statement = connection.prepare(
        "SELECT machine_id, agent_id, chain_epoch, seq, self_hash, payload, shipped FROM events ORDER BY rowid ASC",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        action(EventRow {
            machine_id: row.get(0)?,
            agent_id: row.get(1)?,
            chain_epoch: row.get(2)?,
            seq: row.get(3)?,
            self_hash: row.get(4)?,
            payload: row.get(5)?,
            shipped: row.get(6)?,
        })?;
    }
    Ok(())
}

fn update_chain(
    groups: &mut BTreeMap<(String, String, i64), ChainAccumulator>,
    row: &EventRow,
    envelope: &Value,
) -> Result<(), CliError> {
    let is_off = envelope
        .get("consent_level")
        .and_then(Value::as_str)
        .ok_or_else(invalid_envelope)?
        == "off";
    let group = groups
        .entry((
            row.machine_id.clone(),
            row.agent_id.clone(),
            row.chain_epoch,
        ))
        .or_insert_with(|| ChainAccumulator {
            first_seq: row.seq,
            last_seq: row.seq,
            first_self: row.self_hash.clone(),
            last_self: row.self_hash.clone(),
            row_count: 0,
            shipped_count: 0,
            off_period: true,
        });
    if row.seq < group.first_seq {
        group.first_seq = row.seq;
        group.first_self.clone_from(&row.self_hash);
    }
    if row.seq > group.last_seq {
        group.last_seq = row.seq;
        group.last_self.clone_from(&row.self_hash);
    }
    group.row_count += 1;
    group.shipped_count += usize::from(row.shipped == 1);
    group.off_period &= is_off;
    Ok(())
}

fn finish_chains(groups: BTreeMap<(String, String, i64), ChainAccumulator>) -> Vec<ManifestChain> {
    groups
        .into_iter()
        .map(
            |((machine_id, agent_id, chain_epoch), group)| ManifestChain {
                machine_id,
                agent_id,
                chain_epoch,
                first_seq: group.first_seq,
                last_seq: group.last_seq,
                first_self: group.first_self,
                last_self: group.last_self,
                row_count: group.row_count,
                shipped_count: group.shipped_count,
                off_period: group.off_period,
            },
        )
        .collect()
}

fn write_tar_header(output: &mut dyn Write, path: &str, size: u64) -> Result<(), CliError> {
    let mut header = Header::new_ustar();
    header.set_path(path)?;
    header.set_size(size);
    header.set_mode(0o644);
    header.set_mtime(Utc::now().timestamp() as u64);
    header.set_cksum();
    output.write_all(header.as_bytes())?;
    Ok(())
}

fn write_tar_padding(output: &mut dyn Write, size: u64) -> Result<(), CliError> {
    let padding = (512 - size % 512) % 512;
    if padding > 0 {
        output.write_all(&[0_u8; 512][..padding as usize])?;
    }
    Ok(())
}

fn build_csv_record(envelope: &Value) -> Result<String, CliError> {
    let fields = [
        csv_text_cell(&string_field(envelope, "event_id")?),
        csv_text_cell(&string_field(envelope, "event_type")?),
        integer_field(envelope, "ts_edge_ms")?.to_string(),
        csv_text_cell(&string_field(envelope, "consent_level")?),
        boolean_field(envelope, "redacted")?.to_string(),
        csv_text_cell(
            envelope
                .pointer("/scope/agent_id")
                .and_then(Value::as_str)
                .ok_or_else(invalid_envelope)?,
        ),
        integer_field(envelope, "chain_epoch")?.to_string(),
        integer_field(envelope, "seq")?.to_string(),
        csv_text_cell(
            envelope
                .pointer("/hash_chain/self")
                .and_then(Value::as_str)
                .ok_or_else(invalid_envelope)?,
        ),
        csv_text_cell(
            envelope
                .pointer("/hash_chain/prev")
                .and_then(Value::as_str)
                .ok_or_else(invalid_envelope)?,
        ),
    ];
    Ok(fields.map(|field| csv_cell(&field)).join(","))
}

fn parse_envelope(payload: &[u8]) -> Result<Value, CliError> {
    serde_json::from_slice(payload).map_err(|_| invalid_envelope())
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

fn csv_text_cell(value: &str) -> String {
    let significant = value
        .trim_start_matches(|character: char| character.is_whitespace() || character.is_control());
    if significant.starts_with(['=', '+', '-', '@']) {
        format!("'{value}")
    } else {
        value.to_owned()
    }
}

struct CountingWriter<'a> {
    inner: &'a mut dyn Write,
    bytes: u64,
}

impl<'a> CountingWriter<'a> {
    fn new(inner: &'a mut dyn Write) -> Self {
        Self { inner, bytes: 0 }
    }
}

impl Write for CountingWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.bytes += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct HashingWriter<'a> {
    inner: &'a mut dyn Write,
    hash: Sha256,
    bytes: u64,
}

impl<'a> HashingWriter<'a> {
    fn new(inner: &'a mut dyn Write) -> Self {
        Self {
            inner,
            hash: Sha256::new(),
            bytes: 0,
        }
    }

    fn finish(self) -> (u64, String) {
        (self.bytes, hex::encode(self.hash.finalize()))
    }
}

impl Write for HashingWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.hash.update(&buffer[..written]);
        self.bytes += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use rusqlite::params;
    use serde_json::json;
    use tempfile::TempDir;

    use super::{ExportFormat, build_csv_record, csv_text_cell, write_export};
    use crate::state::open_buffer;

    #[test]
    fn csv_text_cells_neutralize_spreadsheet_formulas() {
        for value in [
            "=1+1",
            "+cmd",
            "-2+3",
            "@SUM(A1:A2)",
            " \t=HYPERLINK(\"https://example.com\")",
            "\u{a0}@SUM(A1:A2)",
        ] {
            assert_eq!(csv_text_cell(value), format!("'{value}"));
        }
        assert_eq!(csv_text_cell("safe"), "safe");
    }

    #[test]
    fn csv_export_round_trips_special_text_once() {
        let values = ["event,one", "type\"two", "agent\nthree"];
        let payload = json!({
            "event_id": values[0],
            "event_type": values[1],
            "ts_edge_ms": 1,
            "consent_level": "metadata-only",
            "redacted": false,
            "scope": {"agent_id": values[2]},
            "chain_epoch": 0,
            "seq": 0,
            "hash_chain": {"self": "self", "prev": "GENESIS"}
        });
        let csv = format!("{}\n", build_csv_record(&payload).unwrap());
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv.as_bytes());
        let record = reader.records().next().unwrap().unwrap();
        assert_eq!(&record[0], values[0]);
        assert_eq!(&record[1], values[1]);
        assert_eq!(&record[5], values[2]);
    }

    #[test]
    fn jsonl_compacts_multiline_json() {
        let profile = TempDir::new().unwrap();
        let connection = open_buffer(&profile.path().join("buffer.db")).unwrap();
        insert_event(&connection, 0, b"{\n  \"value\": 1\n}");
        let mut output = Vec::new();
        write_export(&connection, ExportFormat::Jsonl, &mut output).unwrap();
        assert_eq!(output, b"{\"value\":1}\n");
    }

    #[test]
    fn large_jsonl_export_never_writes_the_full_buffer_at_once() {
        let profile = TempDir::new().unwrap();
        let mut connection = open_buffer(&profile.path().join("buffer.db")).unwrap();
        let transaction = connection.transaction().unwrap();
        for index in 0..2048 {
            let payload = serde_json::to_vec(&json!({
                "index": index,
                "value": "x".repeat(1024)
            }))
            .unwrap();
            insert_event(&transaction, index, &payload);
        }
        transaction.commit().unwrap();
        let mut output = ChunkLimitedWriter {
            max_chunk: 2048,
            bytes: 0,
        };
        let stats = write_export(&connection, ExportFormat::Jsonl, &mut output).unwrap();
        assert_eq!(stats.event_count, 2048);
        assert!(stats.bytes > 2_000_000);
        assert_eq!(stats.bytes, output.bytes);
    }

    fn insert_event(connection: &rusqlite::Connection, index: i64, payload: &[u8]) {
        connection
            .execute(
                "INSERT INTO events (event_id, machine_id, agent_id, chain_epoch, seq, self_hash, prev, payload, shipped, terminal, created_at, attempts) VALUES (?1, 'machine', 'agent', 0, ?2, 'self', 'prev', ?3, 0, 0, 0, 0)",
                params![format!("event-{index}"), index, payload],
            )
            .unwrap();
    }

    struct ChunkLimitedWriter {
        max_chunk: usize,
        bytes: u64,
    }

    impl Write for ChunkLimitedWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            if buffer.len() > self.max_chunk {
                return Err(io::Error::other("write exceeded streaming chunk limit"));
            }
            self.bytes += buffer.len() as u64;
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
