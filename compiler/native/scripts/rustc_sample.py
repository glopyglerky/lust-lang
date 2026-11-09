#!/usr/bin/env python3
# rustc_sample.py: a cheap rule-zero oracle for fuzz lanes. A fuzz lane offers each program
# that lust accepts; this keeps a uniform sample (reservoir, own RNG, duplicates dropped) of
# at most N of them, then builds each with rustc at its default lint levels (a full build:
# `--emit=metadata` misses deny-by-default lints such as unconditional_panic). A rustc
# rejection is a failure: lust accepted a program that is not valid Rust. A rustc timeout is
# counted, never judged.
#   fuzz.py run --rustc-sample N   (Sample in-process)
#   rustc_sample.py N SEED OUTDIR FILE...   (parse-fuzz.sh: the files lust accepted)
# The CLI prints "rustc sample: <k> built, <r> rejected, <t> timeouts, of <a> accepted" and
# exits 1 when r > 0. Rejected programs are copied to OUTDIR as rustc-reject-<i>.rs.
#
# RunSample (fuzz.py run --rustc-run-sample N): a sample of accepted states that also ran
# warm and cold under lust, kept per semantic family (the edit op that made the state), so
# the common scalar edits cannot fill it. At the end each state is built and run with rustc,
# and its stdout, exit status and normalized stderr must equal both lust runs. A rustc run
# timeout is unjudged (the family's spare states take its place); fewer judged states than
# the minimum is a failure. Mismatches are copied to OUTDIR as rustc-run-mismatch-<i>.rs.
import concurrent.futures, hashlib, os, random, re, shutil, subprocess, sys, tempfile, time
from collections import Counter

FLAGS = ["--edition", "2024", "--crate-type", "bin", "-C", "overflow-checks=on", "-A", "warnings"]


class Sample:
    """Up to n distinct offered texts, chosen uniformly with a seeded RNG."""

    def __init__(self, n, seed):
        self.n, self.rng = n, random.Random(seed ^ 0x2C5C)
        self.seen, self.keep, self.hashes = 0, [], set()

    def offer(self, text):
        if self.n <= 0:
            return
        h = hashlib.sha256(text.encode() if isinstance(text, str) else text).digest()
        if h in self.hashes:
            return
        self.hashes.add(h)
        self.seen += 1
        if len(self.keep) < self.n:
            self.keep.append(text)
        else:
            j = self.rng.randrange(self.seen)
            if j < self.n:
                self.keep[j] = text

    def judge(self, out, timeout=120, jobs=8):
        """Builds the sample with rustc; prints one FAIL line per rejection and the summary.
        Returns the rejection count."""
        if self.n <= 0:
            return 0
        tmp = tempfile.mkdtemp(prefix="lust-rustc-sample-")

        def one(i):
            d = os.path.join(tmp, str(i))
            os.makedirs(d)
            src = os.path.join(d, "main.rs")
            text = self.keep[i]
            with open(src, "wb") as f:
                f.write(text.encode() if isinstance(text, str) else text)
            try:
                r = subprocess.run(["rustc", *FLAGS, "-o", os.path.join(d, "bin"), "main.rs"], cwd=d,
                                   capture_output=True, text=True, timeout=timeout)
            except subprocess.TimeoutExpired:
                return i, "timeout", ""
            if r.returncode == 0:
                return i, "ok", ""
            err = next((l for l in r.stderr.splitlines() if l.startswith("error")), r.stderr[:200])
            return i, "reject", err

        rejected = timeouts = 0
        try:
            with concurrent.futures.ThreadPoolExecutor(jobs) as ex:
                for i, st, err in sorted(ex.map(one, range(len(self.keep)))):
                    if st == "timeout":
                        timeouts += 1
                    elif st == "reject":
                        rejected += 1
                        os.makedirs(out, exist_ok=True)
                        keep = os.path.join(out, f"rustc-reject-{rejected}.rs")
                        shutil.copy(os.path.join(tmp, str(i), "main.rs"), keep)
                        print(f"FAIL[rustc rejects a lust accept] {keep}: {err}", flush=True)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        built = len(self.keep) - rejected - timeouts
        print(f"rustc sample: {built} built, {rejected} rejected, {timeouts} timeouts, of {self.seen} accepted",
              flush=True)
        return rejected


# The semantic family of the edit op that made a state (fuzz.py op names, before any ":").
FAMILY = {
    "const_edit": "scalar", "word_sub": "scalar",
    "insert_fn": "item", "insert_item": "item", "move_item": "item",
    "insert_snippet": "snippet",
    "delete_lines": "lines", "dup_lines": "lines", "swap_lines": "lines", "join_line": "lines",
    "split_line": "lines",
    "truncate": "bytes", "delete_bytes": "bytes", "grow_prefix": "bytes",
    "restore": "restore",
}
FAMILIES = ["scalar", "item", "snippet", "lines", "bytes", "restore", "feature"]


def family_of(op):
    base = op.split(":")[0]
    return "feature" if base.startswith(("feat_", "feat3_")) else FAMILY.get(base, "lines")


def norm_status(rc):
    """A run's exit status as a shell reports it (a signal n is 128 + n)."""
    return 128 - rc if isinstance(rc, int) and rc < 0 else rc


def norm_stdout(text):
    """A program's stdout with absolute .rs paths masked: a program that prints
    `file!()` or a panic Location prints the path it was compiled from, which is
    the fuzz crate's file for lust and this judge's temp main.rs for rustc.
    Relative paths already agree (both sides invoke with a relative main.rs)."""
    return re.sub(r"/\S*?\.rs\b", "@CASE@", text)


def norm_stderr(text):
    """A program's stderr with the parts that differ by build and not by behavior masked:
    the thread id, the panicking file's path (lust and rustc compile different paths), and
    rustc's std source prefix (scripts/case-pool.sh mask_stderr)."""
    text = re.sub(r"^thread '([^']*)' \(\d+\) ", r"thread '\1' (TID) ", text, flags=re.M)
    text = re.sub(r"panicked at [^:\n]*?([^/:\n]*):(\d+):(\d+):", r"panicked at @F:\2:\3:", text)
    return re.sub(r"(/rustc/[0-9a-f]+|/[^ :]*/lib/rustlib/src/rust)/library/", "@STD@/library/", text)


class RunSample:
    """Accepted states with their warm and cold `lust run` answers, `slots` per family plus
    `spare` more (a family's reservoir over the states offered for it)."""

    def __init__(self, n, minimum, seed, spare=2):
        self.n, self.min, self.spare = n, minimum, spare
        self.slots = max(1, -(-n // len(FAMILIES)))
        self.rng = random.Random(seed ^ 0x52554E)
        self.keep = {f: [] for f in FAMILIES}
        self.seen, self.hashes = Counter(), set()

    def want_extra(self, op):
        """Whether the fuzz loop should run this accepted state now only to offer it: while
        its family has open slots, one state in four (spread over the stream)."""
        f = family_of(op)
        return self.n > 0 and len(self.keep[f]) < self.slots + self.spare and self.rng.random() < 0.25

    def offer(self, text, op, warm, cold):
        """`warm` and `cold` are (rc, stdout, stderr) of `lust run` on `text`; a state that
        timed out on either side is not offered."""
        if self.n <= 0 or "TIMEOUT" in (warm[0], cold[0]):
            return
        h = hashlib.sha256(text.encode()).digest()
        if h in self.hashes:
            return
        self.hashes.add(h)
        f = family_of(op)
        self.seen[f] += 1
        cap, kept = self.slots + self.spare, self.keep[f]
        if len(kept) < cap:
            kept.append((text, warm, cold))
        else:
            j = self.rng.randrange(self.seen[f])
            if j < cap:
                kept[j] = (text, warm, cold)

    def judge(self, out, timeout=10, jobs=8):
        """Builds and runs every kept state with rustc and compares; prints one FAIL line per
        mismatch and the summary. Returns the failure count (mismatches, rustc rejections, and
        one for a judged count under the minimum)."""
        if self.n <= 0:
            return 0
        states = [(f, *st) for f in FAMILIES for st in self.keep[f]]
        t0 = time.time()
        tmp = tempfile.mkdtemp(prefix="lust-rustc-run-")

        def one(i):
            fam, text, warm, cold = states[i]
            d = os.path.join(tmp, str(i))
            os.makedirs(d)
            with open(os.path.join(d, "main.rs"), "w") as fh:
                fh.write(text)
            try:
                b = subprocess.run(["rustc", *FLAGS, "-o", "bin", "main.rs"], cwd=d, capture_output=True,
                                   text=True, timeout=120)
            except subprocess.TimeoutExpired:
                return i, "unjudged", "rustc build timeout"
            if b.returncode != 0:
                err = next((l for l in b.stderr.splitlines() if l.startswith("error")), b.stderr[:200])
                return i, "reject", err
            try:
                r = subprocess.run(["./bin"], cwd=d, capture_output=True, text=True, errors="replace",
                                   timeout=timeout)
            except subprocess.TimeoutExpired:
                return i, "unjudged", "rustc run timeout"
            want = (norm_status(r.returncode), norm_stdout(r.stdout), norm_stderr(r.stderr))
            for side, got in (("warm", warm), ("cold", cold)):
                have = (norm_status(got[0]), norm_stdout(got[1]), norm_stderr(got[2]))
                if have != want:
                    what = [k for k, a, b in zip(("status", "stdout", "stderr"), have, want) if a != b]
                    return i, "mismatch", f"{side} lust differs in {'/'.join(what)}: lust={have!r:.300} rustc={want!r:.300}"
            return i, "ok", ""

        judged, unjudged, bad = Counter(), 0, 0
        try:
            with concurrent.futures.ThreadPoolExecutor(jobs) as ex:
                for i, st, msg in sorted(ex.map(one, range(len(states)))):
                    fam = states[i][0]
                    if st == "ok":
                        judged[fam] += 1
                    elif st == "unjudged":
                        unjudged += 1
                    else:
                        bad += 1
                        judged[fam] += st == "mismatch"
                        os.makedirs(out, exist_ok=True)
                        keep = os.path.join(out, f"rustc-run-{st}-{bad}.rs")
                        shutil.copy(os.path.join(tmp, str(i), "main.rs"), keep)
                        kind = "rustc run differs from lust" if st == "mismatch" else "rustc rejects a lust accept"
                        print(f"FAIL[{kind}] {keep} ({fam}): {msg}", flush=True)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        total = sum(judged.values())
        fams = " ".join(f"{f}={judged[f]}" for f in FAMILIES if self.seen[f])
        print(f"rustc run sample: {total} judged ({fams}), {unjudged} unjudged, {bad} failed, "
              f"of {sum(self.seen.values())} offered, {time.time() - t0:.1f}s", flush=True)
        if total < self.min:
            print(f"FAIL[rustc run sample] only {total} judged states, need {self.min}", flush=True)
            bad += 1
        return bad


def main():
    n, seed, out, files = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3], sys.argv[4:]
    s = Sample(n, seed)
    for f in sorted(files):
        with open(f, "rb") as fh:
            s.offer(fh.read())
    return 1 if s.judge(out) else 0


if __name__ == "__main__":
    sys.exit(main())
