#!/usr/bin/env python3
"""Verify real staged inputs and explicitly derived declaration cases; never publish."""

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tarfile
import tempfile

import yaml

REACH_SOURCE = "75fa8d21a76909ce3ebdadfa45df31e3feef6adb"
PINS = {
    "reach": "47edd32a93366bfac2d650c261869599e8f8730fc5aab5d9ddfaf918d63a21b5",
    "heartbeat": "29d1eb773df780b0f85b935549b90d05de0856411e82144f0331de8defd0cb53",
    "subscription-quota-check": "b857aef24ffc1216c2818469d17175823f388e3c57a4abb67866e2f9e5375505",
    "skills": "857e2f29f11bcce208823adc0b1a6bac33db417e09581e59069d898ec4b94627",
}
FIXTURE = "scripts/fixtures/s-category-requirements.json"
CONTRACT = "scripts/requirements-contract.json"
CATEGORY = "skills/S-communication-and-handoff"
UNITS = ["reach", "handoff", "heartbeat", "subscription-quota-check"]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def home_snapshot(home):
    """Only the empty Store::open lock is excluded; every other HOME path is checked."""
    result = {}
    for path in sorted(home.rglob("*")):
        relative = path.relative_to(home)
        if relative == Path(".config/sno/assemble.lock"):
            continue
        metadata = path.lstat()
        item = {"mode": stat.S_IMODE(metadata.st_mode)}
        if path.is_symlink():
            item.update(kind="symlink", target=os.readlink(path))
        elif path.is_dir():
            item.update(kind="directory")
        else:
            assert stat.S_ISREG(metadata.st_mode), path
            item.update(kind="file", sha256=digest(path))
        result[str(relative)] = item
    return result


class Verification:
    def __init__(self, args, work):
        self.repo = Path(__file__).resolve().parents[3]
        self.work = work
        self.output = args.output.resolve()
        supplied = json.loads(args.inputs.read_text())
        self.artifacts = supplied["artifacts"]
        assert {a["name"]: digest(a["path"]) for a in self.artifacts} == PINS
        assert len(self.artifacts) == len(PINS)
        self.base = work / "original"
        skills = next(a for a in self.artifacts if a["name"] == "skills")
        with tarfile.open(skills["path"]) as archive:
            archive.extractall(self.base, filter="data")
        self.fixture = json.loads((self.base / FIXTURE).read_text())
        assert digest(self.base / FIXTURE) == "17eb722714548f71492d89d40ba09764bd2a24053c1c67f6ef70039dc3785d69"
        assert digest(self.base / CONTRACT) == self.fixture["contract_sha256"]
        self.gate_path = args.gate.resolve()
        self.receipt = {
            "host": os.uname().nodename,
            "reach_source_commit": REACH_SOURCE,
            "skills_source_commit": self.fixture["source"]["commit"],
            "artifacts": self.artifacts,
            "fixture_sha256": digest(self.base / FIXTURE),
            "contract_sha256": digest(self.base / CONTRACT),
            "gate": {"path": str(self.gate_path), "sha256": digest(self.gate_path)},
            "events": [], "checks": [], "complete": False,
            "boundaries": [
                "Original positive inputs are real local candidates, not published releases.",
                "Declaration variations are explicit copies; they are not new upstream releases.",
                "Configured harness roots prove installer resolution, not live harness loading.",
                "Same-release missing-link repair proves changed owned targets, not a version upgrade.",
                "No live reader delivery, adapter handshake, prompt injection or QCG-9 claim.",
            ],
        }
        self.receipt["gate"]["commit"] = self.run("gate-commit", ["git", "-C", str(self.gate_path.parent.parent), "rev-parse", "HEAD"]).stdout.strip()
        build = self.run("build-library", ["cargo", "build", "--lib", "--message-format=json"], cwd=self.repo)
        assert build.returncode == 0, build.stderr
        libraries = [Path(p) for line in build.stdout.splitlines()
                     if (row := json.loads(line)).get("reason") == "compiler-artifact"
                     and row["target"]["name"] == "sno" and "lib" in row["target"]["kind"]
                     for p in row["filenames"] if p.endswith(".rlib")]
        assert len(libraries) == 1
        self.driver = work / "fixture-sno"
        result = self.run("compile-driver", ["rustc", "--edition=2024", "--crate-name=fixture_sno",
                         str(self.repo / "tests/fixtures/assemble/driver.rs"), "-L",
                         "dependency=" + str(libraries[0].parent / "deps"), "--extern",
                         "sno=" + str(libraries[0]), "-o", str(self.driver)])
        assert result.returncode == 0, result.stderr
        self.receipt["driver_sha256"] = digest(self.driver)
        self.receipt["cli_commit"] = self.run("cli-commit", ["git", "rev-parse", "HEAD"], cwd=self.repo).stdout.strip()

    def save(self):
        self.output.parent.mkdir(parents=True, exist_ok=True)
        self.output.write_text(json.dumps(self.receipt, indent=2) + "\n")

    def run(self, label, command, env=None, cwd=None):
        result = subprocess.run([str(p) for p in command], env=env, cwd=cwd, capture_output=True, text=True, timeout=60)
        self.receipt["events"].append({"label": label, "command": [str(p) for p in command],
                                       "environment": env, "cwd": str(cwd) if cwd else None,
                                       "exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        self.save()
        return result

    def check(self, label, condition):
        self.receipt["checks"].append({"name": label, "passed": bool(condition)})
        self.save()
        assert condition, label
        print("PASS", label, flush=True)

    def environment(self, label, roots=False):
        home = self.work / label / "home"
        home.mkdir(parents=True)
        env = {"HOME": str(home), "PATH": str(home / ".local/bin") + ":" + os.environ["PATH"],
               "XDG_CONFIG_HOME": str(home / ".config"), "SNO_PROFILE_DIR": str(home / "station")}
        (home / ".claude/skills").mkdir(parents=True)
        if roots:
            (home / ".claude/skills").rmdir()
            shared = home / "shared-skill-text"
            shared.mkdir()
            (home / ".claude/skills").symlink_to(shared, target_is_directory=True)
            (home / ".codex").mkdir()
            (home / ".codex/skills").symlink_to(shared, target_is_directory=True)
            for kind, variable in [("hermes", "HERMES_HOME"), ("openclaw", "OPENCLAW_STATE_DIR")]:
                base = home.parent / ("custom-" + kind)
                (base / "skills").mkdir(parents=True)
                env[variable] = str(base)
        return home, env

    def select(self, home, archive=None, omitted=()):
        selected = copy.deepcopy(self.artifacts)
        if archive:
            skill = next(a for a in selected if a["name"] == "skills")
            skill.update(path=str(archive), sha256=digest(archive))
        (home / ".fixture-releases").write_text("".join(
            "\t".join((a["name"], a["version"], Path(a["path"]).as_uri(), a["sha256"], a["entry_point"])) + "\n"
            for a in selected if a["name"] not in omitted))

    def cli(self, label, env, verb="assemble", expected=0):
        result = self.run(label, [self.driver, verb, "--json"], env)
        self.check(label + " exit", result.returncode == expected)
        return json.loads(result.stdout)

    def gate_check(self, label, tree, unit, expected=0, error=None):
        result = self.run(label, ["python3", self.gate_path, "validate", tree / CONTRACT, tree / CATEGORY / unit, "S"])
        self.check(label + " exit", result.returncode == expected)
        if error:
            self.check(label + " field error", error in result.stderr)
        else:
            self.check(label + " contract binding", json.loads(result.stdout)["contract_sha256"] == self.receipt["contract_sha256"])

    def variant(self, label, unit="reach", requires=None, fault=None, overlay=False):
        tree = self.work / ("copy-" + label)
        shutil.copytree(self.base, tree, symlinks=True)
        skill = tree / CATEGORY / unit / "skill/SKILL.md"
        prefix, front, body = skill.read_text().split("---", 2)
        assert not prefix
        header = yaml.safe_load(front)
        if requires is not None:
            header["requires"] = copy.deepcopy(requires)
        if fault == "missing":
            header.pop("requires")
        changed = "---\n" + yaml.safe_dump(header, sort_keys=False, allow_unicode=True)
        if fault == "duplicate":
            changed += "requires: " + json.dumps(header["requires"]) + "\n"
        changed += "---" + body
        if overlay:
            skill = skill.parent / "overlays/probe/SKILL.md"
            skill.parent.mkdir(parents=True)
        skill.write_text(changed)
        fixture = json.loads((tree / FIXTURE).read_text())
        if not overlay:
            row = next(row for row in fixture["units"] if row["unit"] == unit)
            row["skill_sha256"] = digest(skill)
            row["requires"] = header.get("requires", row["requires"])
        (tree / FIXTURE).write_text(json.dumps(fixture, indent=2) + "\n")
        archive = self.work / (label + ".tar.gz")
        with tarfile.open(archive, "w:gz") as output:
            for path in sorted(tree.rglob("*")):
                output.add(path, arcname=path.relative_to(tree), recursive=False)
        self.receipt.setdefault("copied_variants", []).append({"name": label, "archive_sha256": digest(archive),
                                                               "fixture_sha256": digest(tree / FIXTURE),
                                                               "unit": unit, "overlay": overlay})
        return tree, archive


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--gate", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="sno-real-staged-verifier-") as folder:
        verifier = Verification(args, Path(folder))
        try:
            exercise(verifier)
            verifier.check("original artifacts unchanged", {a["name"]: digest(a["path"]) for a in verifier.artifacts} == PINS)
            verifier.check("gate source unchanged", digest(verifier.gate_path) == verifier.receipt["gate"]["sha256"])
            verifier.receipt["complete"] = True
        finally:
            verifier.save()


def exercise(v):
    def row(rows, unit, consumer="claude"):
        matches = [r for r in rows if r["target"] == f"skill {unit} {consumer}"]
        assert len(matches) == 1, (unit, consumer, rows)
        return matches[0]

    original = {r["unit"]: r["requires"] for r in v.fixture["units"]}
    for unit in UNITS:
        v.gate_check("original-gate-" + unit, v.base, unit)
    home, env = v.environment("roots", roots=True)
    v.select(home)
    rows = v.cli("original-install-custom-roots", env)
    manifest_path = home / ".config/sno/assemble.json"
    manifest = json.loads(manifest_path.read_text())
    v.check("original fixture hash recorded", manifest["requirements_fixture_sha256"] == v.receipt["fixture_sha256"])
    v.check("original contract hash recorded", manifest["contract_sha256"] == v.receipt["contract_sha256"])
    shared = home / "shared-skill-text"
    v.check("aliased consumers written together once", row(rows, "reach", "claude,codex")["result"] == "installed"
            and manifest["skill_destinations"][str(shared / "reach")] == ["claude", "codex"])
    v.check("Hermes real pre-turn slot not degraded", row(rows, "reach", "hermes")["result"] == "installed")
    v.check("OpenClaw unsupported preferred slot degrades", row(rows, "reach", "openclaw")["result"] == "degraded"
            and "2.pre-turn-context-injection" in row(rows, "reach", "openclaw")["detail"])
    v.check("custom roots exclude default directories", not (home / ".hermes").exists() and not (home / ".openclaw").exists())
    copied = 0
    for destination in [shared, Path(env["HERMES_HOME"]) / "skills", Path(env["OPENCLAW_STATE_DIR"]) / "skills"]:
        for unit in ["reach", "handoff"]:
            unit_source = v.base / CATEGORY / unit
            for source in (unit_source / "skill").rglob("*"):
                target = destination / unit / source.relative_to(unit_source / "skill")
                if source.is_symlink():
                    assert target.is_symlink() and os.readlink(source) == os.readlink(target), target
                elif source.is_file():
                    assert source.read_bytes() == target.read_bytes(), target
                    copied += 1
            assert (unit_source / "PROMOTED.json").read_bytes() == (destination / unit / "PROMOTED.json").read_bytes()
            copied += 1
        for unit in ["heartbeat", "subscription-quota-check"]:
            assert not (destination / unit).exists(), destination / unit
    v.receipt["real_mapping_file_count"] = copied
    v.check("real nested payload and stamps copied to resolved roots", copied > 0)
    for consumer in ["claude,codex", "hermes", "openclaw"]:
        for unit in ["heartbeat", "subscription-quota-check"]:
            r = row(rows, unit, consumer)
            v.check(f"{consumer} {unit} unverified delivery skipped", r["result"] == "skipped" and "3.reader-to-agent-delivery" in r["detail"])
    reach_entry = home / ".local/bin/sno-reach"
    release_root = home / ".local/lib/sno-reach/releases/2.0"
    v.check("Reach entry belongs to this installed release", reach_entry.is_symlink()
            and reach_entry.resolve(strict=True) == (release_root / "bin/sno-reach").resolve(strict=True)
            and str(reach_entry) in manifest["files"]
            and str(reach_entry.resolve(strict=True)) in manifest["files"])
    version = v.run("real-reach-version", [reach_entry, "--version"], env)
    v.check("real Reach version", version.returncode == 0 and version.stdout.strip() == "2.0")
    reach_health = v.cli("doctor-real-reach", env, "doctor", 1)
    v.check("installed Reach is healthy", any(r["target"] == "reach" and r["result"] == "ok" for r in reach_health["reach"]))
    for unit in ["heartbeat", "subscription-quota-check"]:
        entry = home / ".local/bin" / unit
        help_result = v.run("real-help-" + unit, [entry, "--help"], env)
        v.check(unit + " real help", help_result.returncode == 0 and unit in help_result.stdout)
        before = v.cli("doctor-before-" + unit, env, "doctor", 1)
        v.check(unit + " healthy before removal", any(r["target"] == unit and r["result"] == "ok" for r in before["skills"]))
        target = os.readlink(entry)
        entry.unlink()
        missing = v.cli("doctor-missing-" + unit, env, "doctor", 1)
        v.check(unit + " missing detected", any(r["target"] == unit and r["result"] == "fail" for r in missing["skills"]))
        v.cli("same-release-update-" + unit, env, "update")
        v.check(unit + " owned link changed by update", entry.is_symlink() and os.readlink(entry) == target)
        restored = v.run("restored-real-help-" + unit, [entry, "--help"], env)
        v.check(unit + " restored help", restored.returncode == 0 and unit in restored.stdout)

    # Every invalid case changes only a copy of actual staged declarations. Specific
    # parser errors prevent unrelated setup failures from counting as parity.
    invalid = []
    for label, group, field, value, gate_error, cli_error in [
        ("unknown-program", "programs", "name", "not-a-program", "unknown identifier", "requires.programs unknown/duplicate name"),
        ("unknown-slot", "harness", "slot", "9.not-a-slot", "unknown identifier", "requires.harness invalid slot/need"),
        ("invalid-need", "harness", "need", "sometimes", "expected required, preferred or optional", "requires.harness invalid slot/need"),
        ("invalid-version", "programs", "min_version", ">=2.0", "malformed version", "invalid version:"),
        ("wrong-version-type", "programs", "min_version", 2, "expected a nonempty string", "requires: invalid type"),
    ]:
        req = copy.deepcopy(original["reach"])
        req[group][0][field] = value
        invalid.append((label, req, None, False, gate_error, cli_error))
    req = copy.deepcopy(original["reach"])
    req["programs"] = "wrong"
    invalid.append(("wrong-list-type", req, None, False, "expected a list", "requires: invalid type"))
    req = copy.deepcopy(original["reach"])
    req["programs"].append(copy.deepcopy(req["programs"][0]))
    invalid.append(("duplicate-entry", req, None, False, "duplicate entry", "requires.programs unknown/duplicate name"))
    invalid += [
        ("duplicate-key", None, "duplicate", False, "duplicate field", "duplicate entry"),
        ("missing-requires", None, "missing", False, "requires: missing field", "SKILL.md missing requires"),
        ("invalid-overlay", None, "missing", True, "requires: missing field", "SKILL.md missing requires"),
    ]
    for label, req, fault, overlay, gate_error, cli_error in invalid:
        tree, archive = v.variant(label, requires=req, fault=fault, overlay=overlay)
        v.gate_check("gate-" + label, tree, "reach", 1, gate_error)
        bad_home, bad_env = v.environment(label)
        v.select(bad_home, archive)
        (bad_home / ".config/sno").mkdir(parents=True)
        before = home_snapshot(bad_home)
        error = v.cli("installer-" + label, bad_env, expected=3)
        v.check(label + " intended parser refusal", error["error"] == "source_error" and cli_error in error["message"])
        lock = bad_home / ".config/sno/assemble.lock"
        v.check(label + " only excluded artifact is regular empty lock", not lock.is_symlink()
                and stat.S_ISREG(lock.lstat().st_mode) and lock.read_bytes() == b"")
        v.check(label + " complete target tree unchanged", home_snapshot(bad_home) == before)

    cases = []
    for need, expected in [("required", "skipped"), ("preferred", "degraded"), ("optional", "installed")]:
        req = copy.deepcopy(original["reach"])
        req["harness"].append({"slot": "3.reader-to-agent-delivery", "need": need})
        cases.append(("reader-" + need, "reach", req, expected, "3.reader-to-agent-delivery" if need != "optional" else ""))
    for version, expected in [("2.0.0", "installed"), ("02.00", "installed"), ("2.1", "skipped")]:
        req = copy.deepcopy(original["reach"])
        req["programs"][0]["min_version"] = version
        cases.append(("numeric-" + version, "reach", req, expected, "reach requires 2.1 installed 2.0" if expected == "skipped" else ""))
    req = copy.deepcopy(original["reach"])
    req["programs"].append({"name": "heartbeat", "min_version": "1.1"})
    cases.append(("heartbeat-too-old-with-current-reach", "reach", req, "skipped", "heartbeat requires 1.1 installed 1.0"))
    for index, name in [(0, "subscription-quota-check"), (1, "heartbeat")]:
        req = copy.deepcopy(original["subscription-quota-check"])
        req["programs"][index]["min_version"] = "1.1"
        cases.append(("quota-needs-" + name, "subscription-quota-check", req, "skipped", name + " requires 1.1 installed 1.0"))
    for label, unit, req, expected, reason in cases:
        tree, archive = v.variant(label, unit, req)
        v.gate_check("gate-" + label, tree, unit)
        case_home, case_env = v.environment(label)
        v.select(case_home, archive)
        rows = v.cli("installer-" + label, case_env)
        r = row(rows, unit)
        v.check(label + " disposition and reason", r["result"] == expected and reason in r["detail"])
        v.check(label + " placement matches disposition", (case_home / ".claude/skills" / unit / "SKILL.md").exists() == (expected != "skipped"))

    tree, archive = v.variant("valid-overlay", overlay=True)
    v.gate_check("gate-valid-overlay", tree, "reach")
    overlay_home, overlay_env = v.environment("valid-overlay")
    v.select(overlay_home, archive)
    v.cli("installer-valid-overlay", overlay_env)
    expected = tree / CATEGORY / "reach/skill/overlays/probe/SKILL.md"
    v.check("valid overlay actually copied", expected.read_bytes() == (overlay_home / ".claude/skills/reach/overlays/probe/SKILL.md").read_bytes())

    missing_home, missing_env = v.environment("missing-heartbeat")
    v.select(missing_home, omitted={"heartbeat"})
    rows = v.cli("installer-missing-heartbeat", missing_env)
    for unit in ["heartbeat", "subscription-quota-check"]:
        r = row(rows, unit)
        v.check(unit + " missing program precedes slot decision", r["result"] == "skipped" and "heartbeat missing" in r["detail"])
    v.check("omitted real program not fabricated", not (missing_home / ".local/bin/heartbeat").exists())


if __name__ == "__main__":
    main()
