use std::path::PathBuf;

use sno::assemble::{Artifact, InstallError, ReleaseSet, ReleaseSource};

struct Files;

impl ReleaseSource for Files {
    fn resolve(&self, _: Option<&str>, _: Option<&str>) -> Result<ReleaseSet, InstallError> {
        let home = PathBuf::from(std::env::var_os("HOME").expect("isolated HOME"));
        let table = std::fs::read_to_string(home.join(".fixture-releases"))
            .map_err(|e| InstallError::source(e.to_string()))?;
        let mut programs = Vec::new();
        let mut skills = None;
        for line in table.lines() {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 5, "fixture artifact record");
            let artifact = Artifact {
                name: fields[0].into(),
                version: fields[1].into(),
                url: fields[2].into(),
                sha256: fields[3].into(),
                entry_point: fields[4].into(),
            };
            if artifact.name == "skills" {
                skills = Some(artifact);
            } else {
                programs.push(artifact);
            }
        }
        Ok(ReleaseSet {
            programs,
            skills: skills.expect("fixture skills"),
            contract_sha256: sno::assemble::CONTRACT_SHA256.into(),
        })
    }

    fn fetch(&self, url: &str) -> Result<Vec<u8>, InstallError> {
        let path = url.strip_prefix("file://").expect("local fixture URL");
        std::fs::read(path).map_err(|e| InstallError::source(e.to_string()))
    }
}

fn main() {
    std::process::exit(sno::cli::run_with_source(std::env::args_os(), &Files));
}
