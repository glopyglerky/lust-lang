#!/usr/bin/env python3
# Unit tests for the --check comparison in scripts/port-scan-sync.py:
# a committed status must equal the status the probes derive, a
# supported feature must sit at milestone 0 (what --write sets), a
# probe id with no table row fails, and unprobed or manual features
# are reported as unchecked instead of passing silently.
# Usage: python3 scripts/test-port-scan-sync.py   (exit 0 = pass)
import importlib.util
import os
import tempfile
from pathlib import Path

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("port_scan_sync", os.path.join(HERE, "port-scan-sync.py"))
pss = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pss)

failures = []


def check(name, cond, detail=""):
    print(f"{'ok' if cond else 'FAIL'} {name}" + (f": {detail}" if detail and not cond else ""))
    if not cond:
        failures.append(name)


def feat(fid, status, milestone=0):
    return {"id": fid, "status": status, "milestone": milestone}


def probe(fid, verdict, file="p.rs"):
    return {"file": file, "feature": fid, "verdict": verdict}


# feature_status: the three derived statuses, plus manual.
check("all pass is supported", pss.feature_status([probe("a", "pass")]) == "supported")
check("some pass is partial", pss.feature_status([probe("a", "pass"), probe("a", "reject")]) == "partial")
check("no pass is unsupported", pss.feature_status([probe("a", "reject"), probe("a", "runfail")]) == "unsupported")
check("lust-only accept is manual", pss.feature_status([probe("a", "lust-only-accept")]) == "manual")

features = [
    feat("ok.supported", "supported", 0),
    feat("stale.partial", "partial", 5),       # probes now say supported (the ext.memchr case)
    feat("stale.milestone", "supported", 5),  # status right, milestone not what --write sets
    feat("ok.unsupported", "unsupported", 7),
    feat("no.probe", "unsupported", 7),
    feat("needs.human", "unsupported", None),
    feat("bad.probe", "partial", 5),
]
by_feature = {
    "ok.supported": [probe("ok.supported", "pass")],
    "stale.partial": [probe("stale.partial", "pass"), probe("stale.partial", "pass", "q.rs")],
    "stale.milestone": [probe("stale.milestone", "pass")],
    "ok.unsupported": [probe("ok.unsupported", "reject")],
    "needs.human": [probe("needs.human", "lust-only-accept")],
    "bad.probe": [probe("bad.probe", "pass"), probe("bad.probe", "wrong", "w.rs")],
    "not.in.table": [probe("not.in.table", "pass")],
}
derived = {fid: pss.feature_status(rs) for fid, rs in by_feature.items()}
res = pss.check_features(features, derived, by_feature)
failed_ids = [fid for fid, _msg in res["failed"]]
unchecked_ids = [fid for fid, _why in res["unchecked"]]

check("matching features pass", res["passed"] == ["ok.supported", "ok.unsupported"], str(res))
check("stale status fails", "stale.partial" in failed_ids, str(res))
check(
    "stale status message names both statuses",
    any(fid == "stale.partial" and "partial" in msg and "supported" in msg for fid, msg in res["failed"]),
    str(res["failed"]),
)
check("stale milestone fails", "stale.milestone" in failed_ids, str(res))
check("wrong-output probe fails its feature", "bad.probe" in failed_ids, str(res))
check("probe id not in table fails", "not.in.table" in failed_ids, str(res))
check("unprobed feature is unchecked, not passed", "no.probe" in unchecked_ids and "no.probe" not in res["passed"], str(res))
check("manual feature is unchecked, not failed", "needs.human" in unchecked_ids and "needs.human" not in failed_ids, str(res))

# A timeout is never a status: under gate load a lust compile can pass
# the per-step limit, and round 1 read that as a reject, which flipped
# ext.rustix to unsupported. A timed-out probe must stay unjudged.
check("timeout alone is unjudged", pss.feature_status([probe("a", "timeout")]) == "timeout")
check("pass plus timeout is unjudged, not partial", pss.feature_status([probe("a", "pass"), probe("a", "timeout")]) == "timeout")
check("reject plus timeout is unjudged, not unsupported", pss.feature_status([probe("a", "reject"), probe("a", "timeout")]) == "timeout")

tfeatures = [feat("slow.feature", "supported", 0)]
tprobes = {"slow.feature": [{"file": "slow.rs", "feature": "slow.feature", "verdict": "timeout", "detail": "lust build timed out after 30s"}]}
tderived = {fid: pss.feature_status(rs) for fid, rs in tprobes.items()}
tres = pss.check_features(tfeatures, tderived, tprobes)
check(
    "timeout is a timeout failure, not a status mismatch",
    [fid for fid, _msg in tres.get("timeouts", [])] == ["slow.feature"] and tres["failed"] == [] and tres["passed"] == [],
    str(tres),
)
check(
    "timeout message names the timed-out step",
    any("lust build timed out" in msg for _fid, msg in tres.get("timeouts", [])),
    str(tres.get("timeouts")),
)

# probe_one classification: exit 124 from a lust step is a timeout, not
# a reject. Fake every subprocess; only the lust step times out.
def fake_run_factory(lust_step):
    def fake_run(cmd, timeout, env=None, cwd=None):
        head = os.path.basename(cmd[0])
        if head == "rustc" or head == "cargo":
            return 0, "", ""
        if cmd[1:2] == [lust_step]:
            return 124, "", "timeout"
        return 0, "out\n", ""

    return fake_run


real_run = pss.run
try:
    with tempfile.TemporaryDirectory() as td:
        work = Path(td)
        plain = work / "plain.rs"
        plain.write_text("// feature: slow.plain\nfn main() {}\n")
        (work / "w1").mkdir(exist_ok=True)
        pss.run = fake_run_factory("check")
        timed = pss.probe_one(plain, Path("/bin/lust"), work / "w1", {})
        check("lust check exit 124 is a timeout, not a reject", timed["verdict"] == "timeout", str(timed))

        (work / "w2").mkdir(exist_ok=True)
        pss.run = fake_run_factory("run")
        timed = pss.probe_one(plain, Path("/bin/lust"), work / "w2", {})
        check("lust run exit 124 is a timeout, not a runfail", timed["verdict"] == "timeout", str(timed))

        cargop = work / "ext_slow.rs"
        cargop.write_text('// feature: slow.cargo\n// cargo-dep: slow = "1"\nfn main() {}\n')
        (work / "w3").mkdir(exist_ok=True)
        pss.run = fake_run_factory("build")
        timed = pss.probe_one(cargop, Path("/bin/lust"), work / "w3", {})
        check("lust build exit 124 is a timeout, not a reject", timed["verdict"] == "timeout", str(timed))

        # Under load a lust compile can also die with no diagnostic at
        # all (seen on boat for ext_rustix: a fast reject with no
        # error code). A real lust reject always carries an L####.
        def fake_lust(code_err):
            def fake_run(cmd, timeout, env=None, cwd=None):
                head = os.path.basename(cmd[0])
                if head in ("rustc", "cargo"):
                    return 0, "", ""
                if cmd[1:2] in (["check"], ["build"]):
                    return code_err[0], "", code_err[1]
                return 0, "out\n", ""

            return fake_run

        (work / "w4").mkdir(exist_ok=True)
        pss.run = fake_lust((1, "error: failed to run tcc: Resource temporarily unavailable\n"))
        timed = pss.probe_one(plain, Path("/bin/lust"), work / "w4", {})
        check("lust check with no diagnostic is a timeout, not a reject", timed["verdict"] == "timeout" and "no diagnostic" in timed["detail"], str(timed))

        (work / "w5").mkdir(exist_ok=True)
        pss.run = fake_lust((-9, ""))
        timed = pss.probe_one(plain, Path("/bin/lust"), work / "w5", {})
        check("killed lust check is a timeout, not a reject", timed["verdict"] == "timeout" and "killed by signal 9" in timed["detail"], str(timed))

        (work / "w6").mkdir(exist_ok=True)
        pss.run = fake_lust((1, "error[L0405]: `async` is not supported\n"))
        timed = pss.probe_one(plain, Path("/bin/lust"), work / "w6", {})
        check("lust check with an L code stays a reject", timed["verdict"] == "reject" and timed["lust_code"] == "L0405", str(timed))

        (work / "w7").mkdir(exist_ok=True)
        pss.run = fake_lust((-11, ""))
        timed = pss.probe_one(cargop, Path("/bin/lust"), work / "w7", {})
        check("killed lust build is a timeout, not a reject", timed["verdict"] == "timeout" and "killed by signal 11" in timed["detail"], str(timed))
finally:
    pss.run = real_run

# run_probes retry: a probe that times out in the parallel pool runs
# once more, serially, with the retry timeouts. Fake probe_one.
real_probe_one = pss.probe_one
try:
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        src = tmp / "retry.rs"
        src.write_text("// feature: slow.retry\nfn main() {}\n")
        calls = []

        def flaky(path, lust, work, env, compile_timeout=None, run_timeout=None, cargo_timeout=None):
            calls.append(compile_timeout)
            verdict = "timeout" if len(calls) == 1 else "pass"
            return {"file": path.name, "feature": "slow.retry", "form": "", "verdict": verdict, "detail": "x"}

        pss.probe_one = flaky
        import contextlib, io

        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            got = pss.run_probes([src], Path("/bin/lust"), tmp / "pool", {}, jobs=4)
        check("timed-out probe is retried once and can pass", got[0]["verdict"] == "pass" and len(calls) == 2, str(got))
        check("a judged retry is reported as slow", "slow: retry.rs timed out under load, judged alone: pass" in buf.getvalue(), buf.getvalue())
        check(
            "retry uses the long timeouts",
            calls == [pss.COMPILE_TIMEOUT, pss.RETRY_COMPILE_TIMEOUT] and pss.RETRY_COMPILE_TIMEOUT >= 180,
            str(calls),
        )
        check("retry is marked and timed", got[0].get("retried") is True and "secs" in got[0], str(got))

        calls.clear()

        def always_slow(path, lust, work, env, compile_timeout=None, run_timeout=None, cargo_timeout=None):
            calls.append(compile_timeout)
            return {"file": path.name, "feature": "slow.retry", "form": "", "verdict": "timeout", "detail": "x"}

        pss.probe_one = always_slow
        got = pss.run_probes([src], Path("/bin/lust"), tmp / "pool2", {}, jobs=4)
        check("still timed out after one retry, no third try", got[0]["verdict"] == "timeout" and len(calls) == 2, str(got))
finally:
    pss.probe_one = real_probe_one

# The planted-timeout proof sets a limit below one second, so the env
# form must accept floats; a bad value falls back to the default.
os.environ["PORT_SCAN_TEST_TINY"] = "0.05"
check("timeout env accepts a float", pss._timeout_env("TEST_TINY", 30) == 0.05)
os.environ["PORT_SCAN_TEST_TINY"] = "banana"
check("timeout env bad value uses the default", pss._timeout_env("TEST_TINY", 30) == 30.0)
del os.environ["PORT_SCAN_TEST_TINY"]
check("timeout detail keeps whole seconds plain", pss.timed_out({}, "lust build", pss.COMPILE_TIMEOUT)["detail"] == "lust build timed out after 30s")

# Coverage scan feature override: env lists become scanner flags, and
# the two forms do not mix.
check("no dev env is no override", pss.scan_feature_args({}) == [])
check(
    "exclude env becomes the scanner flag",
    pss.scan_feature_args({"LUST_DEV_EXCLUDE_FEATURES": "jemalloc"}) == ["--cargo-exclude-features", "jemalloc"],
)
check(
    "exact env becomes the scanner flag",
    pss.scan_feature_args({"LUST_DEV_FEATURES": "fast,noembed"}) == ["--cargo-features", "fast,noembed"],
)

print(f"test-port-scan-sync: {'FAILED ' + repr(failures) if failures else 'all passed'}")
raise SystemExit(1 if failures else 0)
