use super::*;
use std::cell::Cell;

fn assert_platform_refusal(os: &str, arch: &str) {
    let fetches = Cell::new(0);
    let error = GithubSource
        .resolve_for_platform(None, None, os, arch, None, |_| {
            fetches.set(fetches.get() + 1);
            Ok(b"[]".to_vec())
        })
        .expect_err("unsupported build platform must fail before fetching releases");
    assert_eq!(error.exit_code, 2);
    let expected = if matches!(os, "linux" | "macos") {
        format!("{os}-{arch}")
    } else {
        "Linux or macOS".to_owned()
    };
    assert!(error.message.contains(&expected), "{}", error.message);
    assert_eq!(
        fetches.get(),
        0,
        "unsupported platform fetched release data"
    );
}

fn checked_build_host_suffix() -> Option<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    if matches!((os, arch), ("linux", "x86_64") | ("macos", "aarch64")) {
        Some(reach_archive_suffix(os, arch).expect("supported build platform"))
    } else {
        assert_platform_refusal(os, arch);
        None
    }
}

#[test]
fn supported_pairs_use_the_exact_release_suffix() {
    for (os, arch, expected) in [
        ("linux", "x86_64", "-linux-x86_64.tar.gz"),
        ("macos", "aarch64", "-macos-aarch64.tar.gz"),
    ] {
        assert_eq!(reach_archive_suffix(os, arch).unwrap(), expected);
    }
}

#[test]
fn build_host_accepts_only_its_qualified_reach_archive() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let Some(suffix) = checked_build_host_suffix() else {
        return;
    };
    let expected = format!("reach-2.10-{os}-{arch}.tar.gz");
    assert_eq!(
        core_asset_version("reach", &expected, &suffix),
        Some("2.10")
    );
    assert_eq!(
        version(core_asset_version("reach", &expected, &suffix).unwrap()).unwrap(),
        [2, 10, 0]
    );
    for (other_os, other_arch) in [
        ("linux", "x86_64"),
        ("linux", "aarch64"),
        ("macos", "aarch64"),
        ("macos", "x86_64"),
    ] {
        if (other_os, other_arch) != (os, arch) {
            let other = format!("reach-2.10-{other_os}-{other_arch}.tar.gz");
            assert_eq!(
                core_asset_version("reach", &other, &suffix),
                None,
                "{other}"
            );
        }
    }
    for invalid in ["reach-2.10.tar.gz".to_owned(), format!("{expected}.sha256")] {
        assert_eq!(
            core_asset_version("reach", &invalid, &suffix),
            None,
            "{invalid}"
        );
    }
}

#[test]
fn utility_names_remain_platform_independent() {
    for suffix in ["-linux-x86_64.tar.gz", "-macos-aarch64.tar.gz"] {
        for program in ["heartbeat", "subscription-quota-check"] {
            let name = format!("{program}-1.0.tar.gz");
            assert_eq!(core_asset_version(program, &name, suffix), Some("1.0"));
            assert_eq!(
                core_asset_version(program, &format!("{name}.sha256"), suffix),
                None
            );
        }
    }
}

#[test]
fn host_archive_requires_its_own_exact_checksum_asset() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let Some(suffix) = checked_build_host_suffix() else {
        return;
    };
    let archive = format!("reach-2.10-{os}-{arch}.tar.gz");
    let other_os = if os == "linux" { "macos" } else { "linux" };
    let assets = vec![
        json!({"name":"reach-2.10.tar.gz.sha256","url":"https://example.invalid/legacy"}),
        json!({"name":format!("reach-2.10-{other_os}-{arch}.tar.gz.sha256"),"url":"https://example.invalid/other-platform"}),
        json!({"name":format!("reach-2.9-{os}-{arch}.tar.gz.sha256"),"url":"https://example.invalid/old-version"}),
        json!({"name":archive,"url":"https://example.invalid/host-archive"}),
        json!({"name":format!("{archive}.sha256"),"url":"https://example.invalid/host-checksum"}),
    ];
    assert_eq!(core_asset_version("reach", &archive, &suffix), Some("2.10"));
    assert_eq!(
        asset_checksum_url(&assets, &archive).as_deref(),
        Some("https://example.invalid/host-checksum")
    );
    assert_eq!(
        asset_checksum_url(&assets[..4], &archive),
        None,
        "no fallback to another platform, version, or old unqualified name"
    );
}

#[test]
fn unsupported_pairs_refuse_before_release_resolution() {
    assert_platform_refusal("linux", "aarch64");
    assert_platform_refusal("macos", "x86_64");
    assert_platform_refusal("linux", "riscv64");
    assert_platform_refusal("windows", "x86_64");
}

#[test]
fn skills_release_requires_the_exact_distribution_asset() {
    let release = json!({
        "tag_name": "skills-v1.0",
        "assets": [
            {"name":"final-skills.tar.gz","url":"https://api.github.com/assets/final"},
            {"name":"final-skills.tar.gz.sha256","url":"https://api.github.com/assets/checksum"},
            {"name":"source.tar.gz","url":"https://api.github.com/assets/source"}
        ]
    });
    let artifact = skills_release_artifact(&release, &|url| match url {
        "https://api.github.com/assets/checksum" => Ok(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  final-skills.tar.gz\n".to_vec()),
        url if url.contains("/git/ref/tags/skills-v1.0") => Ok(br#"{"object":{"type":"commit","sha":"1111111111111111111111111111111111111111"}}"#.to_vec()),
        other => panic!("unexpected fetch: {other}"),
    }).unwrap();
    assert_eq!(artifact.version, "skills-v1.0");
    assert_eq!(artifact.url, "https://api.github.com/assets/final");
    assert_eq!(artifact.sha256, "a".repeat(64));

    let missing = json!({"tag_name":"skills-v1.0","assets":[]});
    assert!(
        skills_release_artifact(&missing, &|_| unreachable!())
            .unwrap_err()
            .message
            .contains("missing release asset: final-skills.tar.gz")
    );

    let ambiguous = json!({
        "tag_name":"skills-v1.0",
        "assets":[
            {"name":"final-skills.tar.gz","url":"https://api.github.com/assets/one"},
            {"name":"final-skills.tar.gz","url":"https://api.github.com/assets/two"},
            {"name":"final-skills.tar.gz.sha256","url":"https://api.github.com/assets/checksum"}
        ]
    });
    assert!(
        skills_release_artifact(&ambiguous, &|_| unreachable!())
            .unwrap_err()
            .message
            .contains("ambiguous release asset: final-skills.tar.gz")
    );
}
