#!/usr/bin/env python3
"""Verify the published installer journey from durable external effects."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


TAG = "s-category-v1.0.0"
SKILLS_SHA256 = "563b87bf64324f789cb47adcf7132dff351a4f0d552867caf5ca2c0391186ed7"
PROGRAMS = {
    "reach": ("2.0", "b087a730097f16f7f115b33257da4783d0b7c927b91a17551fbc78245aafbc34"),
    "heartbeat": ("1.0", "5fbeb9222fd522d5fe3b7182712877b27d8d5a16d453aecc76a282a6b1597a96"),
    "subscription-quota-check": ("1.0", "e632a4cc3a4d40e18fdd6da1132a0544234034251851df932f6827b5bb79cd91"),
}
QUESTION_ID = "<qcg9-1789444437-10941@gpt1>"
NONCE = "QCG9-3af83dc2-5645-49f8-a3b9-505c9b6edf03"


def run(*command):
    result = subprocess.run(command, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result.stdout


def check(checks, name, condition):
    checks.append({"name": name, "passed": bool(condition)})
    assert condition, name


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--acceptance-home", type=Path, required=True)
    parser.add_argument("--reach-root", type=Path, required=True)
    parser.add_argument("--published-assemble", type=Path, required=True)
    parser.add_argument("--export", type=Path, required=True)
    parser.add_argument("--wait-result", type=Path, required=True)
    parser.add_argument("--sender-log", type=Path, required=True)
    parser.add_argument("--hook-log", type=Path, required=True)
    parser.add_argument("--wrong-current", type=Path, required=True)
    parser.add_argument("--remove", type=Path, required=True)
    parser.add_argument("--discovery-manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    checks = []

    release = json.loads(run(
        "gh", "release", "view", TAG, "--repo", "sno-ai/sno-station-skills",
        "--json", "tagName,assets",
    ))
    asset_names = [asset["name"] for asset in release["assets"]]
    check(checks, "published tag selected", release["tagName"] == TAG)
    check(checks, "exact distribution assets published",
          asset_names.count("final-skills.tar.gz") == 1
          and asset_names.count("final-skills.tar.gz.sha256") == 1)
    reference = json.loads(run(
        "gh", "api", f"repos/sno-ai/sno-station-skills/git/ref/tags/{TAG}",
    ))
    check(checks, "published tag has immutable object",
          len(reference["object"]["sha"]) == 40)
    with tempfile.TemporaryDirectory(prefix="sno-qcg9-assets-") as folder:
        run(
            "gh", "release", "download", TAG, "--repo", "sno-ai/sno-station-skills",
            "--pattern", "final-skills.tar.gz", "--pattern", "final-skills.tar.gz.sha256",
            "--dir", folder,
        )
        archive = Path(folder) / "final-skills.tar.gz"
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        published_digest = (Path(folder) / "final-skills.tar.gz.sha256").read_text().split()[0]
        check(checks, "published bytes equal tested distribution",
              digest == SKILLS_SHA256 == published_digest)

    rows = json.loads(args.published_assemble.read_text())
    for program, (version, _) in PROGRAMS.items():
        check(checks, f"clean install placed {program}",
              any(row["target"] == program and row["result"] == "installed"
                  and row["detail"] == version for row in rows))
    manifest = json.loads(Path.home().joinpath(".config/sno/assemble.json").read_text())
    check(checks, "real installation records published skills tag", manifest["skills_version"] == TAG)
    for program, (version, digest) in PROGRAMS.items():
        installed = manifest["programs"][program]
        check(checks, f"real {program} identity", installed["version"] == version
              and installed["sha256"] == digest)
    check(checks, "installed Reach entry runs", run("sno", "reach", "--version").strip() == "2.0")
    run("heartbeat", "--help")
    run("subscription-quota-check", "--help")
    check(checks, "installed utility entries run", True)

    transcript = args.export.read_text()
    accepted = transcript.index("X-State: accepted")
    completed = transcript.index("X-State: completed")
    check(checks, "receiver accepted before completion", accepted < completed)
    check(checks, "one question and nonce remained in one thread",
          transcript.count(QUESTION_ID) >= 3 and transcript.count(NONCE) >= 3)
    check(checks, "sender wait returned terminal answer", args.wait_result.read_text().strip() != "")
    sender_log = args.sender_log.read_text()
    check(checks, "sender acknowledged both reports",
          sender_log.count(":2,T") == 2 and "[ANSWER]" in sender_log)

    hook_log = args.hook_log.read_text()
    check(checks, "Codex explicitly trusted the installed hook",
          "Hooks need review" in hook_log and "Trust all and continue" in hook_log)
    check(checks, "addressed real prompt received reminder context",
          "HOOK-INJECTED-cfc15291" in hook_log)
    wrong = json.loads(args.wrong_current.read_text())
    check(checks, "wrong current fails Reach before seat setup",
          any(row["result"] == "fail" for row in wrong["reach"]))

    removed = json.loads(args.remove.read_text())
    check(checks, "remove deleted all owned commands",
          all(any(row["target"].endswith(f"/.local/bin/{program}")
                  and row["result"] == "removed" for row in removed)
              for program in ("sno-reach", "heartbeat", "subscription-quota-check")))
    check(checks, "remove preserved user Reach state",
          args.acceptance_home.joinpath(".local/state/sno-reach/user-state.txt").read_text().strip()
          == "preserve-qcg9-user-state")
    discovery = json.loads(args.discovery_manifest.read_text())
    destinations = discovery["skill_destinations"]
    check(checks, "actual Claude and Codex roots discovered",
          any("/.claude/skills/" in path for path in destinations)
          and any("/.codex/skills/" in path for path in destinations))
    check(checks, "Hermes and custom OpenClaw roots discovered",
          any("sno-qcg9-hermes." in path for path in destinations)
          and any("sno-qcg9-openclaw." in path for path in destinations))

    state = json.loads(run(
        "env", f"SNO_REACH_ROOT={args.reach_root}", "sno", "reach", "state",
        "--work", "qcg9-published-install", "--json",
    ))
    check(checks, "Reach work remains completed", state["state"] == "completed")
    result = {
        "host": "gpt1",
        "tag": TAG,
        "tag_object": reference["object"],
        "skills_sha256": SKILLS_SHA256,
        "acceptance_home": str(args.acceptance_home),
        "reach_root": str(args.reach_root),
        "checks": checks,
        "passed": all(item["passed"] for item in checks),
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(f"QCG-9 published acceptance: {len(checks)} checks passed")


if __name__ == "__main__":
    main()
