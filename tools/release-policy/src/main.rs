use serde_yaml_ng::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::process;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| "workflow path is required".to_owned())?;
    let source = fs::read_to_string(&path).map_err(|error| format!("read {path}: {error}"))?;
    let workflow: Value =
        serde_yaml_ng::from_str(&source).map_err(|error| format!("parse {path}: {error}"))?;
    let jobs = mapping_value(&workflow, "jobs")?
        .as_mapping()
        .ok_or_else(|| "workflow jobs must be a mapping".to_owned())?;
    let allowed = allowed_writes();
    let mut errors = Vec::new();

    for (job_name, job) in jobs {
        let job_name = job_name
            .as_str()
            .ok_or_else(|| "workflow job name must be a string".to_owned())?;
        let Some(permissions) = job.get("permissions") else {
            continue;
        };
        if permissions.as_str() == Some("write-all") {
            errors.push(format!("{job_name}:write-all"));
            continue;
        }
        let Some(permissions) = permissions.as_mapping() else {
            continue;
        };
        for (permission, access) in permissions {
            if access.as_str() != Some("write") {
                continue;
            }
            let permission = permission
                .as_str()
                .ok_or_else(|| "permission name must be a string".to_owned())?;
            if !allowed
                .get(job_name)
                .is_some_and(|permissions| permissions.contains(permission))
            {
                errors.push(format!("{job_name}:{permission}"));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "unapproved write permission: {}",
            errors.join(", ")
        ))
    }
}

fn mapping_value<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("workflow is missing {key}"))
}

fn allowed_writes() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    BTreeMap::from([
        (
            "host",
            BTreeSet::from(["attestations", "contents", "id-token"]),
        ),
        (
            "custom-release-draft-installer-smoke",
            BTreeSet::from(["contents"]),
        ),
        ("publish-release", BTreeSet::from(["contents"])),
        ("cleanup-failed-draft", BTreeSet::from(["contents"])),
        ("host-public-candidate", BTreeSet::from(["contents"])),
        ("cleanup-public-candidate", BTreeSet::from(["contents"])),
        (
            "cleanup-confirmed-mutable-release",
            BTreeSet::from(["contents"]),
        ),
        ("cleanup-failed-public-smoke", BTreeSet::from(["contents"])),
    ])
}
