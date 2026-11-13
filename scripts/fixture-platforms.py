#!/usr/bin/env python3
"""Print explicit platform exclusions only after Rust confirms their cause.

Output is a relative fixture path and reason separated by a tab. Every other
fixture remains eligible for the ordinary differential runner. No native
compiler failure can create a platform exclusion.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def confirms_linux_exclusion(diagnostics, path, source, requires):
    """Require E0433 on the declared import's `linux` token, without other errors."""
    source = source.encode()
    declaration = ("use " + requires + ";").encode()
    expected = 0
    for diagnostic in diagnostics:
        if diagnostic.get("level") != "error":
            continue
        code = diagnostic.get("code")
        spans = diagnostic.get("spans", [])
        # rustc also emits a terminal error-count summary without a code
        # or location. Every located/coded error must explain this boundary.
        if code is None and not spans and diagnostic.get("message", "").startswith("aborting due to "):
            continue
        if not isinstance(code, dict) or code.get("code") != "E0433":
            return False
        matched = False
        for span in spans:
            if not span.get("is_primary") or Path(span.get("file_name", "")).resolve() != path.resolve():
                continue
            start, end = span.get("byte_start"), span.get("byte_end")
            if not isinstance(start, int) or not isinstance(end, int) or not 0 <= start < end <= len(source):
                continue
            line_start = source.rfind(b"\n", 0, start) + 1
            line_end = source.find(b"\n", end)
            line = source[line_start:line_end if line_end != -1 else len(source)]
            if source[start:end] == b"linux" and line.strip() == declaration:
                matched = True
        if not matched:
            return False
        expected += 1
    return expected > 0


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
        data = (ROOT / path).read_bytes()
        if hashlib.sha256(data).hexdigest() != fixture.get("sha256"):
            raise ValueError("platform fixture changed; review its applicability: " + name)
        source = data.decode()
        if "use " + requires + ";" not in source:
            raise ValueError("platform requirement is no longer present: " + name)
        with tempfile.TemporaryDirectory(prefix="lust-platform-") as directory:
            probe, probe_source = ROOT / path, source
            if host not in fixture["target_os"]:
                # Test the declared requirement itself. The full Linux
                # fixture produces derivative missing-method errors on
                # other hosts; those are not applicability evidence.
                probe = Path(directory) / "platform_import.rs"
                probe_source = "use " + requires + ";\nfn main() {}\n"
                probe.write_text(probe_source)
            result = subprocess.run(["rustc", "--edition=2024", "-A", "warnings", "--emit=metadata",
                                     "--error-format=json", probe, "-o", Path(directory) / "probe.rmeta"],
                                    capture_output=True, text=True, timeout=60)
        diagnostics = []
        for line in result.stderr.splitlines():
            try:
                diagnostics.append(json.loads(line))
            except json.JSONDecodeError:
                raise RuntimeError(name + ": Rust emitted a non-JSON diagnostic: " + line[:2000])
        errors = [{"code": diagnostic.get("code", {}).get("code") if diagnostic.get("code") else None,
                   "message": diagnostic.get("message")} for diagnostic in diagnostics if diagnostic.get("level") == "error"]
        if host in fixture["target_os"]:
            if result.returncode != 0 or errors:
                raise RuntimeError(name + ": Rust rejected the full fixture on its declared host: " + json.dumps(errors))
            continue
        expected = confirms_linux_exclusion(diagnostics, probe, probe_source, requires)
        if result.returncode != 1 or not expected:
            raise RuntimeError(name + ": Rust did not confirm the declared platform exclusion: " + json.dumps(errors))
        excluded.append((name, "target_os=" + host + ": " + reason + " Rust rejects " + requires + "."))
    for name, reason in excluded:
        print(name + "\t" + reason)


if __name__ == "__main__":
    main()
