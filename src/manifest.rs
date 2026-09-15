use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::assemble::{InstallError, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Snapshot {
    File {
        hash: String,
        mode: u32,
        #[serde(skip)]
        data: Vec<u8>,
    },
    Link {
        target: PathBuf,
    },
}
impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::File {
                    hash: a, mode: x, ..
                },
                Self::File {
                    hash: b, mode: y, ..
                },
            ) => a == b && x == y,
            (Self::Link { target: a }, Self::Link { target: b }) => a == b,
            _ => false,
        }
    }
}
impl Eq for Snapshot {}
fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
impl Snapshot {
    pub fn file(bytes: &[u8], mode: u32) -> Self {
        Self::File {
            hash: hash_bytes(bytes),
            mode,
            data: bytes.to_vec(),
        }
    }
    pub fn bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::File { hash, data, .. } if hash_bytes(data) == *hash => Ok(data.clone()),
            Self::File { .. } => Err(InstallError::state(
                "snapshot body requires its content store",
            )),
            Self::Link { .. } => Err(InstallError::state("expected a regular file")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub version: String,
    pub sha256: String,
    pub entry: PathBuf,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    pub path: PathBuf,
    pub entry: String,
    pub owned: bool,
    pub harness: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub generation: String,
    pub files: BTreeMap<PathBuf, Snapshot>,
    pub directories: Vec<PathBuf>,
    pub programs: BTreeMap<String, Program>,
    pub skills_version: String,
    pub skill_destinations: BTreeMap<PathBuf, Vec<String>>,
    pub contract_sha256: String,
    pub requirements_fixture_sha256: String,
    pub hooks: Vec<Hook>,
    pub timer: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub path: PathBuf,
    pub before: Option<Snapshot>,
    pub after: Option<Snapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Effect {
    pub before_files: bool,
    pub apply: Vec<CommandSpec>,
    pub before_restore: Vec<CommandSpec>,
    pub after_restore: Vec<CommandSpec>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    generation: String,
    previous: Option<Manifest>,
    next: Option<Manifest>,
    operations: Vec<Operation>,
    directories: Vec<PathBuf>,
    effect: Option<Effect>,
    effect_started: bool,
}

pub fn read(path: &Path) -> Result<Option<Snapshot>> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(InstallError::path(path, e)),
    };
    if meta.file_type().is_symlink() {
        return Ok(Some(Snapshot::Link {
            target: fs::read_link(path).map_err(|e| InstallError::path(path, e))?,
        }));
    }
    if !meta.is_file() || meta.len() > 64 * 1024 * 1024 {
        return Err(InstallError::state(format!(
            "not a bounded regular file: {}",
            path.display()
        )));
    }
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o777
    };
    #[cfg(not(unix))]
    let mode = 0o600;
    Ok(Some(Snapshot::file(
        &fs::read(path).map_err(|e| InstallError::path(path, e))?,
        mode,
    )))
}

pub fn atomic(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(InstallError::path(
            path,
            "transaction/file exceeds 64 MiB limit",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| InstallError::state("file has no parent"))?;
    let temp = parent.join(format!(".sno-{}", Uuid::now_v7()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(mode);
        }
        let mut file = options
            .open(&temp)
            .map_err(|e| InstallError::path(&temp, e))?;
        file.write_all(bytes)
            .map_err(|e| InstallError::path(&temp, e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(mode))
                .map_err(|e| InstallError::path(&temp, e))?;
        }
        file.sync_all().map_err(|e| InstallError::path(&temp, e))?;
        fs::rename(&temp, path).map_err(|e| InstallError::path(path, e))?;
        sync_dir(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| InstallError::path(path, e))?;
    Ok(())
}

fn put(path: &Path, value: &Option<Snapshot>, blob_root: &Path) -> Result<()> {
    match value {
        Some(Snapshot::File { hash, mode, data }) => {
            let bytes = if hash_bytes(data) == *hash {
                data.clone()
            } else {
                fs::read(blob_path(blob_root, hash)?)
                    .map_err(|e| InstallError::path(blob_root, e))?
            };
            if hash_bytes(&bytes) != *hash {
                return Err(InstallError::state("snapshot body checksum mismatch"));
            }
            atomic(path, &bytes, *mode)
        }
        Some(Snapshot::Link { target }) => {
            #[cfg(unix)]
            {
                let parent = path
                    .parent()
                    .ok_or_else(|| InstallError::state("link has no parent"))?;
                let temp = parent.join(format!(".sno-{}", Uuid::now_v7()));
                std::os::unix::fs::symlink(target, &temp)
                    .map_err(|e| InstallError::path(&temp, e))?;
                if let Err(e) = fs::rename(&temp, path) {
                    let _ = fs::remove_file(&temp);
                    return Err(InstallError::path(path, e));
                }
                sync_dir(parent)
            }
            #[cfg(not(unix))]
            {
                let _ = target;
                Err(InstallError::usage("installer requires Linux or macOS"))
            }
        }
        None => match fs::remove_file(path) {
            Ok(()) => sync_dir(
                path.parent()
                    .ok_or_else(|| InstallError::state("file has no parent"))?,
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(InstallError::path(path, e)),
        },
    }
}

pub struct Store {
    pub root: PathBuf,
    pub allowed: Vec<PathBuf>,
    _lock: File,
}

impl Store {
    pub fn open(root: PathBuf, allowed: Vec<PathBuf>) -> Result<Self> {
        fs::create_dir_all(&root).map_err(|e| InstallError::path(&root, e))?;
        let lock_path = root.join("assemble.lock");
        if fs::symlink_metadata(&lock_path)
            .is_ok_and(|m| !m.is_file() || m.file_type().is_symlink())
        {
            return Err(InstallError::state("unsafe installer lock"));
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| InstallError::path(&lock_path, e))?;
        lock.try_lock_exclusive()
            .map_err(|e| InstallError::state(format!("installer busy: {e}")))?;
        Ok(Self {
            root,
            allowed,
            _lock: lock,
        })
    }
    pub fn load(root: &Path) -> Result<Option<Manifest>> {
        let path = root.join("assemble.json");
        match read(&path)? {
            None => Ok(None),
            Some(snapshot) => serde_json::from_slice(&snapshot.bytes()?)
                .map(Some)
                .map_err(|e| InstallError::path(&path, e)),
        }
    }
    fn safe(&self, path: &Path) -> Result<()> {
        if !path.is_absolute()
            || path.components().any(|c| matches!(c, Component::ParentDir))
            || !self
                .allowed
                .iter()
                .any(|root| path.starts_with(root) && path != root)
        {
            return Err(InstallError::state(format!(
                "unowned destination: {}",
                path.display()
            )));
        }
        let mut parent = path.parent();
        while let Some(p) = parent {
            if fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(InstallError::state(format!(
                    "destination parent changed to symlink: {}",
                    p.display()
                )));
            }
            parent = p.parent();
        }
        Ok(())
    }
    pub fn recover(&self) -> Result<()> {
        let pending_path = self.root.join("assemble.pending.json");
        let Some(snapshot) = read(&pending_path)? else {
            return self.collect_blobs();
        };
        let pending: Pending = serde_json::from_slice(&snapshot.bytes()?)
            .map_err(|e| InstallError::path(&pending_path, e))?;
        for op in &pending.operations {
            self.safe(&op.path)?;
        }
        for dir in &pending.directories {
            self.safe(dir)?;
        }
        let current = Self::load(&self.root)?;
        let committed = match &pending.next {
            Some(_) => current
                .as_ref()
                .is_some_and(|m| m.generation == pending.generation),
            None => current.is_none() && pending.previous.is_some(),
        };
        if !committed {
            if pending.effect_started {
                if let Some(effect) = &pending.effect {
                    crate::assemble::execute_specs(&effect.before_restore)?;
                }
            }
            for op in pending.operations.iter().rev() {
                let actual = read(&op.path)?;
                if actual == op.before {
                    continue;
                }
                if actual != op.after {
                    return Err(InstallError::state(format!(
                        "recovery conflict: {}",
                        op.path.display()
                    )));
                }
                put(&op.path, &op.before, &self.root.join("assemble-blobs"))?;
            }
            for dir in pending.directories.iter().rev() {
                remove_empty(dir)?;
            }
            if pending.effect_started {
                if let Some(effect) = &pending.effect {
                    crate::assemble::execute_specs(&effect.after_restore)?;
                }
            }
        }
        if committed {
            if let Some(previous) = &pending.previous {
                let mut directories = previous.directories.clone();
                directories.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
                for dir in directories {
                    if pending
                        .next
                        .as_ref()
                        .is_some_and(|m| m.directories.contains(&dir))
                    {
                        continue;
                    }
                    self.safe(&dir)?;
                    remove_empty(&dir)?;
                }
            }
        }
        fs::remove_file(&pending_path).map_err(|e| InstallError::path(&pending_path, e))?;
        sync_dir(&self.root)?;
        self.collect_blobs()
    }
    fn save_blob(&self, snapshot: &Snapshot) -> Result<()> {
        let Snapshot::File { hash, data, .. } = snapshot else {
            return Ok(());
        };
        let root = self.root.join("assemble-blobs");
        if fs::symlink_metadata(&root).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(InstallError::state("snapshot directory is a symlink"));
        }
        if !root.exists() {
            fs::create_dir(&root).map_err(|e| InstallError::path(&root, e))?;
            sync_dir(&self.root)?;
        }
        let path = blob_path(&root, hash)?;
        if let Some(saved) = read(&path)? {
            if hash_bytes(&saved.bytes()?) != *hash {
                return Err(InstallError::state("stored snapshot checksum mismatch"));
            }
            return Ok(());
        }
        if hash_bytes(data) != *hash {
            return Err(InstallError::state(format!(
                "missing snapshot body: {hash}"
            )));
        }
        atomic(&path, data, 0o600)
    }
    fn collect_blobs(&self) -> Result<()> {
        let root = self.root.join("assemble-blobs");
        if !root.exists() {
            return Ok(());
        }
        if fs::symlink_metadata(&root)
            .map_err(|e| InstallError::path(&root, e))?
            .file_type()
            .is_symlink()
        {
            return Err(InstallError::state("snapshot directory is a symlink"));
        }
        let mut live = BTreeSet::new();
        if let Some(manifest) = Self::load(&self.root)? {
            for value in manifest.files.values() {
                if let Snapshot::File { hash, .. } = value {
                    live.insert(hash.clone());
                }
            }
        }
        for entry in fs::read_dir(&root).map_err(|e| InstallError::path(&root, e))? {
            let entry = entry.map_err(|e| InstallError::path(&root, e))?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if live.contains(&name) || !valid_hash(&name) {
                continue;
            }
            let Some(value) = read(&entry.path())? else {
                continue;
            };
            if hash_bytes(&value.bytes()?) != name {
                return Err(InstallError::state("changed snapshot body during cleanup"));
            }
            fs::remove_file(entry.path()).map_err(|e| InstallError::path(&entry.path(), e))?;
        }
        sync_dir(&root)
    }
    pub fn commit(
        &self,
        operations: Vec<Operation>,
        next: Option<Manifest>,
        checkpoint: &dyn Fn(&str) -> Result<()>,
    ) -> Result<()> {
        self.commit_effect(operations, next, checkpoint, None)
    }
    pub fn commit_effect(
        &self,
        operations: Vec<Operation>,
        mut next: Option<Manifest>,
        checkpoint: &dyn Fn(&str) -> Result<()>,
        effect: Option<Effect>,
    ) -> Result<()> {
        let previous = Self::load(&self.root)?;
        for op in &operations {
            self.safe(&op.path)?;
            if read(&op.path)? != op.before {
                return Err(InstallError::state(format!(
                    "concurrent edit: {}",
                    op.path.display()
                )));
            }
        }
        let mut directories = Vec::new();
        for op in &operations {
            if op.after.is_none() {
                continue;
            }
            let mut p = op.path.parent();
            while let Some(dir) = p {
                if dir.exists() {
                    break;
                }
                if !directories.contains(&dir.to_path_buf()) {
                    directories.push(dir.to_path_buf());
                }
                p = dir.parent();
            }
        }
        directories.sort_by_key(|p| p.components().count());
        let generation = Uuid::now_v7().to_string();
        if let Some(m) = next.as_mut() {
            m.generation = generation.clone();
            m.directories
                .retain(|dir| m.files.keys().any(|p| p.starts_with(dir)));
            for dir in &directories {
                if !m.directories.contains(dir) {
                    m.directories.push(dir.clone());
                }
            }
        }
        let mut pending = Pending {
            generation,
            previous,
            next,
            operations,
            directories,
            effect,
            effect_started: false,
        };
        for op in &pending.operations {
            if let Some(value) = &op.before {
                self.save_blob(value)?;
            }
            if let Some(value) = &op.after {
                self.save_blob(value)?;
            }
        }
        if let Some(next) = &pending.next {
            for value in next.files.values() {
                self.save_blob(value)?;
            }
        }
        let pending_path = self.root.join("assemble.pending.json");
        atomic(
            &pending_path,
            &serde_json::to_vec(&pending).map_err(|e| InstallError::state(e.to_string()))?,
            0o600,
        )?;
        let apply_effect = |pending: &mut Pending| -> Result<()> {
            pending.effect_started = true;
            atomic(
                &pending_path,
                &serde_json::to_vec(pending).map_err(|e| InstallError::state(e.to_string()))?,
                0o600,
            )?;
            if let Some(effect) = &pending.effect {
                crate::assemble::execute_specs(&effect.apply)?;
            }
            checkpoint("after-scheduler")
        };
        let applied = (|| {
            if pending.effect.as_ref().is_some_and(|e| e.before_files) {
                apply_effect(&mut pending)?;
            }
            for dir in &pending.directories {
                fs::create_dir(dir).map_err(|e| InstallError::path(dir, e))?;
                sync_dir(
                    dir.parent()
                        .ok_or_else(|| InstallError::state("directory has no parent"))?,
                )?;
            }
            for op in &pending.operations {
                if read(&op.path)? != op.before {
                    return Err(InstallError::state(format!(
                        "concurrent edit: {}",
                        op.path.display()
                    )));
                }
                put(&op.path, &op.after, &self.root.join("assemble-blobs"))?;
                checkpoint(&format!("after-write:{}", op.path.display()))?;
            }
            if pending.effect.as_ref().is_some_and(|e| !e.before_files) {
                apply_effect(&mut pending)?;
            }
            let path = self.root.join("assemble.json");
            match &pending.next {
                Some(m) => atomic(
                    &path,
                    &serde_json::to_vec_pretty(m)
                        .map_err(|e| InstallError::state(e.to_string()))?,
                    0o600,
                )?,
                None => put(&path, &None, &self.root.join("assemble-blobs"))?,
            }
            checkpoint("after-manifest")?;
            Ok(())
        })();
        if let Err(error) = applied {
            return match self.recover() {
                Ok(()) => Err(error),
                Err(recovery) => Err(InstallError::state(format!(
                    "{error}; recovery pending: {recovery}"
                ))),
            };
        }
        self.recover()?;
        Ok(())
    }
}

fn remove_empty(path: &Path) -> Result<()> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(e) => Err(InstallError::path(path, e)),
    }
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn blob_path(root: &Path, hash: &str) -> Result<PathBuf> {
    if !valid_hash(hash) {
        return Err(InstallError::state("invalid snapshot hash"));
    }
    Ok(root.join(hash))
}
