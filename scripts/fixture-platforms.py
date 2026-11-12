#!/usr/bin/env python3
"""Print explicit platform exclusions only after Rust confirms their cause.

Output is a relative fixture path and reason separated by a tab. Every other
fixture remains eligible for the ordinary differential runner. No native
compiler failure can create a platform exclusion.
"""
import json
from pathlib import Path
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / "tests/platforms.json").read_text())
    if manifest.get("schema") != "lust.fixture-platforms.v1":
        raise ValueError("unknown fixture platform manifest schema")
    cfg = subprocess.check_output(["rustc", "--print", "cfg"], text=True)
    matches = re.findall(r'^target_os="([a-z0-9_]+)"$', cfg, re.M)
    if len(matches) != 1:
        raise ValueError("Rust did not report one native target OS")
    host = matches[0]
    seen = set()
    excluded = []
    for fixture in manifest["fixtures"]:
        name, reason, requires = fixture["file"], fixture["reason"], fixture["requires"]
        path = Path(name)
        if path.is_absolute() or ".." in path.parts or path.parts[:2] != ("tests", "pass") or path.suffix != ".rs":
            raise ValueError("invalid platform fixture path: " + name)
        if name in seen or any(character in name + reason for character in "\t\r\n"):
            raise ValueError("duplicate or malformed platform fixture: " + name)
        seen.add(name)
        # This manifest currently records one confirmed API boundary. New
        # boundaries need an explicit Rust diagnostic check of their own.
        if fixture["target_os"] != ["linux"] or requires != "std::os::linux::net::SocketAddrExt":
            raise ValueError("unverified platform requirement: " + name)
        source = (ROOT / path).read_text()
        if "use " + requires + ";" not in source:
            raise ValueError("platform requirement is no longer present: " + name)
        if host in fixture["target_os"]:
            continue
        with tempfile.TemporaryDirectory(prefix="lust-platform-") as directory:
            result = subprocess.run(["rustc", "--edition=2024", "-A", "warnings", "--emit=metadata",
                                     "--error-format=json", ROOT / path, "-o", Path(directory) / "probe.rmeta"],
                                    capture_output=True, text=True, timeout=60)
        diagnostics = []
        for line in result.stderr.splitlines():
            try:
                diagnostics.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        expected = any(diagnostic.get("code") is not None and diagnostic["code"].get("code") == "E0433"
                       and "could not find `linux` in `os`" in diagnostic.get("message", "")
                       for diagnostic in diagnostics)
        if result.returncode != 1 or not expected:
            raise RuntimeError(name + ": Rust did not confirm the declared platform exclusion: " + result.stderr[:2000])
        excluded.append((name, "target_os=" + host + ": " + reason + " Rust rejects " + requires + "."))
    for name, reason in excluded:
        print(name + "\t" + reason)


if __name__ == "__main__":
    main()
