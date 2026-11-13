#!/usr/bin/env python3
# Unit tests for the root cause grouping of scripts/tsrust_full.py: one site counts once over
# targets, numbers are masked, fork paths are relative, crashes group by their stderr line.
# Usage: python3 scripts/test-tsrust-full.py   (exit 0 = pass)
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tsrust_full as tf  # noqa: E402

fork = Path("/w/fork")
failures = []


def diag(code, msg, file, line):
    return {"code": code, "message": msg, "file": file, "span": {"line": line, "col": 5}}


def res(target, diags, status="error", stderr=""):
    return {"target": target, "verb": "check", "status": status, "diags": diags, "stderr": stderr, "rc": 1}


def expect(name, got, want):
    if got != want:
        failures.append(name)
        print(f"FAIL {name}: got {got!r}, want {want!r}")


a = diag("L0506", "unresolved import `x::Y2`", "/w/fork/src/a.rs", 3)
b = diag("L0506", "unresolved import `x::Y7`", "/w/fork/src/b.rs", 9)
rows = tf.group([res("lib", [a, b]), res("lib:bin", [a])], fork)
expect("one cause for masked numbers", len(rows), 1)
expect("sites count once over targets", rows[0]["count"], 2)
expect("targets", rows[0]["targets"], ["lib", "lib:bin"])
expect("first span is relative", rows[0]["first"], "src/a.rs:3:5")
expect("message masked", rows[0]["message"], "unresolved import `YN`")

crash = res("t1", [], "crash", "thread 'main' panicked at src/x.rs:12:3:\nboom")
rows = tf.group([crash, dict(crash, target="t2")], fork)
expect("crashes group", [(r["code"], r["count"]) for r in rows], [("CRASH", 2)])
expect("norm path", tf.norm_message("file /w/fork/src/a.rs line 10", fork), "file src/a.rs line N")

expect("generic args merge", tf.norm_message("`collect` into `Result<Vec<(String, x::E)>, x::G>` is not implemented", fork),
       tf.norm_message("`collect` into `Result<Vec<_>, _>` is not implemented", fork))
expect("paths drop", tf.norm_message("unresolved import `rustix::fs::RawDir`", fork), "unresolved import `RawDir`")

print(f"test-tsrust-full: {'FAILED ' + repr(failures) if failures else 'all passed'}")
sys.exit(1 if failures else 0)
