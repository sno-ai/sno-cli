use crate::assemble::{InstallError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROGRAM_IDS: [&str; 3] = ["reach", "heartbeat", "subscription-quota-check"];
pub const SLOT_IDS: [&str; 5] = [
    "4.shell",
    "4.file-read-write",
    "4.background-processes",
    "2.pre-turn-context-injection",
    "3.reader-to-agent-delivery",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub schema_version: u32,
    pub program_ids: Vec<String>,
    pub slot_ids: Vec<String>,
    pub need_values: Vec<String>,
    pub version_pattern: String,
}
impl Contract {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let c: Self = serde_json::from_slice(bytes)
            .map_err(|e| InstallError::source(format!("requirements contract: {e}")))?;
        fn exact(values: &[String], expected: &[&str]) -> bool {
            values.len() == expected.len()
                && values.iter().collect::<BTreeSet<_>>().len() == expected.len()
                && expected.iter().all(|v| values.iter().any(|x| x == v))
        }
        if c.schema_version != 1
            || !exact(&c.program_ids, &PROGRAM_IDS)
            || !exact(&c.slot_ids, &SLOT_IDS)
            || !exact(&c.need_values, &["required", "preferred", "optional"])
            || c.version_pattern != "^[0-9]+[.][0-9]+([.][0-9]+)?$"
        {
            return Err(InstallError::source("unsupported requirements contract"));
        }
        Ok(c)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    pub programs: Vec<ProgramNeed>,
    pub harness: Vec<HarnessNeed>,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProgramNeed {
    pub name: String,
    pub min_version: String,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HarnessNeed {
    pub slot: String,
    pub need: String,
}

pub fn version(text: &str) -> Result<[u64; 3]> {
    let parts: Vec<_> = text.split('.').collect();
    if !(2..=3).contains(&parts.len())
        || parts
            .iter()
            .any(|x| x.is_empty() || !x.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err(InstallError::source(format!("invalid version: {text}")));
    }
    let mut result = [0; 3];
    for (i, p) in parts.iter().enumerate() {
        result[i] = p
            .parse()
            .map_err(|_| InstallError::source(format!("version out of range: {text}")))?;
    }
    Ok(result)
}
impl Requirements {
    pub fn from_skill(bytes: &[u8], contract: &Contract) -> Result<Self> {
        let text = std::str::from_utf8(bytes).map_err(|e| InstallError::source(e.to_string()))?;
        let text = text
            .strip_prefix("---\n")
            .or_else(|| text.strip_prefix("---\r\n"))
            .ok_or_else(|| InstallError::source("SKILL.md missing frontmatter"))?;
        let mut offset = 0;
        let mut end = None;
        for line in text.split_inclusive('\n') {
            if line.trim_end() == "---" {
                end = Some(offset);
                break;
            }
            offset += line.len();
        }
        let front =
            &text[..end.ok_or_else(|| InstallError::source("SKILL.md truncated frontmatter"))?];
        let map: serde_yaml::Value = serde_yaml::from_str(front)
            .map_err(|e| InstallError::source(format!("SKILL.md: {e}")))?;
        let requires = map
            .get("requires")
            .ok_or_else(|| InstallError::source("SKILL.md missing requires"))?;
        let r: Self = serde_yaml::from_value(requires.clone())
            .map_err(|e| InstallError::source(format!("requires: {e}")))?;
        let mut names = BTreeSet::new();
        for p in &r.programs {
            if !contract.program_ids.contains(&p.name) || !names.insert(&p.name) {
                return Err(InstallError::source(format!(
                    "requires.programs unknown/duplicate name: {}",
                    p.name
                )));
            }
            version(&p.min_version)?;
        }
        let mut slots = BTreeSet::new();
        for h in &r.harness {
            if !contract.slot_ids.contains(&h.slot)
                || !slots.insert(&h.slot)
                || !contract.need_values.contains(&h.need)
            {
                return Err(InstallError::source(format!(
                    "requires.harness invalid slot/need: {} {}",
                    h.slot, h.need
                )));
            }
        }
        Ok(r)
    }
    pub fn decide(
        &self,
        consumers: &[String],
        installed: &BTreeMap<String, String>,
    ) -> Result<(&'static str, String)> {
        for p in &self.programs {
            let Some(v) = installed.get(&p.name) else {
                return Ok((
                    "skipped",
                    format!("{} missing; requires {}", p.name, p.min_version),
                ));
            };
            if version(v)? < version(&p.min_version)? {
                return Ok((
                    "skipped",
                    format!("{} requires {} installed {}", p.name, p.min_version, v),
                ));
            }
        }
        let mut degraded = Vec::new();
        for consumer in consumers {
            for h in &self.harness {
                let supported = match h.slot.as_str() {
                    "4.shell" | "4.file-read-write" | "4.background-processes" => matches!(
                        consumer.as_str(),
                        "claude" | "codex" | "hermes" | "openclaw"
                    ),
                    "2.pre-turn-context-injection" => {
                        matches!(consumer.as_str(), "claude" | "codex" | "hermes")
                    }
                    _ => false,
                };
                if supported || h.need == "optional" {
                    continue;
                }
                let detail = format!("{consumer} {} unsupported or unverified", h.slot);
                if h.need == "required" {
                    return Ok(("skipped", detail));
                }
                degraded.push(detail);
            }
        }
        Ok(if degraded.is_empty() {
            ("installed", String::new())
        } else {
            ("degraded", degraded.join("; "))
        })
    }
}
