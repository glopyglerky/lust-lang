#!/usr/bin/env python3
"""Check that image startup works and keeps its private arena out of child processes.

Run after building lust-core. Non-Linux hosts also get a syntactically valid
Linux arena address: they must keep using the normal allocator.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    root = Path(__file__).resolve().parent.parent
    core = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / "target/release/lust-core"
    env = {key: value for key, value in os.environ.items() if not key.startswith("LUST_FRONT_")}
    env.pop("LUST_PROBE_FAKE_DIR", None)
    baseline = subprocess.run([str(core), "capabilities", "--json"], env=env, capture_output=True, text=True, timeout=30)
    if baseline.returncode != 0 or json.loads(baseline.stdout).get("kind") != "capabilities":
        raise RuntimeError(f"normal startup failed: {baseline.returncode}\n{baseline.stdout}{baseline.stderr}")

    arenas = ["invalid-arena"]
    if sys.platform != "linux":
        arenas.append("100000000000")  # First Linux image arena: 0x1000_0000_0000.
    with tempfile.TemporaryDirectory(prefix="lust-image-startup-") as tmp:
        for arena in arenas:
            child_env = dict(env, LUST_FRONT_ARENA=arena, LUST_FRONT_STATE=str(Path(tmp) / "state"))
            started = subprocess.run([str(core), "capabilities", "--json"], env=child_env, capture_output=True, text=True, timeout=30)
            if (started.returncode, started.stdout, started.stderr) != (baseline.returncode, baseline.stdout, baseline.stderr):
                raise RuntimeError(f"startup with arena {arena!r} changed: {started.returncode}\n{started.stdout}{started.stderr}")
            # The Cargo probe wrapper is an existing child-exec path. It runs
            # this observer exactly once; no compiler or emitted code is mocked.
            child_env["LUST_PROBE_FAKE_DIR"] = tmp
            child_env["LUST_STARTUP_TEST_VALUE"] = "preserved"
            observer = "import json, os; print(json.dumps([os.getenv('LUST_FRONT_ARENA'), os.getenv('LUST_STARTUP_TEST_VALUE')]))"
            child = subprocess.run([str(core), sys.executable, "-c", observer], env=child_env, capture_output=True, text=True, timeout=30)
            if child.returncode != 0:
                raise RuntimeError(f"child startup with arena {arena!r} failed: {child.returncode}\n{child.stdout}{child.stderr}")
            if json.loads(child.stdout) != [None, "preserved"]:
                raise RuntimeError(f"private arena reached child process: {child.stdout}")
            print(f"ok image startup: {sys.platform}, arena={arena}, child environment preserved")


if __name__ == "__main__":
    main()
