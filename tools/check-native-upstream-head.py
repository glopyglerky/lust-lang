#!/usr/bin/env python3
"""Check remote Lust HEAD against the explicit pin without fetching or changing it."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time
import tomllib


UPSTREAM = "https://github.com/t3dotgg/lust-lang"
TIMEOUT = 20


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_head(data: bytes) -> dict[str, str]:
    """Accept exactly one symbolic default branch and one HEAD commit."""
    refs, commits = [], []
    for line in data.decode("utf-8").splitlines():
        if not line:
            continue
        ref = re.fullmatch(r"ref: (refs/heads/[^\s]+)\tHEAD", line)
        commit = re.fullmatch(r"([0-9a-f]{40})\tHEAD", line)
        if ref:
            name = ref[1]
            parts = name.split("/")
            if (re.search(r"[\x00-\x20\x7f~^:?*\[\\]", name)
                    or ".." in name or "@{" in name or name.endswith(".")
                    or any(not part or part.startswith(".") or part.endswith(".lock") for part in parts)):
                raise ValueError("remote HEAD has an invalid default branch ref")
            refs.append(name)
        elif commit:
            commits.append(commit[1])
        else:
            raise ValueError("unexpected or invalid git ls-remote output")
    if len(refs) != 1 or len(commits) != 1:
        raise ValueError("remote HEAD must have exactly one default branch ref and one commit")
    return {"default_ref": refs[0], "revision": commits[0]}


def check(repository: Path, output: Path) -> dict:
    repository, output = repository.resolve(), output.resolve()
    if output.is_relative_to(repository):
        raise ValueError("evidence output must be outside the source checkout")
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic_ns()
    report = {"schema": "lustr.upstream-head.v1", "success": False, "state": "unavailable",
              "started_at": datetime.now(timezone.utc).isoformat(),
              "helper_sha256": digest(Path(__file__).read_bytes()),
              "scope": "Remote default-branch freshness only; no source synchronization or compiler qualification.",
              "command": {"argv": [], "cwd": str(output), "timeout_seconds": TIMEOUT,
                          "exit_code": None, "timed_out": False}, "errors": []}
    stdout, stderr = b"", b""
    try:
        pin_path = repository / "upstream/lust.pin"
        raw_pin = pin_path.read_bytes()
        (output / "pin.toml").write_bytes(raw_pin)
        pin = tomllib.loads(raw_pin.decode("utf-8"))
        report["pin"] = {"file": str(pin_path), "sha256": digest(raw_pin), **pin}
        if (pin.get("repository") != UPSTREAM
                or not re.fullmatch(r"[0-9a-f]{40}", str(pin.get("revision", "")))
                or not re.fullmatch(r"[0-9a-f]{40}", str(pin.get("tree", "")))
                or pin.get("path") != "compiler/native"):
            raise ValueError("invalid pinned Lust repository, revision, tree or source path")
        argv = ["git", "ls-remote", "--symref", pin["repository"], "HEAD"]
        report["command"]["argv"] = argv
        # An empty non-repository cwd and disabled config/credential sources keep
        # this public metadata query independent of checkout credentials/helpers.
        env = {key: value for key, value in os.environ.items()
               if not key.startswith("GIT_") and key not in ("SSH_ASKPASS", "SSH_ASKPASS_REQUIRE")}
        env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_CEILING_DIRECTORIES=str(output.parent), GIT_TERMINAL_PROMPT="0")
        command_start = time.monotonic_ns()
        with subprocess.Popen(argv, cwd=output, env=env, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, start_new_session=True) as process:
            try:
                stdout, stderr = process.communicate(timeout=TIMEOUT)
            except subprocess.TimeoutExpired:
                report["command"]["timed_out"] = True
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                stdout, stderr = process.communicate()
            report["command"]["exit_code"] = process.returncode
        report["command"]["elapsed_ns"] = time.monotonic_ns() - command_start
        if report["command"]["timed_out"]:
            raise ValueError("git ls-remote exceeded its 20-second timeout")
        if report["command"]["exit_code"] != 0:
            raise ValueError(f"git ls-remote exited with {report['command']['exit_code']}")
        report["observed"] = parse_head(stdout)
        if report["observed"]["revision"] != pin["revision"]:
            report["state"] = "drift"
            raise ValueError("remote default-branch HEAD differs from the pinned Lust revision")
        report.update(success=True, state="matched")
    except (OSError, ValueError) as error:
        report["errors"].append(f"{type(error).__name__}: {error}")
    finally:
        for name, data in (("stdout", stdout), ("stderr", stderr)):
            filename = f"command.{name}"
            (output / filename).write_bytes(data)
            report["command"][name] = filename
            report["command"][name + "_sha256"] = digest(data)
        report["elapsed_ns"] = time.monotonic_ns() - started
        (output / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = check(args.repository, args.output)
    print(json.dumps({"report": str(args.output / "report.json"), "success": report["success"],
                      "state": report["state"], "observed": report.get("observed"), "errors": report["errors"]}))
    return 0 if report["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
