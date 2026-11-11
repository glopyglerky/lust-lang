#!/usr/bin/env python3
"""Replay the independent source-race and warm-hit output-failure review.

Saved after the original inline run. The ten checks and backend instrumentation
match that run; CLI paths allow replay without replacing its retained evidence.
This is a functional diagnostic, not a performance qualification.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--native", type=Path, required=True)
parser.add_argument("--tcc-root", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
root = args.output.resolve()
root.mkdir(exist_ok=False)
package = root / "package"
(package / "bin").mkdir(parents=True)
compiler = package / "bin/lustr-native"
shutil.copy2(args.native.resolve(), compiler)
tcc = package / "lib/lust/tcc"
shutil.copytree(args.tcc_root.resolve(), tcc)
helper = tcc / "bin/tcc"
helper.rename(helper.with_name("tcc-real"))
helper.write_text("""#!/usr/bin/python3
import os,pathlib,sys,time
here=pathlib.Path(__file__).resolve()
root=here.parents[4]
with (root/'backend-invocations').open('ab',buffering=0) as log:
    log.write((str(os.getpid())+'\\n').encode())
if (root/'backend-disabled').exists():
    sys.stderr.write('test backend disabled\\n')
    sys.exit(77)
if (root/'backend-hold').exists():
    (root/'backend-entered').touch()
    deadline=time.monotonic()+30
    while not (root/'backend-released').exists():
        if time.monotonic()>deadline:
            sys.exit(78)
        time.sleep(0.005)
os.execv(str(here.with_name('tcc-real')),[str(here.with_name('tcc-real'))]+sys.argv[1:])
""")
helper.chmod(0o755)
project = root / "project"
project.mkdir()
(project / "tmp").mkdir()
source = project / "main.rs"
valid = 'unsafe fn add(value: i32) -> i32 { value + 1 }\nfn main() { println!("{}", unsafe { add(41) }); }\n'
source.write_text(valid)
command = [str(compiler), "--crate-name", "race_probe", "--crate-type", "bin", "--edition=2024",
           "--emit=link,dep-info", "--out-dir", str(project), "--error-format=json",
           "--json=artifacts,unused-externs-silent", "--force-warn=unused_crate_dependencies", str(source)]
env = {key: value for key, value in os.environ.items() if not key.startswith(("LUST_", "LUSTR_"))}
env.update(LUST_CACHE_DIR=str(project / "cache"), LUST_NO_DAEMON="1", TMPDIR=str(project / "tmp"))
report = {"schema": "lustr.native-request-cache-independent-review.v1", "success": False,
          "qualified": False, "performance_claim": False,
          "native_sha256": hashlib.sha256(compiler.read_bytes()).hexdigest(),
          "runner_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          "checks": [], "commands": [], "failures": []}
process = None

def check(value, label):
    if not value:
        raise AssertionError(label)
    report["checks"].append(label)

def record(label, result):
    report["commands"].append({"label": label, "exit_code": result.returncode,
                               "stdout": result.stdout.decode(errors="replace"),
                               "stderr": result.stderr.decode(errors="replace")})
    return result

def build(label):
    return record(label, subprocess.run(command, cwd=project, env=env, stdin=subprocess.DEVNULL,
                                        capture_output=True, timeout=30))

def markers():
    return list((project / "cache").glob("req-native-*"))

try:
    (package / "backend-hold").touch()
    process = subprocess.Popen(command, cwd=project, env=env, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    deadline = time.monotonic() + 30
    while not (package / "backend-entered").exists():
        if process.poll() is not None:
            raise AssertionError("compiler exited before blocked backend")
        if time.monotonic() > deadline:
            raise TimeoutError("backend was not reached")
        time.sleep(0.005)
    check(True, "frontend completed and actual TinyCC helper was blocked")
    source.write_text("fn main() { undefined_symbol(); }\n")
    (package / "backend-released").touch()
    stdout, stderr = process.communicate(timeout=30)
    record("compile-root-edited-during-backend", subprocess.CompletedProcess(command, process.returncode, stdout, stderr))
    check(process.returncode == 0, "in-flight compile completes its original validated source")
    result = record("run-original-source-artifact",
                    subprocess.run([project / "race_probe"], cwd=project, env=env, capture_output=True, timeout=10))
    check(result.returncode == 0 and result.stdout == b"42\n", "in-flight artifact reflects the original source")
    check(not markers(), "root edited during compilation creates no successful cache admission")
    (project / "race_probe").unlink()
    (project / "race_probe.d").unlink()
    result = build("compile-edited-invalid-root")
    check(result.returncode == 1 and not (project / "race_probe").exists() and not markers(),
          "next invocation rejects edited invalid root and creates no artifact or cache admission")
    source.write_text(valid)
    check(build("restore-valid-root").returncode == 0 and len(markers()) == 1,
          "stable validated source creates one successful admission")
    (project / "race_probe").unlink()
    (project / "race_probe.d").unlink()
    (project / "race_probe.d").mkdir()
    (package / "backend-disabled").touch()
    before = (package / "backend-invocations").read_bytes()
    result = build("warm-hit-dep-info-failure")
    check(result.returncode != 0 and not (project / "race_probe").exists(),
          "warm-hit dep-info failure removes unqualified executable")
    check((package / "backend-invocations").read_bytes() == before,
          "dep-info failure was exercised on a hit with no backend invocation")
    events = []
    for line in result.stderr.splitlines():
        try:
            events.append(json.loads(line))
        except (ValueError, UnicodeDecodeError):
            pass
    check(not any(row.get("$message_type") in ("artifact", "unused_extern") for row in events),
          "failed artifact contract emits no successful artifact or unused-extern notifications")
    (project / "race_probe.d").rmdir()
    result = build("warm-hit-after-destination-repair")
    check(result.returncode == 0 and (package / "backend-invocations").read_bytes() == before,
          "repairing output destination permits the existing valid warm hit")
    report["success"] = True
except Exception as error:
    report["failures"].append(type(error).__name__ + ": " + str(error))
finally:
    (package / "backend-released").touch()
    if process is not None and process.poll() is None:
        process.kill()
        process.communicate()
    (package / "backend-disabled").unlink(missing_ok=True)
    (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({"success": report["success"], "checks": len(report["checks"]),
                  "commands": len(report["commands"]), "failures": report["failures"],
                  "report": str(root / "report.json")}))
raise SystemExit(0 if report["success"] else 1)
