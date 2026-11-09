"""Oracle log counting for the daemon sweeps: the patch oracle (LUST_CHECK_ITEMPATCH=1,
src/driver/item_oracle.rs) and the resolve oracle (LUST_CHECK_RESOLVE=1,
src/driver/resolve_oracle.rs).

Each daemon a sweep starts writes its oracle lines to its own log (`setup`). After the daemon
stops, `collect` counts them and appends them to the caller's LUST_ITEMPATCH_LOG, if set (so
scripts/itempatch-sweep.sh sees every line). `problems` turns the counts into failures: any line
but `itempatch oracle: ok ...` or `resolve oracle: ok ...` fails, and so does a run with no `ok`
line at all (the oracle saw nothing, as on the single-file chunk path before it was covered).
"""
import os

# The start of a passing check's line, per oracle.
OK = ("itempatch oracle: ok", "resolve oracle: ok")


def on():
    """True when an oracle runs (LUST_CHECK_ITEMPATCH or LUST_CHECK_RESOLVE set and not 0)."""
    return any(os.environ.get(v, "") not in ("", "0") for v in ("LUST_CHECK_ITEMPATCH", "LUST_CHECK_RESOLVE"))


def setup(env, work):
    """Points `env`'s daemon at its own oracle log in `work`; returns the path (None when off)."""
    if not on():
        return None
    path = os.path.join(work, "oracle.log")
    env["LUST_ITEMPATCH_LOG"] = path
    env["LUST_RESOLVE_LOG"] = path
    return path


def collect(path):
    """(ok count, failure lines) of the log at `path`; appends its lines to the outer log."""
    if not path or not os.path.exists(path):
        return 0, []
    with open(path) as f:
        lines = [l.rstrip("\n") for l in f if l.strip()]
    outer = os.environ.get("LUST_ITEMPATCH_LOG")
    if outer and os.path.abspath(outer) != os.path.abspath(path) and lines:
        with open(outer, "a") as f:
            f.write("".join(l + "\n" for l in lines))
    ok = sum(l.startswith(OK) for l in lines)
    return ok, [l for l in lines if not l.startswith(OK)]


def problems(ok, fails, what):
    """Failure strings for one run: every failure line, plus a run with no check while the patch
    oracle runs (it checks every path; the resolve oracle alone sees no single-file chunk path)."""
    if not on():
        return []
    out = [f"{what}: {l[:300]}" for l in fails]
    if ok == 0 and os.environ.get("LUST_CHECK_ITEMPATCH", "") not in ("", "0"):
        out.append(f"{what}: the oracles logged 0 checks (a blind spot)")
    return out
