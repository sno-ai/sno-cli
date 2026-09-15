use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json, value::RawValue};
use sha2::{Digest, Sha256};

use crate::harness_slots::{Contract, PROGRAM_IDS, Requirements, version};
use crate::manifest::{self, Hook, Manifest, Operation, Program, Snapshot, Store};

pub const CONTRACT_SHA256: &str =
    "5c29a218bd7c3d43003fad6f0e3939914a3f82ede18eb60f2e5f31a3b9a11b92";
const MAX_BYTES: u64 = 64 * 1024 * 1024;
const SKILLS: &str = "skills/S-communication-and-handoff";
pub type Result<T> = std::result::Result<T, InstallError>;

#[derive(Debug)]
pub struct InstallError {
    pub exit_code: i32,
    pub message: String,
}
impl InstallError {
    pub fn source(message: impl Into<String>) -> Self {
        Self {
            exit_code: 3,
            message: message.into(),
        }
    }
    pub fn state(message: impl Into<String>) -> Self {
        Self {
            exit_code: 4,
            message: message.into(),
        }
    }
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            exit_code: 2,
            message: message.into(),
        }
    }
    pub(crate) fn path(path: &Path, error: impl fmt::Display) -> Self {
        Self::state(format!("{}: {error}", path.display()))
    }
}
impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for InstallError {}

fn parse_reach_version(value: &str) -> std::result::Result<String, String> {
    version(value).map_err(|e| e.message)?;
    Ok(value.to_owned())
}
fn parse_skills_tag(value: &str) -> std::result::Result<String, String> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err("skills tag must be nonblank and contain no whitespace".into());
    }
    Ok(value.to_owned())
}

#[derive(Debug, Default, Args)]
pub struct InstallOptions {
    #[arg(long,value_parser=parse_reach_version)]
    pub reach_version: Option<String>,
    #[arg(long,value_parser=parse_skills_tag)]
    pub skills_version: Option<String>,
    /// Use a local skills archive for the initial bootstrap; core releases remain published.
    #[arg(long, conflicts_with = "skills_version")]
    pub skills_archive: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct UpdateOptions {
    #[command(flatten)]
    pub versions: InstallOptions,
    #[arg(long, value_parser=["on","off"])]
    pub auto: Option<String>,
    #[arg(long)]
    pub quiet: bool,
}
pub enum Action {
    Assemble(InstallOptions),
    Update(UpdateOptions),
    Doctor,
    Remove { purge_state: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub entry_point: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSet {
    pub programs: Vec<Artifact>,
    pub skills: Artifact,
    pub contract_sha256: String,
}
pub trait ReleaseSource {
    fn resolve(
        &self,
        reach_version: Option<&str>,
        skills_version: Option<&str>,
        skills_archive: Option<&Artifact>,
    ) -> Result<ReleaseSet>;
    fn fetch(&self, url: &str) -> Result<Vec<u8>>;
    fn checkpoint(&self, _phase: &str) -> Result<()> {
        Ok(())
    }
}

fn reach_archive_suffix(os: &str, arch: &str) -> Result<String> {
    if !matches!(os, "linux" | "macos") {
        return Err(InstallError::usage("installer requires Linux or macOS"));
    }
    if !matches!((os, arch), ("linux", "x86_64") | ("macos", "aarch64")) {
        return Err(InstallError::usage(format!(
            "unsupported installer platform: {os}-{arch}"
        )));
    }
    Ok(format!("-{os}-{arch}.tar.gz"))
}

fn core_asset_version<'a>(name: &str, filename: &'a str, reach_suffix: &str) -> Option<&'a str> {
    filename
        .strip_prefix(&format!("{name}-"))?
        .strip_suffix(if name == "reach" {
            reach_suffix
        } else {
            ".tar.gz"
        })
}

fn asset_checksum_url(assets: &[Value], filename: &str) -> Option<String> {
    assets
        .iter()
        .find(|a| a["name"].as_str() == Some(&format!("{filename}.sha256")))
        .and_then(|a| a["url"].as_str())
        .map(str::to_owned)
}

fn exact_asset_url(assets: &[Value], filename: &str) -> Result<String> {
    let mut matches = assets
        .iter()
        .filter(|asset| asset["name"].as_str() == Some(filename));
    let asset = matches
        .next()
        .ok_or_else(|| InstallError::source(format!("missing release asset: {filename}")))?;
    if matches.next().is_some() {
        return Err(InstallError::source(format!(
            "ambiguous release asset: {filename}"
        )));
    }
    asset["url"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| InstallError::source(format!("missing artifact URL: {filename}")))
}

fn skills_release_artifact(
    release: &Value,
    fetch: &impl Fn(&str) -> Result<Vec<u8>>,
) -> Result<Artifact> {
    const ARCHIVE: &str = "final-skills.tar.gz";
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| InstallError::source("missing skills tag"))?;
    let assets = release["assets"]
        .as_array()
        .ok_or_else(|| InstallError::source("skills release missing assets"))?;
    let archive_url = exact_asset_url(assets, ARCHIVE)?;
    let checksum_url = exact_asset_url(assets, &format!("{ARCHIVE}.sha256"))?;
    let sha256 = std::str::from_utf8(&fetch(&checksum_url)?)
        .map_err(|e| InstallError::source(e.to_string()))?
        .split_whitespace()
        .next()
        .filter(|value| value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| InstallError::source("invalid skills checksum"))?
        .to_ascii_lowercase();
    let ref_url = format!(
        "https://api.github.com/repos/sno-ai/sno-station-skills/git/ref/tags/{}",
        url::form_urlencoded::byte_serialize(tag.as_bytes()).collect::<String>()
    );
    let reference: Value = serde_json::from_slice(&fetch(&ref_url)?)
        .map_err(|e| InstallError::source(e.to_string()))?;
    let mut object = reference["object"].clone();
    for _ in 0..8 {
        if object["type"].as_str() == Some("commit") {
            break;
        }
        if object["type"].as_str() != Some("tag") {
            return Err(InstallError::source(
                "skills tag does not resolve to a commit",
            ));
        }
        let tagged: Value = serde_json::from_slice(&fetch(
            object["url"]
                .as_str()
                .ok_or_else(|| InstallError::source("tag object URL missing"))?,
        )?)
        .map_err(|e| InstallError::source(e.to_string()))?;
        object = tagged["object"].clone();
    }
    let _commit = object["sha"]
        .as_str()
        .filter(|value| value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| InstallError::source("invalid skills commit"))?;
    if object["type"].as_str() != Some("commit") {
        return Err(InstallError::source("skills tag nesting exceeds limit"));
    }
    Ok(Artifact {
        name: "skills".into(),
        version: tag.into(),
        url: archive_url,
        sha256,
        entry_point: String::new(),
    })
}

pub struct GithubSource;
impl GithubSource {
    fn releases(&self, repo: &str, fetch: &impl Fn(&str) -> Result<Vec<u8>>) -> Result<Vec<Value>> {
        let url = format!("https://api.github.com/repos/sno-ai/{repo}/releases?per_page=100");
        serde_json::from_slice(&fetch(&url)?)
            .map_err(|e| InstallError::source(format!("{url}: {e}")))
    }
    fn resolve_for_platform(
        &self,
        reach_version: Option<&str>,
        skills_version: Option<&str>,
        os: &str,
        arch: &str,
        skills_archive: Option<&Artifact>,
        fetch: impl Fn(&str) -> Result<Vec<u8>>,
    ) -> Result<ReleaseSet> {
        let reach_suffix = reach_archive_suffix(os, arch)?;
        let core = self.releases("sno-station-core", &fetch)?;
        let mut programs = Vec::new();
        for name in PROGRAM_IDS {
            let mut choices = Vec::new();
            for release in &core {
                if release["draft"].as_bool() != Some(false)
                    || release["prerelease"].as_bool() != Some(false)
                {
                    continue;
                }
                let Some(assets) = release["assets"].as_array() else {
                    continue;
                };
                for asset in assets {
                    let Some(filename) = asset["name"].as_str() else {
                        continue;
                    };
                    let Some(v) = core_asset_version(name, filename, &reach_suffix) else {
                        continue;
                    };
                    let Ok(order) = version(v) else {
                        continue;
                    };
                    if name == "reach" && reach_version.is_some_and(|pin| pin != v) {
                        continue;
                    }
                    let checksum_url = asset_checksum_url(assets, filename);
                    let url = asset["url"]
                        .as_str()
                        .ok_or_else(|| InstallError::source("missing artifact URL"))?
                        .to_owned();
                    choices.push((
                        order,
                        Artifact {
                            name: name.into(),
                            version: v.into(),
                            url,
                            sha256: String::new(),
                            entry_point: format!(
                                "bin/{}",
                                if name == "reach" { "sno-reach" } else { name }
                            ),
                        },
                        checksum_url,
                    ));
                }
            }
            choices.sort_by_key(|(order, _, _)| *order);
            if let Some((_, mut artifact, checksum_url)) = choices.pop() {
                let url = checksum_url.ok_or_else(|| {
                    InstallError::source(format!(
                        "missing checksum: {} {}",
                        artifact.name, artifact.version
                    ))
                })?;
                let bytes = fetch(&url)?;
                artifact.sha256 = std::str::from_utf8(&bytes)
                    .map_err(|e| InstallError::source(e.to_string()))?
                    .split_whitespace()
                    .next()
                    .ok_or_else(|| InstallError::source("empty checksum"))?
                    .to_owned();
                programs.push(artifact);
            } else if name == "reach" {
                return Err(InstallError::source(
                    "no published Reach archive matches the selected version",
                ));
            }
        }
        if let Some(skills) = skills_archive {
            return Ok(ReleaseSet {
                programs,
                skills: skills.clone(),
                contract_sha256: CONTRACT_SHA256.into(),
            });
        }
        let skills_releases = self.releases("sno-station-skills", &fetch)?;
        let release = skills_releases
            .iter()
            .find(|r| {
                r["draft"].as_bool() == Some(false)
                    && r["prerelease"].as_bool() == Some(false)
                    && skills_version.is_none_or(|pin| r["tag_name"].as_str() == Some(pin))
            })
            .ok_or_else(|| {
                InstallError::source("no published skills release matches the selected tag")
            })?;
        Ok(ReleaseSet {
            programs,
            skills: skills_release_artifact(release, &fetch)?,
            contract_sha256: CONTRACT_SHA256.into(),
        })
    }
}
impl ReleaseSource for GithubSource {
    fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        let parsed = url::Url::parse(url).map_err(|e| InstallError::source(e.to_string()))?;
        if parsed.scheme() != "https" {
            return Err(InstallError::source(
                "production release URLs require HTTPS",
            ));
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| InstallError::source(e.to_string()))?;
        let mut request = client.get(url).header("User-Agent", "sno-installer");
        if parsed.host_str() == Some("api.github.com") {
            if let Ok(token) = env::var("GH_TOKEN") {
                request = request.bearer_auth(token);
            }
            if parsed.path().contains("/releases/assets/") {
                request = request.header("Accept", "application/octet-stream");
            }
        }
        let response = request
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| InstallError::source(format!("{url}: {e}")))?;
        let mut bytes = Vec::new();
        response
            .take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| InstallError::source(e.to_string()))?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(InstallError::source("release exceeds download limit"));
        }
        Ok(bytes)
    }
    fn resolve(
        &self,
        reach_version: Option<&str>,
        skills_version: Option<&str>,
        skills_archive: Option<&Artifact>,
    ) -> Result<ReleaseSet> {
        self.resolve_for_platform(
            reach_version,
            skills_version,
            env::consts::OS,
            env::consts::ARCH,
            skills_archive,
            |url| self.fetch(url),
        )
    }
}

#[derive(Debug, Serialize)]
struct Row {
    target: String,
    result: String,
    detail: String,
}
fn row(target: impl Into<String>, result: impl Into<String>, detail: impl Into<String>) -> Row {
    Row {
        target: target.into(),
        result: result.into(),
        detail: detail.into(),
    }
}
struct Harness {
    name: String,
    root: PathBuf,
    config: PathBuf,
}
struct Environment {
    home: PathBuf,
    control: PathBuf,
    harnesses: Vec<Harness>,
    path: String,
}
fn canonical(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|e| InstallError::path(path, e));
    }
    let parent = path
        .parent()
        .ok_or_else(|| InstallError::path(path, "cannot resolve root"))?;
    Ok(canonical(parent)?.join(
        path.file_name()
            .ok_or_else(|| InstallError::path(path, "no filename"))?,
    ))
}
fn executable(name: &str, path: &str) -> Option<PathBuf> {
    env::split_paths(path).map(|p| p.join(name)).find(|p| {
        let Ok(m) = fs::metadata(p) else {
            return false;
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            m.is_file() && m.permissions().mode() & 0o111 != 0
        }
        #[cfg(not(unix))]
        {
            m.is_file()
        }
    })
}
impl Environment {
    fn load() -> Result<Self> {
        let raw = env::var_os("HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| InstallError::state("HOME is missing"))?;
        if !raw.is_absolute() {
            return Err(InstallError::state("HOME must be absolute"));
        }
        let home = canonical(&raw)?;
        let path = env::var("PATH").unwrap_or_default();
        let mut harnesses = Vec::new();
        for (name, override_name, default, config) in [
            ("claude", "CLAUDE_CONFIG_DIR", ".claude", "settings.json"),
            ("codex", "CODEX_HOME", ".codex", "hooks.json"),
            ("hermes", "HERMES_HOME", ".hermes", "config.yaml"),
            (
                "openclaw",
                "OPENCLAW_STATE_DIR",
                ".openclaw",
                "openclaw.json",
            ),
        ] {
            let base = env::var_os(override_name)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(default));
            if !base.is_absolute() {
                return Err(InstallError::state(format!(
                    "{override_name} must be absolute"
                )));
            }
            let root = base.join("skills");
            if root.is_dir() || executable(name, &path).is_some() {
                harnesses.push(Harness {
                    name: name.into(),
                    root: canonical(&root)?,
                    config: canonical(&base)?.join(config),
                });
            }
        }
        let control = canonical(&home.join(".config/sno"))?;
        Ok(Self {
            home,
            control,
            harnesses,
            path,
        })
    }
    fn allowed(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.home.clone()];
        for h in &self.harnesses {
            roots.push(h.root.clone());
            if let Some(p) = h.config.parent() {
                roots.push(p.to_path_buf());
            }
        }
        roots
    }
}

fn checksum(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn safe_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(InstallError::source(format!(
            "unsafe archive path: {}",
            path.display()
        )));
    }
    Ok(())
}
fn unpack(source: &dyn ReleaseSource, artifact: &Artifact) -> Result<BTreeMap<PathBuf, Snapshot>> {
    let bytes = source.fetch(&artifact.url)?;
    unpack_bytes(bytes, artifact)
}

fn unpack_bytes(bytes: Vec<u8>, artifact: &Artifact) -> Result<BTreeMap<PathBuf, Snapshot>> {
    if checksum(&bytes) != artifact.sha256 {
        return Err(InstallError::source(format!(
            "checksum mismatch: {}",
            artifact.url
        )));
    }
    let gzip = flate2::read::GzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(gzip);
    let mut files = BTreeMap::new();
    let mut size = 0u64;
    for item in archive
        .entries()
        .map_err(|e| InstallError::source(e.to_string()))?
    {
        let item = item.map_err(|e| InstallError::source(e.to_string()))?;
        let path = item
            .path()
            .map_err(|e| InstallError::source(e.to_string()))?
            .into_owned();
        safe_relative(&path)?;
        let path: PathBuf = path
            .components()
            .filter(|c| !matches!(c, Component::CurDir))
            .collect();
        let kind = item.header().entry_type();
        if kind.is_dir() {
            continue;
        }
        if files.contains_key(&path) {
            return Err(InstallError::source("duplicate archive file"));
        }
        let data = if kind.is_file() {
            size = size
                .checked_add(item.size())
                .ok_or_else(|| InstallError::source("archive size overflow"))?;
            if size > MAX_BYTES {
                return Err(InstallError::source("archive exceeds unpack limit"));
            }
            let mode = item
                .header()
                .mode()
                .map_err(|e| InstallError::source(e.to_string()))?
                & 0o777;
            let mut bytes = Vec::new();
            item.take(MAX_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| InstallError::source(e.to_string()))?;
            Snapshot::file(&bytes, mode)
        } else if kind.is_symlink() {
            let target = item
                .link_name()
                .map_err(|e| InstallError::source(e.to_string()))?
                .ok_or_else(|| InstallError::source("empty archive link"))?
                .into_owned();
            safe_relative(&target)?;
            Snapshot::Link { target }
        } else {
            return Err(InstallError::source("unsupported archive entry type"));
        };
        files.insert(path, data);
    }
    for path in files.keys() {
        let mut parent = path.parent();
        while let Some(p) = parent {
            if files.contains_key(p) {
                return Err(InstallError::source("archive file/link used as directory"));
            }
            parent = p.parent();
        }
    }
    Ok(files)
}

struct Plan {
    old: Manifest,
    next: Manifest,
    ops: BTreeMap<PathBuf, Operation>,
    rows: Vec<Row>,
}
impl Plan {
    fn new(old: Option<Manifest>) -> Self {
        let old = old.unwrap_or_default();
        Self {
            next: old.clone(),
            old,
            ops: BTreeMap::new(),
            rows: Vec::new(),
        }
    }
    fn own(&mut self, path: PathBuf, data: Snapshot) -> Result<()> {
        let actual = manifest::read(&path)?;
        match self.old.files.get(&path) {
            Some(previous) if actual.as_ref().is_some_and(|a| a != previous) => {
                return Err(InstallError::state(format!(
                    "owned file changed: {}",
                    path.display()
                )));
            }
            None if actual.is_some() => {
                return Err(InstallError::state(format!(
                    "unowned destination: {}",
                    path.display()
                )));
            }
            _ => {}
        }
        if actual.as_ref() != Some(&data) {
            self.ops.insert(
                path.clone(),
                Operation {
                    path: path.clone(),
                    before: actual,
                    after: Some(data.clone()),
                },
            );
        }
        self.next.files.insert(path, data);
        Ok(())
    }
    fn prune(&mut self, prefix: &Path, keep: &std::collections::BTreeSet<PathBuf>) -> Result<()> {
        let obsolete: Vec<_> = self
            .old
            .files
            .keys()
            .filter(|p| p.starts_with(prefix) && !keep.contains(*p))
            .cloned()
            .collect();
        for path in obsolete {
            let actual = manifest::read(&path)?;
            if actual
                .as_ref()
                .is_some_and(|v| Some(v) != self.old.files.get(&path))
            {
                return Err(InstallError::state(format!(
                    "owned file changed: {}",
                    path.display()
                )));
            }
            self.edit(path.clone(), None)?;
            self.next.files.remove(&path);
        }
        Ok(())
    }
    fn edit(&mut self, path: PathBuf, data: Option<Snapshot>) -> Result<()> {
        let before = manifest::read(&path)?;
        if before != data {
            self.ops.insert(
                path.clone(),
                Operation {
                    path,
                    before,
                    after: data,
                },
            );
        }
        Ok(())
    }
}

fn agents(environment: &Environment, plan: &mut Plan) -> Result<()> {
    let path = environment.home.join(".config/sno-reach/agents.json");
    if let Some(existing) = manifest::read(&path)? {
        validate_agents(&existing.bytes()?)?;
        return Ok(());
    }
    let mut map = BTreeMap::new();
    for h in &environment.harnesses {
        let adapter = match h.name.as_str() {
            "claude" => "claude",
            "codex" => "codex",
            "hermes" => "hermes",
            "openclaw" => "openclaw",
            _ => continue,
        };
        map.insert(&h.name, json!({"acpx_agent":adapter}));
    }
    plan.own(
        path,
        Snapshot::file(
            &serde_json::to_vec_pretty(&map).map_err(|e| InstallError::state(e.to_string()))?,
            0o600,
        ),
    )
}
fn validate_agents(bytes: &[u8]) -> Result<Value> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|e| InstallError::state(format!("agents.json: {e}")))?;
    let map = value
        .as_object()
        .ok_or_else(|| InstallError::state("agents.json must be an object"))?;
    for (kind, entry) in map {
        let agent = entry
            .get("acpx_agent")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                InstallError::state(format!("agents.json {kind}: missing acpx_agent"))
            })?;
        if agent.is_empty()
            || !agent
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
            || !agent.as_bytes()[0].is_ascii_alphanumeric()
        {
            return Err(InstallError::state(format!(
                "agents.json {kind}: invalid adapter"
            )));
        }
    }
    Ok(value)
}

fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}
fn hook_entry(environment: &Environment) -> Result<String> {
    let sno = env::current_exe().map_err(|e| InstallError::state(e.to_string()))?;
    let sno = sno
        .to_str()
        .ok_or_else(|| InstallError::state("sno executable path is not UTF-8"))?;
    let jq = executable("jq", &environment.path).unwrap_or_else(|| PathBuf::from("jq"));
    let timeout = executable("timeout", &environment.path)
        .or_else(|| executable("gtimeout", &environment.path))
        .unwrap_or_else(|| PathBuf::from("timeout"));
    let command = format!(
        "[ -n \"${{SNO_REACH_ADDR:-}}\" ] || exit 0; text=$({} -s KILL 5 {} reach remind --as \"$SNO_REACH_ADDR\"); rc=$?; if [ \"$rc\" -ne 0 ]; then echo 'sno reminder failed or timed out (check timeout and Reach dependencies)' >&2; exit 0; fi; [ -n \"$text\" ] || exit 0; {} -nc --arg text \"$text\" '{{hookSpecificOutput:{{hookEventName:\"UserPromptSubmit\",additionalContext:$text}}}}'",
        quote(&timeout.to_string_lossy()),
        quote(sno),
        quote(&jq.to_string_lossy())
    );
    serde_json::to_string(&json!({"hooks":[{"type":"command","command":command}]}))
        .map_err(|e| InstallError::state(e.to_string()))
}
type RawMap = BTreeMap<String, Box<RawValue>>;
fn raw<T: Serialize>(value: &T) -> Result<Box<RawValue>> {
    serde_json::value::to_raw_value(value).map_err(|e| InstallError::state(e.to_string()))
}
fn hook_merge(bytes: &[u8], entry: &str, remove: bool) -> Result<(Vec<u8>, bool)> {
    let mut root: RawMap = serde_json::from_slice(bytes)
        .map_err(|e| InstallError::state(format!("hook config: {e}")))?;
    let mut hooks: RawMap = match root.get("hooks") {
        Some(r) => {
            serde_json::from_str(r.get()).map_err(|e| InstallError::state(format!("hooks: {e}")))?
        }
        None => BTreeMap::new(),
    };
    let mut entries: Vec<Box<RawValue>> = match hooks.get("UserPromptSubmit") {
        Some(r) => serde_json::from_str(r.get())
            .map_err(|e| InstallError::state(format!("UserPromptSubmit: {e}")))?,
        None => Vec::new(),
    };
    let wanted: Value =
        serde_json::from_str(entry).map_err(|e| InstallError::state(e.to_string()))?;
    let mut found = false;
    let mut kept = Vec::new();
    for existing in entries.drain(..) {
        let value: Value =
            serde_json::from_str(existing.get()).map_err(|e| InstallError::state(e.to_string()))?;
        if value == wanted {
            if found {
                return Err(InstallError::state("duplicate reminder hook"));
            }
            found = true;
            if !remove {
                kept.push(existing);
            }
        } else {
            if !remove && existing.get().contains("reach remind") {
                return Err(InstallError::state("conflicting user reminder hook"));
            }
            kept.push(existing);
        }
    }
    if (remove && !found) || (!remove && found) {
        return Ok((bytes.to_vec(), found));
    }
    if !remove {
        kept.push(
            RawValue::from_string(entry.into()).map_err(|e| InstallError::state(e.to_string()))?,
        );
    }
    hooks.insert("UserPromptSubmit".into(), raw(&kept)?);
    root.insert("hooks".into(), raw(&hooks)?);
    Ok((
        serde_json::to_vec(&root).map_err(|e| InstallError::state(e.to_string()))?,
        found,
    ))
}
fn codex_trusted(hook: &Hook) -> Result<bool> {
    let config_path = hook
        .path
        .parent()
        .ok_or_else(|| InstallError::state("hook config has no parent"))?
        .join("config.toml");
    let Some(snapshot) = manifest::read(&config_path)? else {
        return Ok(false);
    };
    let bytes = snapshot.bytes()?;
    let config: toml::Value = std::str::from_utf8(&bytes)
        .map_err(|e| InstallError::path(&config_path, e))?
        .parse()
        .map_err(|e| InstallError::path(&config_path, e))?;
    if config
        .get("features")
        .and_then(|v| v.get("hooks"))
        .and_then(toml::Value::as_bool)
        == Some(false)
    {
        return Ok(false);
    }
    let Some(snapshot) = manifest::read(&hook.path)? else {
        return Ok(false);
    };
    let root: Value = serde_json::from_slice(&snapshot.bytes()?)
        .map_err(|e| InstallError::path(&hook.path, e))?;
    let entry: Value =
        serde_json::from_str(&hook.entry).map_err(|e| InstallError::state(e.to_string()))?;
    let Some(index) = root["hooks"]["UserPromptSubmit"]
        .as_array()
        .and_then(|items| items.iter().position(|v| *v == entry))
    else {
        return Ok(false);
    };
    let command = entry["hooks"][0]["command"]
        .as_str()
        .ok_or_else(|| InstallError::state("invalid owned hook command"))?;
    // Codex hashes canonical normalized configuration, not command bytes or raw JSON.
    let handler = BTreeMap::from([
        ("async", json!(false)),
        ("command", json!(command)),
        ("timeout", json!(600)),
        ("type", json!("command")),
    ]);
    let identity = BTreeMap::from([
        ("event_name", json!("user_prompt_submit")),
        ("hooks", json!([handler])),
    ]);
    let hash = format!(
        "sha256:{}",
        checksum(&serde_json::to_vec(&identity).map_err(|e| InstallError::state(e.to_string()))?)
    );
    let key = format!("{}:user_prompt_submit:{index}:0", hook.path.display());
    let state = config
        .get("hooks")
        .and_then(|v| v.get("state"))
        .and_then(|v| v.get(&key));
    Ok(state
        .and_then(|v| v.get("trusted_hash"))
        .and_then(toml::Value::as_str)
        == Some(&hash)
        && state
            .and_then(|v| v.get("enabled"))
            .and_then(toml::Value::as_bool)
            != Some(false))
}

fn hooks(environment: &Environment, plan: &mut Plan) -> Result<()> {
    let entry = hook_entry(environment)?;
    for h in &environment.harnesses {
        if !matches!(h.name.as_str(), "claude" | "codex") {
            continue;
        }
        let current = manifest::read(&h.config)?;
        let bytes = match &current {
            Some(x) => x.bytes()?,
            None => b"{}".to_vec(),
        };
        let prior = plan.old.hooks.iter().find(|x| x.path == h.config);
        let replacing = prior.is_some_and(|x| x.owned && x.entry != entry);
        let merge_bytes = if replacing {
            let prior = prior.expect("replacing owned hook");
            let (without, found) = hook_merge(&bytes, &prior.entry, true)?;
            if !found {
                return Err(InstallError::path(&h.config, "owned hook changed"));
            }
            without
        } else {
            bytes.clone()
        };
        let (next, found) = hook_merge(&merge_bytes, &entry, false)
            .map_err(|e| InstallError::path(&h.config, e))?;
        let owned = prior.is_some_and(|x| x.owned) || !found;
        if next != bytes {
            let mode = match current {
                Some(Snapshot::File { mode, .. }) => mode,
                _ => 0o600,
            };
            plan.edit(h.config.clone(), Some(Snapshot::file(&next, mode)))?;
        }
        let record = Hook {
            path: h.config.clone(),
            entry: entry.clone(),
            owned,
            harness: h.name.clone(),
        };
        if let Some(previous) = plan.next.hooks.iter_mut().find(|x| x.path == h.config) {
            *previous = record;
        } else {
            plan.next.hooks.push(record);
        }
        let trust_missing = h.name == "codex"
            && !codex_trusted(&Hook {
                path: h.config.clone(),
                entry: entry.clone(),
                owned,
                harness: h.name.clone(),
            })?;
        plan.rows.push(row(
            format!("hooks {}", h.name),
            if replacing {
                "updated"
            } else if found {
                "current"
            } else {
                "installed"
            },
            if trust_missing {
                "open Codex and press t to trust the new hook"
            } else {
                ""
            },
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgramMetadata {
    program: String,
    version: String,
    requirements_contract_sha256: String,
    dependencies: Vec<String>,
}
fn program_metadata(
    artifact: &Artifact,
    files: &BTreeMap<PathBuf, Snapshot>,
) -> Result<Vec<String>> {
    if artifact.name == "reach" {
        return Ok(Vec::new());
    }
    let bytes = files
        .get(Path::new("release.json"))
        .ok_or_else(|| InstallError::source(format!("{} missing release.json", artifact.name)))?
        .bytes()?;
    let metadata: ProgramMetadata = serde_json::from_slice(&bytes)
        .map_err(|e| InstallError::source(format!("release.json: {e}")))?;
    let contract = files
        .get(Path::new("requirements-contract.json"))
        .ok_or_else(|| InstallError::source("utility missing requirements-contract.json"))?
        .bytes()?;
    if metadata.program != artifact.name
        || metadata.version != artifact.version
        || metadata.requirements_contract_sha256 != CONTRACT_SHA256
        || checksum(&contract) != CONTRACT_SHA256
    {
        return Err(InstallError::source(
            "utility release manifest identity/contract mismatch",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for dependency in &metadata.dependencies {
        if dependency.is_empty()
            || !dependency
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !seen.insert(dependency)
        {
            return Err(InstallError::source(
                "utility manifest has invalid/duplicate dependency",
            ));
        }
    }
    Ok(metadata.dependencies)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclarationFixture {
    schema_version: u32,
    contract_sha256: String,
    source: FixtureSource,
    units: Vec<FixtureUnit>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSource {
    repo: String,
    commit: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureUnit {
    unit: String,
    skill_path: String,
    skill_sha256: String,
    requires: Requirements,
}
fn validate_fixture(
    files: &BTreeMap<PathBuf, Snapshot>,
    requirements: &BTreeMap<&str, Requirements>,
) -> Result<String> {
    let path = Path::new("scripts/fixtures/s-category-requirements.json");
    let bytes = files
        .get(path)
        .ok_or_else(|| InstallError::source("missing shared declaration fixture"))?
        .bytes()?;
    let fixture: DeclarationFixture = serde_json::from_slice(&bytes)
        .map_err(|e| InstallError::source(format!("declaration fixture: {e}")))?;
    if fixture.schema_version != 1
        || fixture.contract_sha256 != CONTRACT_SHA256
        || fixture.source.repo != "sno-skills"
        || fixture.source.commit.len() != 40
        || !fixture.source.commit.bytes().all(|b| b.is_ascii_hexdigit())
        || fixture.units.len() != 4
    {
        return Err(InstallError::source(
            "invalid shared declaration fixture identity",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for unit in fixture.units {
        let expected = format!("{SKILLS}/{}/skill/SKILL.md", unit.unit);
        if !seen.insert(unit.unit.clone()) || unit.skill_path != expected {
            return Err(InstallError::source(
                "duplicate unit or wrong fixture skill_path",
            ));
        }
        let requirement = requirements
            .get(unit.unit.as_str())
            .ok_or_else(|| InstallError::source("unknown fixture unit"))?;
        let skill = files
            .get(Path::new(&expected))
            .ok_or_else(|| InstallError::source("fixture skill missing"))?
            .bytes()?;
        if checksum(&skill) != unit.skill_sha256 || &unit.requires != requirement {
            return Err(InstallError::source(format!(
                "fixture skill hash/declaration mismatch: {}",
                unit.unit
            )));
        }
    }
    Ok(checksum(&bytes))
}

fn install(
    environment: &Environment,
    options: &InstallOptions,
    source: &dyn ReleaseSource,
    verb: &str,
) -> Result<Vec<Row>> {
    let store = Store::open(environment.control.clone(), environment.allowed())?;
    store.recover()?;
    if let Some(v) = &options.reach_version {
        version(v)?;
    }
    let local_skills = options
        .skills_archive
        .as_ref()
        .map(|path| {
            let path = fs::canonicalize(path).map_err(|e| InstallError::path(path, e))?;
            let mut bytes = Vec::new();
            fs::File::open(&path)
                .map_err(|e| InstallError::path(&path, e))?
                .take(MAX_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| InstallError::path(&path, e))?;
            if bytes.len() as u64 > MAX_BYTES {
                return Err(InstallError::source("release exceeds download limit"));
            }
            let sha256 = checksum(&bytes);
            let artifact = Artifact {
                name: "skills".into(),
                version: format!("bootstrap-{sha256}"),
                url: url::Url::from_file_path(&path)
                    .map_err(|_| InstallError::source("invalid skills archive path"))?
                    .to_string(),
                sha256,
                entry_point: String::new(),
            };
            Ok((artifact, bytes))
        })
        .transpose()?;
    let releases = source.resolve(
        options.reach_version.as_deref(),
        options.skills_version.as_deref(),
        local_skills.as_ref().map(|(artifact, _)| artifact),
    )?;
    if releases.contract_sha256 != CONTRACT_SHA256 {
        return Err(InstallError::source(
            "unsupported requirements contract hash",
        ));
    }
    let mut skill_files = if let Some((_, bytes)) = local_skills {
        unpack_bytes(bytes, &releases.skills)?
    } else {
        unpack(source, &releases.skills)?
    };
    if !skill_files.contains_key(Path::new("scripts/requirements-contract.json")) {
        let prefixes: std::collections::BTreeSet<_> = skill_files
            .keys()
            .filter_map(|p| p.components().next().map(|c| c.as_os_str().to_owned()))
            .collect();
        if prefixes.len() == 1 {
            let prefix = prefixes.iter().next().expect("one archive prefix");
            skill_files = skill_files
                .into_iter()
                .map(|(p, v)| {
                    p.strip_prefix(prefix)
                        .map(|rel| (rel.to_path_buf(), v))
                        .map_err(|e| InstallError::source(e.to_string()))
                })
                .collect::<Result<_>>()?;
        }
    }
    let contract_bytes = skill_files
        .get(Path::new("scripts/requirements-contract.json"))
        .ok_or_else(|| InstallError::source("skills release missing requirements-contract.json"))?
        .bytes()?;
    if checksum(&contract_bytes) != CONTRACT_SHA256 {
        return Err(InstallError::source(
            "requirements contract digest mismatch",
        ));
    }
    let contract = Contract::parse(&contract_bytes)?;
    let mut requirements = BTreeMap::new();
    for unit in ["reach", "handoff", "heartbeat", "subscription-quota-check"] {
        let path = PathBuf::from(SKILLS).join(unit).join("skill/SKILL.md");
        let bytes = skill_files
            .get(&path)
            .ok_or_else(|| InstallError::source(format!("missing {}", path.display())))?
            .bytes()?;
        requirements.insert(unit, Requirements::from_skill(&bytes, &contract)?);
        let stamp = PathBuf::from(SKILLS).join(unit).join("PROMOTED.json");
        let bytes = skill_files
            .get(&stamp)
            .ok_or_else(|| InstallError::source(format!("missing {}", stamp.display())))?
            .bytes()?;
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|e| InstallError::source(e.to_string()))?;
        if value["unit"].as_str() != Some(unit) || value["selftests_run"].as_u64().unwrap_or(0) == 0
        {
            return Err(InstallError::source(format!(
                "invalid promotion stamp: {unit}"
            )));
        }
    }
    // Validate overlays too, even when their consumer is absent.
    for (path, data) in &skill_files {
        if path.starts_with(Path::new(SKILLS)) && path.file_name().is_some_and(|n| n == "SKILL.md")
        {
            Requirements::from_skill(&data.bytes()?, &contract)?;
        }
    }
    let requirements_fixture_sha256 = validate_fixture(&skill_files, &requirements)?;
    let mut archives = BTreeMap::new();
    let mut program_dependencies = BTreeMap::new();
    for artifact in &releases.programs {
        if !PROGRAM_IDS.contains(&artifact.name.as_str()) || archives.contains_key(&artifact.name) {
            return Err(InstallError::source("unknown or duplicate program"));
        }
        version(&artifact.version)?;
        safe_relative(Path::new(&artifact.entry_point))?;
        let files = unpack(source, artifact)?;
        let installed_version = files
            .get(Path::new("VERSION"))
            .ok_or_else(|| InstallError::source(format!("{} missing VERSION", artifact.name)))?
            .bytes()?;
        if std::str::from_utf8(&installed_version)
            .map_err(|e| InstallError::source(e.to_string()))?
            .trim()
            != artifact.version
        {
            return Err(InstallError::source("archive VERSION differs from release"));
        }
        let entry = files.get(Path::new(&artifact.entry_point)).ok_or_else(|| {
            InstallError::source(format!("{} missing entry point", artifact.name))
        })?;
        if !matches!(entry,Snapshot::File{mode,..} if mode & 0o111 !=0) {
            return Err(InstallError::source(
                "program entry point is not executable",
            ));
        }
        program_dependencies.insert(artifact.name.clone(), program_metadata(artifact, &files)?);
        archives.insert(artifact.name.clone(), files);
    }
    if !archives.contains_key("reach") {
        return Err(InstallError::source(
            "selected release has no Reach program",
        ));
    }
    let mut plan = Plan::new(Store::load(&environment.control)?);
    for artifact in &releases.programs {
        let root = environment.home.join(format!(
            ".local/lib/sno-{}/releases/{}",
            artifact.name, artifact.version
        ));
        let retained = archives[&artifact.name]
            .keys()
            .map(|rel| root.join(rel))
            .collect();
        plan.prune(
            &environment
                .home
                .join(format!(".local/lib/sno-{}/releases", artifact.name)),
            &retained,
        )?;
        for (rel, data) in &archives[&artifact.name] {
            plan.own(root.join(rel), data.clone())?;
        }
        let stable = environment
            .home
            .join(format!(".local/lib/sno-{}/current", artifact.name));
        plan.own(stable.clone(), Snapshot::Link { target: root })?;
        let command = if artifact.name == "reach" {
            "sno-reach"
        } else {
            artifact.name.as_str()
        };
        let entry = environment.home.join(".local/bin").join(command);
        plan.own(
            entry.clone(),
            Snapshot::Link {
                target: stable.join(&artifact.entry_point),
            },
        )?;
        let prior = plan.old.programs.get(&artifact.name);
        let result = match prior {
            Some(p) if p.version == artifact.version && p.sha256 == artifact.sha256 => "current",
            Some(_) => "updated",
            None => "installed",
        };
        let detail = if let Some(p) = prior.filter(|_| result == "updated") {
            format!("{} -> {}", p.version, artifact.version)
        } else {
            artifact.version.clone()
        };
        plan.rows.push(row(&artifact.name, result, detail));
        plan.next.programs.insert(
            artifact.name.clone(),
            Program {
                version: artifact.version.clone(),
                sha256: artifact.sha256.clone(),
                entry,
                dependencies: program_dependencies
                    .remove(&artifact.name)
                    .unwrap_or_default(),
            },
        );
    }
    let mut installed = BTreeMap::new();
    for (name, program) in &plan.next.programs {
        let entry = if let Some(op) = plan.ops.get(&program.entry) {
            op.after.clone()
        } else {
            manifest::read(&program.entry)?
        };
        if entry.as_ref() == plan.next.files.get(&program.entry) && entry.is_some() {
            installed.insert(name.clone(), program.version.clone());
        }
    }
    let mut missing_runtime = BTreeMap::new();
    for (name, program) in &plan.next.programs {
        for dependency in &program.dependencies {
            let available = if PROGRAM_IDS.contains(&dependency.as_str()) {
                installed.contains_key(dependency)
            } else {
                executable(dependency, &environment.path).is_some()
            };
            if !available {
                missing_runtime.insert(
                    name.clone(),
                    format!("{name} dependency {dependency} missing"),
                );
                break;
            }
        }
    }
    let mut destinations: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for h in &environment.harnesses {
        destinations
            .entry(h.root.clone())
            .or_default()
            .push(h.name.clone());
    }
    for (unit, requires) in requirements {
        for (root, consumers) in &destinations {
            let (result, detail) = if let Some(detail) = requires
                .programs
                .iter()
                .find_map(|p| missing_runtime.get(&p.name))
            {
                ("skipped", detail.clone())
            } else {
                requires.decide(consumers, &installed)?
            };
            let target = format!("skill {unit} {}", consumers.join(","));
            let dest = root.join(unit);
            if result == "skipped" {
                plan.prune(&dest, &std::collections::BTreeSet::new())?;
                plan.next.skill_destinations.remove(&dest);
                plan.rows.push(row(target, result, detail));
                continue;
            }
            if dest.exists() && !plan.old.files.keys().any(|p| p.starts_with(&dest)) {
                return Err(InstallError::state(format!(
                    "unowned destination: {}",
                    dest.display()
                )));
            }
            plan.next
                .skill_destinations
                .insert(dest.clone(), consumers.clone());
            let prefix = PathBuf::from(SKILLS).join(unit).join("skill");
            let before = plan.ops.len();
            let mut keep = std::collections::BTreeSet::new();
            for (path, data) in &skill_files {
                if let Ok(rel) = path.strip_prefix(&prefix) {
                    keep.insert(dest.join(rel));
                    plan.own(dest.join(rel), data.clone())?;
                }
            }
            keep.insert(dest.join("PROMOTED.json"));
            plan.prune(&dest, &keep)?;
            let stamp = PathBuf::from(SKILLS).join(unit).join("PROMOTED.json");
            plan.own(dest.join("PROMOTED.json"), skill_files[&stamp].clone())?;
            let result = if result == "degraded" {
                "degraded"
            } else if before == plan.ops.len() {
                "current"
            } else if !plan.old.skills_version.is_empty() {
                "updated"
            } else {
                "installed"
            };
            plan.rows.push(row(target, result, detail));
        }
        for name in ["claude", "codex", "hermes", "openclaw"] {
            if !environment.harnesses.iter().any(|h| h.name == name) {
                plan.rows.push(row(
                    format!("skill {unit} {name}"),
                    "skipped",
                    "harness not present",
                ));
            }
        }
    }
    agents(environment, &mut plan)?;
    hooks(environment, &mut plan)?;
    plan.next.skills_version = releases.skills.version;
    plan.next.contract_sha256 = CONTRACT_SHA256.into();
    plan.next.requirements_fixture_sha256 = requirements_fixture_sha256;
    let changed = plan.next != plan.old || !plan.ops.is_empty();
    if changed {
        let mut ops: Vec<_> = plan.ops.into_values().collect();
        ops.sort_by_key(|op| {
            if op.path.starts_with(environment.home.join(".local/lib")) {
                0
            } else if op.path.starts_with(environment.home.join(".local/bin")) {
                1
            } else {
                2
            }
        });
        store.commit(ops, Some(plan.next), &|phase| source.checkpoint(phase))?;
    }
    let _ = verb;
    Ok(plan.rows)
}

fn remove(environment: &Environment, purge: bool, source: &dyn ReleaseSource) -> Result<Vec<Row>> {
    let store = Store::open(environment.control.clone(), environment.allowed())?;
    store.recover()?;
    let purge_record = environment.control.join("assemble.purge.json");
    let pending_purge = match manifest::read(&purge_record)? {
        None => false,
        Some(snapshot) => serde_json::from_slice::<bool>(&snapshot.bytes()?)
            .map_err(|e| InstallError::path(&purge_record, e))?,
    };
    let Some(old) = Store::load(&environment.control)? else {
        if pending_purge {
            finish_purge(environment)?;
            return Ok(vec![row(
                "reach state",
                "removed",
                "resumed explicit purge",
            )]);
        }
        return Err(InstallError::usage("nothing assembled on this machine"));
    };
    let effect = if old.timer {
        Some(timer_effect(environment, false)?)
    } else {
        None
    };
    let mut ops = Vec::new();
    let mut rows = Vec::new();
    for (path, previous) in &old.files {
        let actual = manifest::read(path)?;
        if actual.as_ref().is_some_and(|v| v != previous) {
            return Err(InstallError::state(format!(
                "owned file changed: {}",
                path.display()
            )));
        }
        if actual.is_some() {
            ops.push(Operation {
                path: path.clone(),
                before: actual,
                after: None,
            });
            rows.push(row(path.to_string_lossy(), "removed", ""));
        }
    }
    for hook in &old.hooks {
        if !hook.owned {
            continue;
        }
        let Some(snapshot) = manifest::read(&hook.path)? else {
            continue;
        };
        let bytes = snapshot.bytes()?;
        let (next, found) = hook_merge(&bytes, &hook.entry, true)?;
        if !found {
            return Err(InstallError::state(format!(
                "owned hook changed: {}",
                hook.path.display()
            )));
        }
        let mode = match snapshot {
            Snapshot::File { mode, .. } => mode,
            _ => 0o600,
        };
        ops.push(Operation {
            path: hook.path.clone(),
            before: Some(snapshot),
            after: Some(Snapshot::file(&next, mode)),
        });
        rows.push(row(
            hook.path.to_string_lossy(),
            "removed",
            "reminder entry",
        ));
    }
    if purge {
        manifest::atomic(&purge_record, b"true", 0o600)?;
    }
    store.commit_effect(ops, None, &|phase| source.checkpoint(phase), effect)?;
    if purge || pending_purge {
        source.checkpoint("before-purge")?;
        finish_purge(environment)?;
        rows.push(row("reach state", "removed", "explicit purge"));
    }
    Ok(rows)
}

fn finish_purge(environment: &Environment) -> Result<()> {
    let path = environment.home.join(".local/state/sno-reach");
    if canonical(
        path.parent()
            .ok_or_else(|| InstallError::state("state has no parent"))?,
    )? != path
        .parent()
        .ok_or_else(|| InstallError::state("state has no parent"))?
    {
        return Err(InstallError::state(
            "state parent is a symlink; purge refused",
        ));
    }
    // remove_dir_all does not follow the root link; never traverse a user-selected path.
    match fs::symlink_metadata(&path) {
        Ok(m) if m.file_type().is_symlink() => {
            fs::remove_file(&path).map_err(|e| InstallError::path(&path, e))?
        }
        Ok(_) => fs::remove_dir_all(&path).map_err(|e| InstallError::path(&path, e))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(InstallError::path(&path, e)),
    }
    let record = environment.control.join("assemble.purge.json");
    fs::remove_file(&record).map_err(|e| InstallError::path(&record, e))?;
    Ok(())
}

fn bounded(mut command: Command) -> Result<bool> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| InstallError::state(format!("start command: {e}")))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child
            .try_wait()
            .map_err(|e| InstallError::state(e.to_string()))?
        {
            Some(status) => return Ok(status.success()),
            None if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            None => {
                child
                    .kill()
                    .map_err(|e| InstallError::state(e.to_string()))?;
                child
                    .wait()
                    .map_err(|e| InstallError::state(e.to_string()))?;
                return Err(InstallError::state("command timed out"));
            }
        }
    }
}
fn scheduler(environment: &Environment, args: &[&str]) -> Result<()> {
    let name = if cfg!(target_os = "macos") {
        "launchctl"
    } else {
        "systemctl"
    };
    let path = executable(name, &environment.path)
        .ok_or_else(|| InstallError::usage("timer unsupported: no user scheduler"))?;
    let mut cmd = Command::new(path);
    cmd.args(args);
    if !bounded(cmd)? {
        return Err(InstallError::state(format!(
            "{name} {} failed",
            args.join(" ")
        )));
    }
    Ok(())
}
pub(crate) fn execute_specs(specs: &[manifest::CommandSpec]) -> Result<()> {
    for spec in specs {
        let mut command = Command::new(&spec.program);
        command.args(&spec.args).envs(&spec.environment);
        if !bounded(command)? {
            return Err(InstallError::state(format!(
                "{} {} failed",
                spec.program.display(),
                spec.args.join(" ")
            )));
        }
    }
    Ok(())
}
fn timer_effect(environment: &Environment, enable: bool) -> Result<manifest::Effect> {
    let name = if cfg!(target_os = "macos") {
        "launchctl"
    } else {
        "systemctl"
    };
    let program = executable(name, &environment.path)
        .ok_or_else(|| InstallError::usage("timer unsupported: user scheduler missing"))?;
    let spec = |args: Vec<String>| manifest::CommandSpec {
        program: program.clone(),
        args,
        environment: BTreeMap::new(),
    };
    if cfg!(target_os = "macos") {
        let file = environment
            .home
            .join("Library/LaunchAgents/ai.sno.update.plist")
            .to_string_lossy()
            .into_owned();
        let load = || spec(vec!["load".into(), file.clone()]);
        let unload = || spec(vec!["unload".into(), file.clone()]);
        Ok(if enable {
            manifest::Effect {
                before_files: false,
                apply: vec![load()],
                before_restore: vec![unload()],
                after_restore: vec![],
            }
        } else {
            manifest::Effect {
                before_files: true,
                apply: vec![unload()],
                before_restore: vec![],
                after_restore: vec![load()],
            }
        })
    } else {
        let reload = || spec(vec!["--user".into(), "daemon-reload".into()]);
        let on = || {
            spec(vec![
                "--user".into(),
                "enable".into(),
                "--now".into(),
                "sno-update.timer".into(),
            ])
        };
        let off = || {
            spec(vec![
                "--user".into(),
                "disable".into(),
                "--now".into(),
                "sno-update.timer".into(),
            ])
        };
        Ok(if enable {
            manifest::Effect {
                before_files: false,
                apply: vec![reload(), on()],
                before_restore: vec![off()],
                after_restore: vec![reload()],
            }
        } else {
            manifest::Effect {
                before_files: true,
                apply: vec![off()],
                before_restore: vec![],
                after_restore: vec![reload(), on()],
            }
        })
    }
}
fn timer(environment: &Environment, enable: bool, source: &dyn ReleaseSource) -> Result<Vec<Row>> {
    let store = Store::open(environment.control.clone(), environment.allowed())?;
    store.recover()?;
    let old = Store::load(&environment.control)?
        .ok_or_else(|| InstallError::usage("assemble before enabling automatic updates"))?;
    let mut plan = Plan::new(Some(old));
    if !plan.old.timer && !enable {
        return Ok(vec![row("timer", "disabled", "")]);
    }
    let mut effect = timer_effect(environment, enable)?;
    if enable && plan.old.timer {
        effect.before_restore.clear();
        effect.after_restore = effect.apply.clone();
    }
    let exe = env::current_exe().map_err(|e| InstallError::state(e.to_string()))?;
    let home = environment
        .home
        .to_str()
        .ok_or_else(|| InstallError::state("HOME not UTF-8"))?;
    let exe = exe
        .to_str()
        .ok_or_else(|| InstallError::state("sno path not UTF-8"))?;
    let path = format!(
        "{}:{}",
        environment.home.join(".local/bin").display(),
        environment.path
    );
    let mut variables = BTreeMap::from([
        ("HOME".to_owned(), home.to_owned()),
        ("PATH".to_owned(), path),
    ]);
    for name in [
        "CLAUDE_CONFIG_DIR",
        "CODEX_HOME",
        "HERMES_HOME",
        "OPENCLAW_STATE_DIR",
    ] {
        if let Ok(value) = env::var(name) {
            variables.insert(name.into(), value);
        }
    }
    let files = if cfg!(target_os = "macos") {
        fn xml(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
        }
        let vars = variables
            .iter()
            .map(|(k, v)| format!("<key>{}</key><string>{}</string>", xml(k), xml(v)))
            .collect::<String>();
        let text = format!(
            "<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>Label</key><string>ai.sno.update</string><key>ProgramArguments</key><array><string>{}</string><string>update</string><string>--quiet</string></array><key>EnvironmentVariables</key><dict>{}</dict><key>StartInterval</key><integer>86400</integer></dict></plist>",
            xml(exe),
            vars
        );
        vec![(
            environment
                .home
                .join("Library/LaunchAgents/ai.sno.update.plist"),
            text,
        )]
    } else if cfg!(target_os = "linux") {
        fn systemd(s: &str) -> String {
            format!(
                "\"{}\"",
                s.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('%', "%%")
                    .replace('\n', "\\n")
            )
        }
        let vars = variables
            .iter()
            .map(|(k, v)| format!("Environment={}\n", systemd(&format!("{k}={v}"))))
            .collect::<String>();
        let service = format!(
            "[Unit]\nDescription=SNO daily update\n[Service]\nType=oneshot\nExecStart={} update --quiet\n{}",
            systemd(exe),
            vars
        );
        vec![(environment.home.join(".config/systemd/user/sno-update.service"),service),(environment.home.join(".config/systemd/user/sno-update.timer"),"[Unit]\nDescription=SNO daily update\n[Timer]\nOnCalendar=daily\nPersistent=true\n[Install]\nWantedBy=timers.target\n".into())]
    } else {
        return Err(InstallError::usage("timer unsupported on this platform"));
    };
    if enable
        && plan.old.timer
        && files.iter().all(|(path, text)| {
            manifest::read(path).ok().flatten() == Some(Snapshot::file(text.as_bytes(), 0o600))
        })
    {
        let active = if cfg!(target_os = "linux") {
            scheduler(
                environment,
                &["--user", "is-active", "--quiet", "sno-update.timer"],
            )
            .is_ok()
        } else {
            scheduler(environment, &["list", "ai.sno.update"]).is_ok()
        };
        if active {
            return Ok(vec![row("timer", "current", "")]);
        }
    }
    if !enable {
        for (path, _) in &files {
            if let Some(previous) = plan.old.files.get(path) {
                let actual = manifest::read(path)?;
                if actual.as_ref().is_some_and(|v| v != previous) {
                    return Err(InstallError::state("timer file changed"));
                }
                plan.edit(path.clone(), None)?;
                plan.next.files.remove(path);
            }
        }
    } else {
        for (path, text) in &files {
            plan.own(path.clone(), Snapshot::file(text.as_bytes(), 0o600))?;
        }
    }
    plan.next.timer = enable;
    store.commit_effect(
        plan.ops.into_values().collect(),
        Some(plan.next),
        &|phase| source.checkpoint(phase),
        Some(effect),
    )?;
    Ok(vec![row(
        "timer",
        if enable { "installed" } else { "disabled" },
        "",
    )])
}

fn adapter_check(environment: &Environment, kind: &str, adapter: &str) -> Result<Row> {
    if executable("acpx", &environment.path).is_none() {
        return Ok(row(
            kind,
            "missing",
            "install acpx and configure the named ACP adapter",
        ));
    }
    let path = environment.home.join(".acpx/config.json");
    let config = match manifest::read(&path)? {
        Some(snapshot) => serde_json::from_slice::<Value>(&snapshot.bytes()?)
            .map_err(|e| InstallError::path(&path, e))?,
        None => json!({}),
    };
    if let Some(entry) = config.get("agents").and_then(|v| v.get(adapter)) {
        let command = entry
            .get("argv")
            .and_then(Value::as_array)
            .filter(|a| {
                !a.is_empty()
                    && a.iter()
                        .all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty()))
            })
            .and_then(|a| a[0].as_str());
        let Some(command) = command else {
            return Ok(row(
                kind,
                "fail",
                format!("fix {} agents.{adapter}.argv", path.display()),
            ));
        };
        let found = if Path::new(command).is_absolute() {
            let parent = Path::new(command)
                .parent()
                .and_then(Path::to_str)
                .unwrap_or("");
            Path::new(command)
                .file_name()
                .and_then(|v| v.to_str())
                .and_then(|name| executable(name, parent))
                .is_some()
        } else {
            executable(command, &environment.path).is_some()
        };
        return Ok(row(
            kind,
            if found { "warn" } else { "missing" },
            if found {
                "adapter command exists; live ACP handshake unverified".into()
            } else {
                format!("install {command}; configured in {}", path.display())
            },
        ));
    }
    if matches!(adapter, "claude" | "codex") {
        Ok(row(
            kind,
            "warn",
            "built-in acpx adapter selected; live ACP handshake unverified",
        ))
    } else {
        Ok(row(
            kind,
            "missing",
            format!("configure {} agents.{adapter}.argv", path.display()),
        ))
    }
}

fn diagnose(environment: &Environment) -> Result<(Value, i32)> {
    let mut sections: BTreeMap<&str, Vec<Row>> = ["reach", "skills", "hooks", "acp", "timer"]
        .into_iter()
        .map(|k| (k, Vec::new()))
        .collect();
    let mut failed = false;
    let pending = environment.control.join("assemble.pending.json").exists();
    if pending {
        sections.get_mut("reach").expect("section exists").push(row(
            "recovery",
            "fail",
            "run: sno update (pending transaction)",
        ));
        failed = true;
    }
    let loaded = Store::load(&environment.control);
    let manifest = match loaded {
        Ok(m) => m,
        Err(e) => {
            failed = true;
            sections
                .get_mut("reach")
                .expect("section exists")
                .push(row("manifest", "fail", e.message));
            None
        }
    };
    if let Some(m) = manifest {
        for (name, program) in &m.programs {
            let section = if name == "reach" { "reach" } else { "skills" };
            let mut good = true;
            for (path, expected) in &m.files {
                if (path == &program.entry
                    || path.to_string_lossy().contains(&format!("/sno-{name}/")))
                    && manifest::read(path).ok().flatten().as_ref() != Some(expected)
                {
                    good = false;
                }
            }
            for dependency in &program.dependencies {
                let available = m.programs.get(dependency).is_some_and(|p| p.entry.exists())
                    || executable(dependency, &environment.path).is_some();
                if !available {
                    failed = true;
                    sections.get_mut(section).expect("section exists").push(row(
                        format!("{name} dependency {dependency}"),
                        "missing",
                        format!("install {dependency} on the user PATH"),
                    ));
                }
            }
            failed |= !good;
            sections.get_mut(section).expect("section exists").push(row(
                name,
                if good { "ok" } else { "fail" },
                if good {
                    program.version.clone()
                } else {
                    "run: sno update".into()
                },
            ));
        }
        for (path, expected) in &m.files {
            if m.skill_destinations
                .keys()
                .any(|root| path.starts_with(root))
            {
                let good = manifest::read(path).ok().flatten().as_ref() == Some(expected);
                failed |= !good;
                sections
                    .get_mut("skills")
                    .expect("section exists")
                    .push(row(
                        path.to_string_lossy(),
                        if good { "ok" } else { "fail" },
                        if good { "" } else { "run: sno update" },
                    ));
            }
        }
        for h in &m.hooks {
            let configured = (|| {
                let Some(snapshot) = manifest::read(&h.path)? else {
                    return Ok(false);
                };
                hook_merge(&snapshot.bytes()?, &h.entry, true).map(|(_, found)| found)
            })();
            let good = match configured {
                Ok(found) => found,
                Err(error) => {
                    failed = true;
                    sections.get_mut("hooks").expect("section exists").push(row(
                        format!("{} configured", h.harness),
                        "fail",
                        error.message,
                    ));
                    continue;
                }
            };
            sections.get_mut("hooks").expect("section exists").push(row(
                format!("{} configured", h.harness),
                if good { "ok" } else { "fail" },
                if good { "" } else { "run: sno assemble" },
            ));
            failed |= !good;
            for dependency in ["jq", "sh"] {
                if executable(dependency, &environment.path).is_none() {
                    failed = true;
                    sections.get_mut("hooks").expect("section exists").push(row(
                        format!("{} dependency {dependency}", h.harness),
                        "missing",
                        format!("install {dependency} on the harness PATH"),
                    ));
                }
            }
            if executable("timeout", &environment.path)
                .or_else(|| executable("gtimeout", &environment.path))
                .is_none()
            {
                failed = true;
                sections.get_mut("hooks").expect("section exists").push(row(
                    format!("{} timeout dependency", h.harness),
                    "missing",
                    "install GNU coreutils timeout (gtimeout on macOS)",
                ));
            }
            if h.harness == "codex" {
                let trusted = match codex_trusted(h) {
                    Ok(value) => value,
                    Err(error) => {
                        failed = true;
                        sections.get_mut("hooks").expect("section exists").push(row(
                            "codex trust",
                            "fail",
                            error.message,
                        ));
                        false
                    }
                };
                sections.get_mut("hooks").expect("section exists").push(row(
                    "codex trust",
                    if trusted { "ok" } else { "missing" },
                    if trusted {
                        "stored trust matches this exact hook"
                    } else {
                        "open Codex and press t to trust this hook"
                    },
                ));
                failed |= !trusted;
            }
            sections.get_mut("hooks").expect("section exists").push(row(
                format!("{} execution", h.harness),
                "warn",
                if env::var_os("SNO_REACH_ADDR").is_none() {
                    "seat address missing; no execution evidence"
                } else {
                    "runtime-unverified; perform an addressed prompt"
                },
            ));
        }
        if m.timer {
            let active = if cfg!(target_os = "linux") {
                scheduler(
                    environment,
                    &["--user", "is-active", "--quiet", "sno-update.timer"],
                )
                .is_ok()
            } else {
                scheduler(environment, &["list", "ai.sno.update"]).is_ok()
            };
            failed |= !active;
            sections.get_mut("timer").expect("section exists").push(row(
                "timer",
                if active { "ok" } else { "fail" },
                if active {
                    ""
                } else {
                    "run: sno update --auto on"
                },
            ));
        } else {
            sections
                .get_mut("timer")
                .expect("section exists")
                .push(row("timer", "disabled", ""));
        }
    } else {
        failed = true;
        sections.get_mut("reach").expect("section exists").push(row(
            "manifest",
            "missing",
            "run: sno assemble",
        ));
        sections
            .get_mut("timer")
            .expect("section exists")
            .push(row("timer", "disabled", ""));
    }
    let agents_path = environment.home.join(".config/sno-reach/agents.json");
    match manifest::read(&agents_path)
        .and_then(|s| s.ok_or_else(|| InstallError::state("agents.json missing")))
        .and_then(|s| s.bytes())
        .and_then(|b| validate_agents(&b))
    {
        Ok(value) => {
            for (kind, entry) in value.as_object().into_iter().flatten() {
                let adapter = entry["acpx_agent"].as_str().expect("validated adapter");
                let check = match adapter_check(environment, kind, adapter) {
                    Ok(check) => check,
                    Err(error) => row(kind, "fail", error.message),
                };
                failed |= matches!(check.result.as_str(), "missing" | "fail");
                sections.get_mut("acp").expect("section exists").push(check);
            }
        }
        Err(e) => {
            failed = true;
            sections.get_mut("acp").expect("section exists").push(row(
                "agents.json",
                "fail",
                e.message,
            ));
        }
    }
    let (sno_station, exit) =
        crate::doctor::report().map_err(|e| InstallError::state(e.to_string()))?;
    let mut result =
        serde_json::to_value(sections).map_err(|e| InstallError::state(e.to_string()))?;
    result["station"] = sno_station;
    Ok((result, if failed || exit != 0 { 1 } else { 0 }))
}

pub fn run(action: Action, json_enabled: bool, source: &dyn ReleaseSource) -> i32 {
    let verb = match &action {
        Action::Assemble(_) => "assemble",
        Action::Update(_) => "update",
        Action::Doctor => "doctor",
        Action::Remove { .. } => "remove",
    };
    let result = (|| {
        if !cfg!(unix) {
            return Err(InstallError::usage("installer requires Linux or macOS"));
        }
        let environment = Environment::load()?;
        if matches!(&action, Action::Doctor) {
            let (value, exit) = diagnose(&environment)?;
            return Ok((value, exit));
        }
        let rows = match action {
            Action::Assemble(options) => install(&environment, &options, source, verb)?,
            Action::Update(options) => {
                let mut rows = if options.auto.as_deref() == Some("off") {
                    Vec::new()
                } else {
                    install(&environment, &options.versions, source, verb)?
                };
                if let Some(auto) = options.auto {
                    rows.extend(timer(&environment, auto == "on", source)?);
                }
                if options.quiet {
                    rows.retain(|r| r.result != "current" || r.detail.contains("trust"));
                }
                rows
            }
            Action::Remove { purge_state } => remove(&environment, purge_state, source)?,
            Action::Doctor => unreachable!("doctor returned above"),
        };
        Ok((
            serde_json::to_value(rows).map_err(|e| InstallError::state(e.to_string()))?,
            0,
        ))
    })();
    match result {
        Ok((value, exit)) => {
            if json_enabled {
                println!("{value}");
            } else if let Some(rows) = value.as_array() {
                for r in rows {
                    println!(
                        "{} {} {} {}",
                        verb,
                        r["target"].as_str().unwrap_or(""),
                        r["result"].as_str().unwrap_or(""),
                        r["detail"].as_str().unwrap_or("")
                    );
                }
            } else if let Some(sections) = value.as_object() {
                for (section, checks) in sections {
                    if let Some(rows) = checks.as_array() {
                        for r in rows {
                            println!(
                                "doctor {section} {} {} {}",
                                r["target"].as_str().unwrap_or(""),
                                r["result"].as_str().unwrap_or(""),
                                r["detail"].as_str().unwrap_or("")
                            );
                        }
                    } else if let Some(checks) = checks.as_object() {
                        for (name, check) in checks {
                            println!(
                                "doctor {section} {name} {} {}",
                                check["status"].as_str().unwrap_or("fail"),
                                check["detail"].as_str().unwrap_or("check unavailable")
                            );
                        }
                    }
                }
            }
            exit
        }
        Err(e) => {
            if json_enabled {
                println!(
                    "{}",
                    json!({"error":if e.exit_code==2{"usage_error"}else if e.exit_code==3{"source_error"}else{"install_error"},"message":e.message})
                );
            } else {
                eprintln!("{verb} fail {}", e.message);
            }
            e.exit_code
        }
    }
}

#[cfg(test)]
#[path = "../tests/fixtures/assemble/platform.rs"]
mod platform_tests;
