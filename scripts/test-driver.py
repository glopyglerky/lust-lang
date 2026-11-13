#!/usr/bin/env python3
# Driver tests (docs/ARCH.md section 9.3, W-DRIVER gate). Each case
# runs the real client and daemon and asserts on the compiler's own
# `lust-stats:` counters plus emitted C slot numbers:
#   1. restart with disk objects: a new daemon reuses unit objects
#      from disk and gives every fn the slot the manifest stored.
#   2. different crate load order: reordering fns in the file does not
#      renumber slots (the manifest, not file order, owns them).
#   3. one added reachable fn: unrelated units keep their objects;
#      crossing a unit boundary rebuilds only the table unit plus the
#      dirty units (section 4.2). The generic-instance form of this
#      gate joins with mono; the unit-key half is pinned by the graph
#      unit test `instance_manifest_round_trips_and_unit_keys_follow_4_2`.
#   4. reads reuse: a signature-header edit that changes no type
#      rechecks only the changed fn itself (its body is checked
#      against its own signature; seed 42) and reuses every clean
#      mentioning body through the dirty walk.
#   5. graph and build-script routing: editing a build script input
#      makes the daemon start a fresh session (its log line proves it)
#      while the program output stays correct.
#   6. std in sessions: a source std fn (std::process::exit, S5) runs
#      through the daemon twice with the same result.
#   7. resolver-path incremental: body, type, import, and new-file
#      edits in a multi-file crate recheck only the bodies whose
#      recorded reads changed, with matching program output.
#   8. slot compaction: renumbered survivors re-emit C from cached
#      IR (reemitted counter) with correct output.
#   9. restart with an added file in the middle: the persisted def
#      order keeps Adt and static numbering across a daemon
#      restart, so old units still hit the disk cache and output
#      stays exact. A corrupted def-order file is a cache miss:
#      the new daemon numbers fresh and output stays exact.
#  10. empty-slot units after a restart: two fn units that hold only
#      generic template slots have the same instance content, but each
#      registers under its own unit number; a restarted daemon must
#      not serve one unit the other's disk object.
#  11. panic positions in reused bodies: an edit that only moves text
#      (spaces inside a body, comment lines above a fn) keeps a warm
#      build's panic line and column equal to a cold build's, on the
#      single-file and the multi-file path.
#  12. generic reuse: in a program with traits, generic fns, and
#      closures, a body edit rechecks only what it touched (template
#      checks and discovery counters), check and build requests
#      alternate, and every answer equals a cold one-shot build,
#      including panic lines and columns after edits that only move
#      text (comment lines above a closure, spaces inside a body).
#      Item edits outside the trait system (a new fn, a bound added to
#      a generic fn) keep the other templates' checks, and a bound the
#      caller cannot meet rejects warm exactly as cold.
#  13. moved instance Adt ids: an edit that makes a new generic Adt
#      instance first in discovery moves the ids (and C names) of the
#      others; a warm build must rebuild the glue unit and every unit
#      whose names moved, and print what a cold build prints.
#  14. a new fn unit with no C: a 16-slot program gains an unused
#      generic fn, whose template slot starts a unit of its own; the
#      warm build must still link (every base unit needs an object).
#  15-17. nested fns: each parse mode, removed and moved nested fns,
#      instance C that calls a moved nested fn (see each case).
#  18. nested fn identity follows the parent fn, and a sibling that
#      shadows a name an unchanged nested fn calls rechecks it.
#  19. stable instance Adt ids: a new instance first in discovery and a
#      new struct regenerate only the edited fn; a restart builds no unit.
#  20. user trait impl edits leave the full generic path (ImplSet reads),
#      and so does a lang trait impl edit (LangImpls reads).
#  21. plain inherent impl edits leave it too (MethodSet reads); an
#      inherent impl with an associated const still takes it.
#  22. std modules in a dependency: edits in a source dependency add
#      and remove std source modules (std::time, VecDeque, PhantomData,
#      LazyLock, ControlFlow, Once) the root never names; every warm
#      answer equals a cold one-shot build.
#  23. fns nested in trait default bodies: each parse mode compiles them
#      (once an internal DefPathHash collision, exit 3), their identity
#      follows the trait method (an item added to another body rechecks
#      none of them), and every warm answer equals a cold one-shot build.
# Program stderr is compared with the OS thread id masked
# (mask_stderr, as in scripts/case-pool.sh): rustc 1.99 and lust
# print `thread 'main' (<os tid>) panicked at ...`, and the tid
# differs on every run, even of one binary. File, line, column, and
# message stay exact.
# Usage: python3 scripts/test-driver.py   (exit 0 = pass)
# The gate runs scripts/test-driver.sh, a thin wrapper over this file.
# Every lust compile runs under the one timeout rule (lust_run and
# Daemon.run): TEST_DRIVER_TIMEOUT seconds (default 300), one solo retry
# at 6x. A finished retry is reported as "slow"; a second timeout is a
# hang and fails its case. Program runs keep their own 60 s limit.
import errno, glob, os, re, shutil, subprocess, sys, tempfile, time, traceback

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUST = os.path.join(ROOT, "target/release/lust")
try:
    TIMEOUT = max(0.01, float(os.environ.get("TEST_DRIVER_TIMEOUT", "300")))
except ValueError:
    TIMEOUT = 300.0
failures = []
passed = 0


# lust_run <args>: one cold lust compile under the timeout rule. A timeout
# retries once, alone (this suite is serial), at 6x the limit; a second
# timeout raises AssertionError, which fails the case as a hang.
def lust_run(args, **kw):
    try:
        return subprocess.run(args, timeout=TIMEOUT, **kw)
    except subprocess.TimeoutExpired:
        print(f"slow: {' '.join(map(str, args[1:3]))} timed out at {TIMEOUT:g}s, retrying alone at {TIMEOUT * 6:g}s", flush=True)
    try:
        r = subprocess.run(args, timeout=TIMEOUT * 6, **kw)
    except subprocess.TimeoutExpired:
        raise AssertionError(f"HANG: {args} timed out alone at {TIMEOUT * 6:g}s (first at {TIMEOUT:g}s)")
    print(f"slow: {' '.join(map(str, args[1:3]))} finished alone", flush=True)
    return r


def check(name, cond, detail=""):
    global passed
    print(f"{'ok' if cond else 'FAIL'} {name}" + (f": {detail}" if detail and not cond else ""), flush=True)
    if cond:
        passed += 1
    else:
        failures.append(name)


# mask_stderr <text>: program stderr without the per-run OS thread id
# in the panic header (see the file header). Same mask as
# scripts/case-pool.sh, which difftest and runtest compare through.
def mask_stderr(text):
    return re.sub(r"^thread '([^']*)' \([0-9]+\) ", r"thread '\1' (TID) ", text, flags=re.M)


# run_prog <path>: run a built program and return the completed
# process. A just-linked binary can still be open for writing for a
# moment after the build returns, and exec then fails with ETXTBSY
# (seen on boat in case 18, where it aborted the whole suite): wait
# a short time and try again. A program still busy after 5 s, or one
# that does not end in 60 s, fails its case instead of hanging it.
def run_prog(path):
    for i in range(50):
        try:
            return subprocess.run([path], capture_output=True, text=True, timeout=60)
        except OSError as e:
            if e.errno != errno.ETXTBSY or i == 49:
                raise
            time.sleep(0.1)


class Daemon:
    def __init__(self, work):
        self.work = work
        # No inherited LUST_* settings (the list scripts/test-client.sh
        # unsets, plus LUST_STATS): a caller or a gate lane may set
        # them for another suite, and this one needs its own daemon,
        # cache, and stats.
        drop = ("LUST_DAEMON_SOCKET", "LUST_CACHE_DIR", "LUST_NO_DAEMON", "LUST_NO_CACHE",
                "LUST_CACHE_COMPILER_ID", "LUST_CACHE_MAX_BYTES", "LUST_DAEMON_IDLE",
                "LUST_THREADS", "LUST_STATS")
        self.env = {k: v for k, v in os.environ.items() if k not in drop}
        self.env.update(LUST_DAEMON_SOCKET=os.path.join(work, "s.sock"),
                        LUST_CACHE_DIR=os.path.join(work, "cache"), LUST_STATS="1",
                        LUST_DAEMON_IDLE="120s")
        self.proc = None

    def start(self):
        self.proc = subprocess.Popen([LUST, "daemon"], cwd=self.work, env=self.env,
                                     stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
        # 30 s, not 6: gate lanes load the box, and a slow start is
        # not a failure. A daemon that exits at once fails fast.
        for _ in range(1500):
            if os.path.exists(os.path.join(self.work, "s.sock")):
                return
            if self.proc.poll() is not None:
                break
            time.sleep(0.02)
        raise RuntimeError("daemon did not start")

    def kill(self):
        err = ""
        if self.proc:
            self.proc.kill()
            _, err = self.proc.communicate()
            self.proc = None
        for f in ("s.sock", "s.sock.lock"):
            try:
                os.remove(os.path.join(self.work, f))
            except FileNotFoundError:
                pass
        return err

    def run(self, args, expect=0):
        # One request under the timeout rule: TIMEOUT seconds (well over a
        # loaded gate lane's build, not the gate's step limit), then one
        # solo retry at 6x on a fresh daemon (the old one may be wedged on
        # the timed-out request). A second timeout is a hang and fails the
        # case. The retry only happens after a timeout, when the case
        # would have failed anyway, so the fresh session cannot hide a
        # passing case's reuse failure.
        try:
            r = subprocess.run([LUST] + args, cwd=self.work, env=self.env, capture_output=True, text=True, timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            print(f"slow: daemon request {args[:2]} timed out at {TIMEOUT:g}s, retrying alone at {TIMEOUT * 6:g}s on a fresh daemon", flush=True)
            self.kill()
            self.start()
            try:
                r = subprocess.run([LUST] + args, cwd=self.work, env=self.env, capture_output=True, text=True, timeout=TIMEOUT * 6)
            except subprocess.TimeoutExpired:
                raise AssertionError(f"HANG: daemon request {args} timed out alone at {TIMEOUT * 6:g}s (first at {TIMEOUT:g}s)")
            print(f"slow: daemon request {args[:2]} finished alone", flush=True)
        if expect is not None:
            assert r.returncode == expect, f"{args}: exit {r.returncode}\n{r.stderr[-600:]}"
        return r

    def build(self, src, out, extra=()):
        r = self.run(["build", src, "-o", out, "--emit-c", os.path.join(self.work, "emit"), *extra])
        stats = {}
        for line in r.stderr.splitlines():
            if line.startswith("lust-stats:"):
                for kv in line.split(":", 1)[1].split():
                    k, _, v = kv.partition("=")
                    stats[k] = int(v) if v.isdigit() else v
        prog = run_prog(out)
        return stats, prog


def slots_of(work):
    """marker name -> slot of the fn whose C body prints it. A body runs from its
    `f<slot>(void) {` header to the next fn header (under panic=unwind it holds
    cleanup blocks, so it is not brace-free)."""
    out = {}
    for p in sorted(glob.glob(os.path.join(work, "emit.u*.c"))):
        text = open(p).read()
        heads = list(re.finditer(r"\bf(\d+)\(void\) \{", text))
        for i, h in enumerate(heads):
            end = heads[i + 1].start() if i + 1 < len(heads) else len(text)
            m = re.search(r"marker-(\w+)", text[h.end():end])
            if m:
                out[m.group(1)] = int(h.group(1))
    return out


def case_slots_and_reuse():
    work = tempfile.mkdtemp(prefix="lust-driver-slots-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    # 15 fns + main = 16 slots: one fn unit at per_unit=16.
    fns = [f'fn f{i}() -> i64 {{ println!("marker-f{i}"); {i} }}' for i in range(15)]
    main = 'fn main() { println!("{}", ' + " + ".join(f"f{i}()" for i in range(15)) + "); }"
    open(src, "w").write("\n".join(fns + [main]) + "\n")
    out = os.path.join(work, "x")
    d.start()
    st, prog = d.build(src, out)
    check("cold build output", prog.stdout.strip().splitlines()[-1] == "105", prog.stdout[-80:])
    slots1 = slots_of(work)
    check("cold slots recorded", len(slots1) == 15, str(slots1))

    # 2 (in session): reorder the file. Slots follow the manifest.
    open(src, "w").write("\n".join(fns[::-1] + [main]) + "\n")
    _, prog = d.build(src, out)
    check("reorder keeps slots in session", slots_of(work) == slots1, str(slots_of(work)))
    check("reorder output", prog.stdout.strip().splitlines()[-1] == "105")

    # 3: one added reachable fn crosses into a second unit.
    open(src, "w").write('fn f15() -> i64 { println!("marker-f15"); 15 }\n' +
                         "\n".join(fns[::-1] + [main.replace("f14()", "f14() + f15()")]) + "\n")
    st, prog = d.build(src, out)
    slots3 = slots_of(work)
    check("added fn keeps old slots", all(slots3.get(k) == v for k, v in slots1.items()), str(slots3))
    check("added fn output", prog.stdout.strip().splitlines()[-1] == "120", prog.stdout[-80:])
    check("added fn rebuilds only table + dirty units",
          st.get("table_rebuilt") == 1 and (st.get("units_compiled") or 99) <= 2 + (st.get("dirty_units") or 0),
          str(st))
    check("added fn compiles few units", (st.get("units_compiled") or 99) <= 4, str(st))

    # 4: a param-name-only signature edit changes no type: the dirty
    # walk reuses the mentioners.
    apply_src = open(src).read()
    open(src, "w").write(apply_src)
    st, _ = d.build(src, out)  # settle
    d.kill()

    # 1: restart. New daemon, same cache dir: slots from the manifest,
    # objects from disk for all but the table and rerun-shaped units.
    d.start()
    st, prog = d.build(src, out)
    check("restart keeps slots (different load order)", slots_of(work) == slots3,
          f"{slots_of(work)} vs {slots3}")
    check("restart reuses disk objects", (st.get("unit_cache_hits") or 0) >= 3, str(st))
    check("restart output", prog.stdout.strip().splitlines()[-1] == "120")
    d.kill()


def case_reads_reuse():
    work = tempfile.mkdtemp(prefix="lust-driver-reads-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    base = ("struct S { compute: i64 }\n"
            "fn compute(a: i64) -> i64 { 7 }\n"
            "fn field_user() -> i64 { let s = S { compute: 1 }; s.compute }\n"
            "fn caller() -> i64 { compute(3) }\n"
            "fn main() { println!(\"{}\", caller() + field_user()); }\n")
    open(src, "w").write(base)
    out = os.path.join(work, "x")
    d.start()
    d.build(src, out)
    d.run(["check", src])  # settle read sets in the daemon session
    open(src, "w").write(base.replace("fn compute(a: i64)", "fn compute(b: i64)"))
    r = d.run(["check", src])
    st = {}
    for line in r.stderr.splitlines():
        if line.startswith("lust-stats:"):
            for kv in line.split(":", 1)[1].split():
                k, _, v = kv.partition("=")
                st[k] = int(v) if v.isdigit() else v
    check("sig-header edit is signatures mode", st.get("mode") == "signatures", str(st))
    # The changed fn (compute) always rechecks: its body is checked
    # against its own signature, which no recorded read set covers
    # (fuzz seed 42, iter 1034). Bodies that only mention it reuse.
    check("dirty walk reuses clean bodies, changed fn rechecks",
          (st.get("reused") or 0) >= 1 and st.get("rechecked") == 1, str(st))
    d.kill()


def case_graph_refresh_and_std():
    work = tempfile.mkdtemp(prefix="lust-driver-graph-")
    pkg = os.path.join(work, "pkg")
    shutil.copytree(os.path.join(ROOT, "tests/crates/buildscript"), pkg)
    d = Daemon(work)
    src = os.path.join(pkg, "src/main.rs")
    out = os.path.join(work, "x")
    extra = ("--manifest-path", os.path.join(pkg, "Cargo.toml"), "--bin", "buildscript")
    d.start()
    _, prog = d.build(src, out, extra)
    check("buildscript crate output", prog.stdout.strip() == "buildscript crate", prog.stdout)
    d.build(src, out, extra)  # warm: same session
    log = d.kill()
    check("no refresh while inputs hold", "graph or build script changed" not in log, log[-300:])
    # Edit the build script's rerun-if-changed input.
    open(os.path.join(pkg, "data.txt"), "w").write("v2\n")
    d.start()
    d.build(src, out, extra)  # first build in the new daemon seeds its key
    open(os.path.join(pkg, "data.txt"), "w").write("v3\n")
    _, prog = d.build(src, out, extra)
    log = d.kill()
    check("build script edit refreshes the session", "graph or build script changed" in log, log[-400:])
    check("output still correct after refresh", prog.stdout.strip() == "buildscript crate", prog.stdout)

    # 6: std::process::exit is a source std fn (S5 registry). Run it
    # twice through one daemon session: same exit code and output.
    ex = os.path.join(work, "exit.rs")
    open(ex, "w").write('fn main() { println!("before-exit"); std::process::exit(3); }\n')
    d.start()
    for tag in ("first", "second"):
        d.build(ex, out)
        prog = run_prog(out)
        check(f"std source fn via daemon ({tag})",
              prog.returncode == 3 and prog.stdout.strip() == "before-exit",
              f"exit {prog.returncode} out {prog.stdout!r}")
    d.kill()


def parse_stats(stderr):
    st = {}
    for line in stderr.splitlines():
        if line.startswith("lust-stats:"):
            for kv in line.split(":", 1)[1].split():
                k, _, v = kv.partition("=")
                st[k] = int(v) if v.isdigit() else v
    return st


def case_multi_incremental():
    # 7. Resolver-path incremental (W-DRIVER phase 2): a multi-file
    # crate in one daemon session. A body edit in one file rechecks
    # one body; a type edit rechecks only the bodies that read the
    # type (dirty walk in full mode); an import edit that changes a
    # binding rechecks its readers; a new file adds one body.
    work = tempfile.mkdtemp(prefix="lust-driver-multi-")
    d = Daemon(work)
    main_rs = os.path.join(work, "main.rs")
    a_rs = os.path.join(work, "a.rs")
    b_rs = os.path.join(work, "b.rs")
    out = os.path.join(work, "x")
    open(main_rs, "w").write(
        "mod a;\nmod b;\n\nuse crate::a::value as av;\n\n"
        "fn main() {\n    let p = a::Point { x: av(), y: b::value() };\n"
        "    println!(\"{}\", p.sum() + b::other());\n}\n")
    open(a_rs, "w").write(
        "pub struct Point {\n    pub x: i64,\n    pub y: i64,\n}\n\n"
        "impl Point {\n    pub fn sum(&self) -> i64 {\n        self.x + self.y\n    }\n}\n\n"
        "pub fn value() -> i64 {\n    3\n}\n")
    open(b_rs, "w").write(
        "pub fn value() -> i64 {\n    4\n}\n\npub fn other() -> i64 {\n    10\n}\n")
    d.start()
    _, prog = d.build(main_rs, out)
    check("multi cold output", prog.stdout.strip() == "17", prog.stdout)

    def edit(path, old, new):
        text = open(path).read()
        assert old in text, f"edit not found in {path}: {old!r}"
        open(path, "w").write(text.replace(old, new, 1))

    # Body edit in one file: one body rechecks, output tracks it.
    edit(b_rs, "    4\n", "    5\n")
    st = parse_stats(d.run(["check", main_rs]).stderr)
    check("multi body edit rechecks one body", st.get("mode") == "bodies" and st.get("rechecked") == 1, str(st))
    _, prog = d.build(main_rs, out)
    check("multi body edit output", prog.stdout.strip() == "18", prog.stdout)

    # Type edit (field order swap in a.rs): full mode, but the dirty
    # walk reuses the bodies that never read Point's layout (both
    # b.rs bodies); a.rs's method and main (the literal) recheck.
    edit(a_rs, "    pub x: i64,\n    pub y: i64,", "    pub y: i64,\n    pub x: i64,")
    st = parse_stats(d.run(["check", main_rs]).stderr)
    check("multi type edit is full mode with reuse",
          st.get("mode") == "full" and (st.get("reused") or 0) >= 2 and (st.get("rechecked") or 99) <= 3, str(st))
    _, prog = d.build(main_rs, out)
    check("multi type edit output", prog.stdout.strip() == "18", prog.stdout)

    # Import edit: point the alias at the other module's fn. The
    # binding main read changes, so main rechecks and output moves.
    edit(main_rs, "use crate::a::value as av;", "use crate::b::value as av;")
    st = parse_stats(d.run(["check", main_rs]).stderr)
    check("multi import edit rechecks its reader", (st.get("rechecked") or 0) >= 1, str(st))
    _, prog = d.build(main_rs, out)
    check("multi import edit output", prog.stdout.strip() == "20", prog.stdout)

    # New file: one new body checks, the rest of the crate stands.
    open(os.path.join(work, "c.rs"), "w").write("pub fn extra() -> i64 {\n    1\n}\n")
    edit(main_rs, "p.sum() + b::other()", "p.sum() + b::other() + c::extra()")
    edit(main_rs, "mod a;", "mod c;\nmod a;")
    st = parse_stats(d.run(["check", main_rs]).stderr)
    check("multi new file rechecks only new and edited bodies",
          (st.get("rechecked") or 99) <= 2, str(st))
    _, prog = d.build(main_rs, out)
    check("multi new file output", prog.stdout.strip() == "21", prog.stdout)
    d.kill()


def case_compaction_reemit():
    # 8. Slot compaction re-emits C from cached IR (section 4.2):
    # delete most fns and edit a type in the same step (full mode,
    # dead slots past the compaction threshold). Survivors renumber;
    # their C re-emits from cached IR instead of rechecking, and the
    # program still prints the right answer.
    work = tempfile.mkdtemp(prefix="lust-driver-compact-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    fillers = [f"fn filler{i}() -> i64 {{ {i} }}" for i in range(100)]
    head = "struct S { x: i64 }\nfn keep(a: i64) -> i64 { a + 1 }\n"
    main = 'fn main() { let s = S { x: 40 }; println!("{}", keep(s.x) + keep(1)); }\n'
    # Fillers first, so the survivors' slots actually move when the
    # dead fillers compact away.
    open(src, "w").write("\n".join(fillers) + "\n" + head + main)
    d.start()
    _, prog = d.build(src, out)
    check("compaction cold output", prog.stdout.strip() == "43", prog.stdout)
    # Full-mode edit: type change plus deleting 95 of 100 fillers.
    keep = "\n".join(fillers[:5])
    open(src, "w").write("struct S { x: i64, y: i64 }\nfn keep(a: i64) -> i64 { a + 1 }\n" + keep + "\n" +
                         'fn main() { let s = S { x: 40, y: 1 }; println!("{}", keep(s.x) + keep(1) + s.y); }\n')
    st = parse_stats(d.run(["check", src]).stderr)
    check("compaction edit is full mode", st.get("mode") == "full", str(st))
    st, prog = d.build(src, out)
    check("compaction re-emits from cached IR", (st.get("reemitted") or 0) >= 1, str(st))
    check("compaction output", prog.stdout.strip() == "44", prog.stdout)
    d.kill()


def case_restart_add_file():
    # 9. Restart with a file added in the middle (W-DRIVER phase 4):
    # the def order persists to disk, so a restarted daemon numbers
    # Adts and statics as the old daemon did. The new file's defs
    # append; old units keep valid indices and hit the disk cache.
    work = tempfile.mkdtemp(prefix="lust-driver-restart-add-")
    d = Daemon(work)
    main_rs = os.path.join(work, "main.rs")
    out = os.path.join(work, "x")
    open(main_rs, "w").write(
        "mod a;\nmod z;\n\nfn main() {\n    println!(\"{}\", a::get() + z::get());\n}\n")
    open(os.path.join(work, "a.rs"), "w").write(
        "pub struct Alpha {\n    pub x: i64,\n    pub y: i64,\n}\n\n"
        "pub static ABASE: i64 = 100;\n\n"
        "pub fn get() -> i64 {\n    let a = Alpha { x: 1, y: 2 };\n    a.x * 10 + a.y + ABASE\n}\n")
    open(os.path.join(work, "z.rs"), "w").write(
        "pub struct Zed {\n    pub x: i64,\n    pub y: i64,\n}\n\n"
        "pub static ZBASE: i64 = 1000;\n\n"
        "pub fn get() -> i64 {\n    let z = Zed { x: 3, y: 4 };\n    z.x * 10 + z.y + ZBASE\n}\n")
    d.start()
    _, prog = d.build(main_rs, out)
    check("restart-add cold output", prog.stdout.strip() == "1146", prog.stdout)
    d.kill()

    def_order_files = glob.glob(os.path.join(work, "cache", "**", "defs-*.bin"), recursive=True)
    check("def order persisted to disk", len(def_order_files) == 1, str(def_order_files))

    # New file in the middle of the mod list, with new Adts and a
    # static, then a fresh daemon on the same cache dir.
    open(os.path.join(work, "m.rs"), "w").write(
        "pub struct Mid1 {\n    pub x: i64,\n    pub y: i64,\n}\n\n"
        "pub struct Mid2 {\n    pub x: i64,\n    pub y: i64,\n}\n\n"
        "pub struct Mid3 {\n    pub x: i64,\n    pub y: i64,\n}\n\n"
        "pub static MBASE: i64 = 7;\n\n"
        "pub fn get() -> i64 {\n    let a = Mid1 { x: 0, y: 0 };\n    let b = Mid2 { x: 5, y: 6 };\n"
        "    let c = Mid3 { x: 0, y: 0 };\n    a.x + b.x * 10 + b.y + c.y + MBASE\n}\n")
    text = open(main_rs).read()
    open(main_rs, "w").write(text.replace("mod a;\nmod z;", "mod a;\nmod m;\nmod z;")
                             .replace("a::get() + z::get()", "a::get() + m::get() + z::get()"))
    d.start()
    st, prog = d.build(main_rs, out)
    check("restart-add output after restart", prog.stdout.strip() == "1209", prog.stdout)
    check("restart-add old units hit the disk cache", (st.get("unit_cache_hits") or 0) >= 1, str(st))
    d.kill()

    # A corrupted def-order file falls back to fresh numbering:
    # output stays exact (only hits may drop).
    for path in glob.glob(os.path.join(work, "cache", "**", "defs-*.bin"), recursive=True):
        open(path, "wb").write(b"garbage, not a def order")
    d.start()
    _, prog = d.build(main_rs, out)
    check("corrupt def order falls back, output exact", prog.stdout.strip() == "1209", prog.stdout)
    d.kill()


def case_empty_units_restart():
    # 10. 40 generic fns first in the file: at 16 slots per unit, the
    # first two fn units hold only template slots (no C content).
    work = tempfile.mkdtemp(prefix="lust-driver-empty-units-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    fns = [f"fn g{i}<T: Copy>(t: T) -> T {{ t }}" for i in range(40)]
    main = 'fn main() { println!("{}", ' + " + ".join(f"g{i}({i}i64)" for i in range(40)) + "); }"
    open(src, "w").write("\n".join(fns + [main]) + "\n")
    out = os.path.join(work, "x")
    d.start()
    _, prog = d.build(src, out)
    check("empty units cold output", prog.stdout.strip() == "780", prog.stdout)
    d.kill()
    d.start()
    r = d.run(["build", src, "-o", out], expect=None)
    check("empty units restart build", r.returncode == 0, r.stderr[-300:])
    prog = run_prog(out)
    check("empty units restart output", prog.stdout.strip() == "780", prog.stdout)
    d.kill()


def case_positions():
    # 11. Body hashes cover each fn's own text, not only its tokens.
    work = tempfile.mkdtemp(prefix="lust-driver-positions-")
    d = Daemon(work)
    main_rs = os.path.join(work, "main.rs")
    a_rs = os.path.join(work, "a.rs")
    single = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    open(single, "w").write("fn other() -> u8 {\n    250\n}\n\nfn main() {\n    let a = other();\n    let r = a + 9;\n    println!(\"{}\", r);\n}\n")
    open(main_rs, "w").write("mod a;\nfn main() {\n    println!(\"{}\", a::f(250));\n}\n")
    open(a_rs, "w").write("pub fn f(a: u8) -> u8 {\n    let r = a + 9;\n    r\n}\n")

    def edit(path, old, new):
        text = open(path).read()
        assert old in text, old
        open(path, "w").write(text.replace(old, new, 1))

    def cold(src):
        env = dict(d.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(work, f"cold{len(os.listdir(work))}"))
        cout = os.path.join(work, "xc")
        lust_run([LUST, "build", src, "-o", cout], cwd=work, env=env, capture_output=True)
        return run_prog(cout).stderr

    d.start()
    for src, path, old, new in [(single, single, "    let r = a + 9;", "    let r =     a + 9;"),
                                (single, single, "fn other() -> u8 {", "// one\n\nfn other() -> u8 {"),
                                (main_rs, a_rs, "    let r = a + 9;", "    let r =   a + 9;")]:
        d.build(src, out)
        edit(path, old, new)
        _, prog = d.build(src, out)
        check(f"positions after {new.strip()!r}: warm panic equals cold", mask_stderr(prog.stderr) == mask_stderr(cold(src)), prog.stderr)
    d.kill()


def case_generic_reuse():
    # 12. Generic reuse (driver/gen_reuse.rs).
    work = tempfile.mkdtemp(prefix="lust-driver-generic-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    text = (
        "trait Score {\n    fn score(&self) -> u8;\n    fn twice(&self) -> u8 {\n        self.score() * 2\n    }\n}\n\n"
        "struct A {\n    v: u8,\n}\n\nimpl Score for A {\n    fn score(&self) -> u8 {\n        self.v\n    }\n}\n\n"
        "fn total<T: Score>(xs: &[T]) -> u8 {\n    let mut s = 0;\n    for x in xs {\n        s += x.twice();\n    }\n    s\n}\n\n"
        "fn apply<F: Fn(u8) -> u8>(f: F, x: u8) -> u8 {\n    f(x)\n}\n\n"
        "fn other() -> u8 {\n    1\n}\n\n"
        "fn main() {\n    let xs = [A { v: 3 }, A { v: 4 }];\n    let t = total(&xs);\n    let k = 2;\n"
        "    let r = apply(|v| v + 240 + k, t + other());\n    println!(\"{} {}\", t, r);\n}\n")
    open(src, "w").write(text)

    def cold():
        env = dict(d.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(work, f"cold{len(os.listdir(work))}"))
        cout = os.path.join(work, "xc")
        lust_run([LUST, "build", src, "-o", cout], cwd=work, env=env, capture_output=True)
        r = run_prog(cout)
        return r.returncode, r.stdout, mask_stderr(r.stderr)

    def edit(old, new):
        text = open(src).read()
        assert old in text, old
        open(src, "w").write(text.replace(old, new, 1))

    def same(name, st, prog):
        check(f"generic {name}: warm equals cold", (prog.returncode, prog.stdout, mask_stderr(prog.stderr)) == cold(),
              f"{prog.stdout}{prog.stderr} {st}")

    d.start()
    st, prog = d.build(src, out)
    same("cold build", st, prog)
    # A template body edit: one template check, no plain fn rechecked.
    edit("        s += x.twice();", "        s += x.twice() + 1;")
    r = d.run(["check", src])
    cst = {k: int(v) for k, _, v in (kv.partition("=") for kv in r.stderr.split("lust-stats:")[-1].split()) if v.isdigit()}
    check("generic template edit checks one template", cst.get("templates") == 1 and cst.get("discovered") == 0, str(cst))
    st, prog = d.build(src, out)
    same("template edit", st, prog)
    check("generic template edit rebuilds only its instances", (st.get("rechecked") or 99) <= 2, str(st))
    # Comment lines above `other` move the panic in main's closure down.
    edit("fn other() -> u8 {", "// one\n// two\nfn other() -> u8 {")
    st, prog = d.build(src, out)
    same("lines moved above a closure", st, prog)
    check("generic moved lines recheck nothing", st.get("rechecked") == 0, str(st))
    # Spaces inside the closure move the panic column.
    edit("|v| v + 240 + k", "|v|   v + 240 + k")
    st, prog = d.build(src, out)
    same("spaces inside a body", st, prog)
    # An item edit seen only by a check (full mode, caches cleared, no
    # generic fn instances made), then a body edit, then a build: the
    # instance slots of the trait default method must not come back
    # as passing with no C.
    edit("struct A {", "struct Unused {\n    n: u8,\n}\n\nstruct A {")
    d.run(["check", src])
    edit("    1\n}", "    2\n}")
    d.run(["check", src])
    st, prog = d.build(src, out)
    same("item edit, check, body edit, check, build", st, prog)

    def check_stats():
        r = d.run(["check", src], expect=None)
        line = r.stderr.split("lust-stats:")[-1].split("\n")[0]
        return r, {k: int(v) for k, _, v in (kv.partition("=") for kv in line.split()) if v.isdigit()}

    # A new fn: an item edit outside the trait system. No template
    # reads the new name, so every template check is reused.
    edit("fn other() -> u8 {", "fn added() -> u8 {\n    9\n}\n\nfn other() -> u8 {")
    _, cst = check_stats()
    check("generic new fn reuses template checks", cst.get("templates") == 0, str(cst))
    st, prog = d.build(src, out)
    same("new fn", st, prog)
    # A bound the caller meets, then one it cannot meet: the warm
    # answer must reject exactly as a cold check does (rule zero).
    edit("fn total<T: Score>(", "fn total<T: Score + Sized>(")
    st, prog = d.build(src, out)
    same("bound added, caller meets it", st, prog)
    edit("fn total<T: Score + Sized>(", "fn total<T: Score + Copy>(")
    r, _ = check_stats()
    env = dict(d.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(work, "cold-reject"))
    c = lust_run([LUST, "check", src], cwd=work, env=env, capture_output=True, text=True)
    strip = lambda t: "\n".join(l for l in t.splitlines() if not l.startswith("lust-stats:"))
    check("generic unmet bound rejects warm as cold", (r.returncode, strip(r.stderr)) == (c.returncode, strip(c.stderr)) and r.returncode != 0,
          f"warm {r.returncode} {r.stderr[-300:]} cold {c.returncode} {c.stderr[-300:]}")
    # A type error in a template body rejects warm as cold; fixing it is
    # a body edit again (the error kept the item state).
    edit("fn total<T: Score + Copy>(", "fn total<T: Score>(")
    d.run(["check", src], expect=None)
    edit("        s += x.twice() + 1;", "        s += x.twice() + true;")
    r, _ = check_stats()
    c = lust_run([LUST, "check", src], cwd=work, env=env, capture_output=True, text=True)
    check("generic template body error rejects warm as cold", (r.returncode, strip(r.stderr)) == (c.returncode, strip(c.stderr)) and r.returncode != 0,
          f"warm {r.returncode} {r.stderr[-300:]} cold {c.returncode} {c.stderr[-300:]}")
    edit("        s += x.twice() + true;", "        s += x.twice() + 1;")
    r, cst = check_stats()
    check("generic fix after a template error is a body edit", r.returncode == 0 and cst.get("rechecked", 99) <= 2, f"{r.returncode} {cst}")
    st, prog = d.build(src, out)
    same("fix after a template error", st, prog)
    d.kill()


def case_moved_instance_ids():
    # 13. W<u8> and W<u16> Debug glue is each shared by several units;
    # the edit adds W<u32> at the top of the first fn.
    work = tempfile.mkdtemp(prefix="lust-driver-moved-ids-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    fns = ["#[derive(Debug, Clone)]\nstruct W<T> {\n    a: T,\n    b: u8,\n}\n",
           "fn f0() -> usize {\n    let w = W { a: 5u8, b: 6 };\n    format!(\"{:?}\", w).len()\n}\n"]
    fns += [f"fn f{i}() -> usize {{\n    let w = W {{ a: {i}u8, b: 9 }};\n    format!(\"{{:?}}\", w).len()\n}}\n" for i in range(1, 49)]
    fns += [f"fn f{i}() -> usize {{\n    let w = W {{ a: {i}u16, b: 3 }};\n    format!(\"{{:?}}\", w).len()\n}}\n" for i in range(49, 97)]
    main = ("fn main() {\n    let mut t = 0;\n" + "".join(f"    t += f{i}();\n" for i in range(97))
            + "    println!(\"{}\", t);\n    println!(\"{:?}\", W { a: 1u8, b: 2 });\n}\n")
    open(src, "w").write("\n".join(fns) + main)
    d.start()
    d.build(src, out)
    text = open(src).read()
    open(src, "w").write(text.replace("fn f0() -> usize {\n", "fn f0() -> usize {\n    let z = W { a: 70000u32, b: 1 };\n"
                                      "    let _ = format!(\"{:?}\", z);\n", 1))
    _, prog = d.build(src, out)
    env = dict(d.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(work, "cold"))
    lust_run([LUST, "build", src, "-o", out + "c"], cwd=work, env=env, capture_output=True)
    cold = run_prog(out + "c")
    check("moved instance ids: warm equals cold", prog.stdout == cold.stdout and prog.returncode == cold.returncode,
          f"warm {prog.stdout[-60:]!r} cold {cold.stdout[-60:]!r}")
    d.kill()


def case_new_empty_unit():
    # 14. 15 fns + main fill one 16-slot unit; the template is slot 17.
    work = tempfile.mkdtemp(prefix="lust-driver-empty-unit-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    fns = [f"fn f{i}() -> i64 {{ {i} }}" for i in range(15)]
    main = 'fn main() { println!("{}", ' + " + ".join(f"f{i}()" for i in range(15)) + "); }"
    open(src, "w").write("\n".join(fns + [main]) + "\n")
    d.start()
    d.build(src, out)
    open(src, "a").write("fn g<T: Copy>(t: T) -> T {\n    t\n}\n")
    r = d.run(["build", src, "-o", out], expect=None)
    prog = run_prog(out)
    check("new template-only unit links", r.returncode == 0 and prog.stdout.strip() == "105", r.stderr[-200:] + prog.stdout)
    d.kill()


def case_nested_items():
    # 15. A single file whose fns hold nested fns, edited in each parse
    # mode. Bodies and signatures mode parsed into the old Ast and kept
    # the replaced nested fn, so the program had one fn more than slots:
    # builds panicked (the daemon answered from a fresh session), and a
    # check never checked the new copy's body (warm false accept).
    work = tempfile.mkdtemp(prefix="lust-driver-nested-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    base = ("fn a(x: i64) -> i64 {\n    fn na() -> i64 { 1 }\n    na() + x\n}\n\n"
            "fn b() -> i64 {\n    fn nb() -> i64 { 20 }\n    nb()\n}\n\n"
            "fn main() {\n    println!(\"{} {}\", a(0), b());\n}\n")
    d.start()
    # Checks only, first in the session: the nested body no longer fits
    # its signature (`b` prints it, which fits both), and cold rejects.
    printed = base.replace("    nb()\n}", "    println!(\"{}\", nb());\n    20\n}")
    open(src, "w").write(printed)
    d.run(["check", src])
    open(src, "w").write(printed.replace("fn nb() -> i64", "fn nb() -> bool"))
    c = d.run(["check", src], expect=None)
    check("nested items, nested signature only: rejects", c.returncode == 1, c.stderr[-200:])
    edits = [
        ("outer body", [("na() + x", "na() + x + 2")], "3 20"),
        ("nested body", [("{ 20 }", "{ 30 }")], "1 30"),
        ("outer signature", [("fn a(x: i64)", "fn a(x: i64, _y: i64)"), ("a(0)", "a(0, 0)")], "1 20"),
        ("later outer signature", [("fn b()", "fn b(_y: i64)"), (", b()", ", b(0)")], "1 20"),
    ]
    for name, subs, want in edits:
        text = base
        for old, new in subs:
            text = text.replace(old, new)
        for step, body, expect in ((f"{name}: base", base, "1 20"), (name, text, want)):
            open(src, "w").write(body)
            c = d.run(["check", src], expect=None)
            b = d.run(["build", src, "-o", out], expect=None)
            prog = run_prog(out) if b.returncode == 0 else None
            got = prog.stdout.strip() if prog else b.stderr[-200:]
            check(f"nested items, {step}", c.returncode == 0 and got == expect, f"check {c.returncode}: {got!r}")
    err = d.kill()
    check("nested items: no warm panic", "panicked" not in err, err[-300:])


def case_moved_and_revived_slots():
    # 16. A nested fn's identity holds its `#nested{k}` stmt index.
    # (a) A removed fn's slot kept its body hash while marked passing
    # with no C, so the same fn put back was reused unchecked: a check
    # that cold rejects passed warm. (b) An item added to an earlier
    # body moves every later nested fn to a new slot while its parent's
    # text stays: the parent's reused C called the dead old slots, and
    # the warm binary crashed.
    work = tempfile.mkdtemp(prefix="lust-driver-moved-slots-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    d.start()
    bad = "fn f() -> i64 {\n    fn g() -> bool { 5 }\n    1\n}\n\nfn main() {\n    println!(\"{}\", f());\n}\n"
    codes = []
    for text in (bad, bad.replace("    fn g() -> bool { 5 }\n", ""), bad):
        open(src, "w").write(text)
        codes.append(d.run(["check", src], expect=None).returncode)
    check("revived nested fn rechecks", codes == [1, 0, 1], str(codes))

    def tree(first):
        # The const routes the file through the resolver path.
        parts = []
        for name, k, nested in (("f0", 9, first), ("f1", 5, [2, 1]), ("f2", 5, [4, 4]), ("f3", 9, [8, 7])):
            lines = [f"fn {name}() -> i64 {{"] + [f"    fn {name}_g{j}() -> i64 {{ {v} }}" for j, v in enumerate(nested)]
            terms = [str(k)] + [f"{name}_g{j}()" for j in range(len(nested))]
            if name == "f1":
                lines.append("    const K: i64 = 6;")
                terms.append("K")
            parts.append("\n".join(lines + ["    " + " + ".join(terms), "}"]))
        return "\n\n".join(parts + ['fn main() {\n    println!("{}", f0() + f1() + f2() + f3());\n}']) + "\n"

    for name, text, want in (("moved nested slots: base", tree([]), "60"), ("moved nested slots", tree([7]), "67")):
        open(src, "w").write(text)
        r = d.run(["build", src, "-o", out], expect=None)
        prog = run_prog(out) if r.returncode == 0 else None
        got = (prog.returncode, prog.stdout.strip()) if prog else r.stderr[-200:]
        check(name, got == (0, want), str(got))
    d.kill()


def undefined_calls(prefix):
    # Slots the emitted C calls that no emitted unit defines (the table
    # size declaration `lust_fnptr lust_fns[N];` is not a call).
    defined, called = set(), set()
    for p in glob.glob(prefix + ".u*.c"):
        text = open(p).read()
        defined |= {int(n) for n in re.findall(r"^static [^\n(]*\bf(\d+)\(", text, re.M)}
        called |= {int(n) for n in re.findall(r"(?<!lust_fnptr )\blust_(?:fns|tramps)\[(\d+)\]", text)}
    return sorted(called - defined)


def case_instance_calls_moved_slot():
    # 17. A generic fn's nested fn moves to a new slot (`#nested{k}`)
    # while the template text stays, here because another nested fn
    # moves between two other bodies (bodies mode on the resolver
    # path). The reused instance C called the dead old slot; the old
    # base often still held that fn, so the binary only crashed
    # sometimes. The emitted C must call only slots it defines.
    work = tempfile.mkdtemp(prefix="lust-driver-inst-moved-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    head = ("use std::fmt::Debug;\n\ntrait Tr {\n    fn t(&self) -> i64;\n}\n\n"
            "impl Tr for i64 {\n    fn t(&self) -> i64 {\n        *self\n    }\n}\n\n")
    gen = "fn f1<T: Tr>(tv: T) -> i64 {\n    fn g1() -> i64 { 2 }\n    tv.t() + g1()\n}\n\n"
    tail = 'fn main() {\n    println!("{}", f0() + f1(5i64) + f2());\n}\n'
    z = "    fn z() -> i64 { 7 }\n"
    before = head + "fn f0() -> i64 {\n" + z + "    9 + z()\n}\n\n" + gen + "fn f2() -> i64 {\n    5\n}\n\n" + tail
    after = head + "fn f0() -> i64 {\n    9\n}\n\n" + gen + "fn f2() -> i64 {\n" + z + "    5 + z()\n}\n\n" + tail
    d.start()
    for name, text in (("base", before), ("nested fn moved", after)):
        open(src, "w").write(text)
        emit = os.path.join(work, name.replace(" ", "-"))
        r = d.run(["build", src, "-o", out, "--emit-c", emit], expect=None)
        prog = run_prog(out) if r.returncode == 0 else None
        got = (prog.returncode, prog.stdout.strip()) if prog else r.stderr[-200:]
        check(f"instance calls moved slot, {name}", got == (0, "28") and not undefined_calls(emit), f"{got} undefined {undefined_calls(emit)}")
    d.kill()


def case_nested_identity_and_shadowing():
    # 18. A nested fn's identity is its parent fn's plus its name (and an
    # ordinal among same-name fns of that parent). (a) An item added to
    # one fn's body keeps the identity of the fns nested in other bodies,
    # so a check rechecks only the edited fn and its own nested fns. (b)
    # A nested fn reads the names of the blocks around it with no
    # recorded read: an added sibling `h` that shadows the top-level `h`
    # the unchanged nested `g` calls must recheck `g` (warm kept `g` and
    # its C: a false accept and wrong output), single-file and multi-file.
    work = tempfile.mkdtemp(prefix="lust-driver-nested-id-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    d.start()

    def tree(extra):
        parts = ["fn f0() -> i64 {\n    fn g() -> i64 { 1 }\n" + extra + "    g()\n}"]
        for i in range(1, 4):
            parts.append(f"fn f{i}() -> i64 {{\n    fn g() -> i64 {{ {i} }}\n    fn h() -> i64 {{ 10 }}\n    g() + h()\n}}")
        return "\n\n".join(parts + ['fn main() {\n    println!("{}", f0() + f1() + f2() + f3());\n}']) + "\n"

    open(src, "w").write(tree(""))
    d.run(["check", src])
    open(src, "w").write(tree("    fn e() -> i64 { 5 }\n"))
    r = d.run(["check", src])
    stats = parse_stats(r.stderr)
    # f0, its nested g (its enclosing body changed), and the new e.
    check("nested identity: other bodies' nested fns keep their slots", stats.get("rechecked") == 3, str(stats))

    top = "fn h() -> i32 { 1 }\n"
    base = top + "fn f() -> i32 {\n    fn g() -> i32 { h() }\n    g()\n}\nfn main() { println!(\"{}\", f()); }\n"
    shadow = base.replace("    g()\n", "    fn h() -> i32 { 2 }\n    g()\n")
    bad = base.replace("    g()\n", "    fn h() -> bool { true }\n    g()\n")
    mwork = os.path.join(work, "m")
    os.makedirs(os.path.join(mwork, "src"))
    main_rs = os.path.join(mwork, "src", "main.rs")
    open(main_rs, "w").write("mod a;\nfn main() { println!(\"{}\", a::f()); }\n")
    for path, entry, wrap in ((src, src, lambda t: t),
                              (os.path.join(mwork, "src", "a.rs"), main_rs, lambda t: t.split("fn main")[0].replace("fn f()", "pub fn f()"))):
        kind = "multi-file" if entry == main_rs else "single-file"
        for step, text, want in (("base", base, "1"), ("shadowing sibling", shadow, "2"), ("shadowing sibling of another type", bad, None), ("back", base, "1")):
            open(path, "w").write(wrap(text))
            out = os.path.join(work, "x")
            c = d.run(["check", entry], expect=None)
            b = d.run(["build", entry, "-o", out], expect=None)
            prog = run_prog(out) if b.returncode == 0 else None
            got = (c.returncode, prog.stdout.strip() if prog else None)
            check(f"nested shadowing, {kind}, {step}", got == ((0, want) if want else (1, None)), str(got))
    d.kill()


def case_stable_instance_ids():
    # 19. Instance Adt ids are session places (collect::InstOrder): a
    # new instance made first in discovery and a new struct (one more
    # template Adt) keep every other instance's id, so a build
    # regenerates only the edited fn, not every body; a restarted
    # daemon reads the places back with the def order and builds no
    # unit again. Every answer equals a cold build.
    work = tempfile.mkdtemp(prefix="lust-driver-stable-ids-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    fns = ["#[derive(Debug, Clone)]\nstruct W<T> {\n    a: T,\n    b: u8,\n}\n",
           "fn f0() -> usize {\n    let w = W { a: 5u8, b: 6 };\n    format!(\"{:?}\", w).len()\n}\n"]
    fns += [f"fn f{i}() -> usize {{\n    let w = W {{ a: {i}u8, b: 9 }};\n    format!(\"{{:?}}\", w).len()\n}}\n" for i in range(1, 49)]
    fns += [f"fn f{i}() -> usize {{\n    let w = W {{ a: {i}u16, b: 3 }};\n    format!(\"{{:?}}\", w).len()\n}}\n" for i in range(49, 97)]
    main = ("fn main() {\n    let mut t = 0;\n" + "".join(f"    t += f{i}();\n" for i in range(97))
            + "    println!(\"{}\", t);\n    println!(\"{:?}\", W { a: 1u8, b: 2 });\n}\n")
    base = "\n".join(fns) + main
    new_inst = base.replace("fn f0() -> usize {\n", "fn f0() -> usize {\n    let z = W { a: 70000u32, b: 1 };\n"
                            "    let _ = format!(\"{:?}\", z);\n", 1)
    new_struct = "struct Fresh { v: u8 }\n\n" + new_inst.replace("    println!(\"{}\", t);\n", "    println!(\"{} {}\", t, Fresh { v: 4 }.v);\n")

    def cold(text):
        cw = tempfile.mkdtemp(prefix="lust-driver-stable-ids-cold-")
        csrc = os.path.join(cw, "x.rs")
        open(csrc, "w").write(text)
        env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(cw, "c"))
        lust_run([LUST, "build", csrc, "-o", os.path.join(cw, "x")], cwd=cw, env=env, capture_output=True)
        r = run_prog(os.path.join(cw, "x"))
        shutil.rmtree(cw, ignore_errors=True)
        return r.stdout

    d.start()
    for name, text, most in (("base", base, None), ("new instance first in discovery", new_inst, 2), ("new struct", new_struct, 2)):
        open(src, "w").write(text)
        st, prog = d.build(src, out)
        check(f"stable instance ids, {name}: warm equals cold", prog.stdout == cold(text), prog.stdout[-80:])
        if most is not None:
            check(f"stable instance ids, {name}: only the edited fn regenerates", st.get("rechecked", 99) <= most, str(st))
    d.kill()
    d.start()
    st, prog = d.build(src, out)
    check("stable instance ids, restart: warm equals cold", prog.stdout == cold(new_struct), prog.stdout[-80:])
    check("stable instance ids, restart: every unit from disk", st.get("units_compiled") == 0, str(st))
    d.kill()


def case_user_trait_impl_edits():
    # 20. Impls of user traits are outside the trait-system key: selection
    # records an `ImplSet` read of the bucket it searched. An added unused
    # impl rechecks no template and discovers almost nothing again; a used
    # one, a removed one, and an impl that starts to override a default
    # method answer as cold. A lang trait impl edit leaves the full path
    # too: the bodies whose types name the impl's Adt record `LangImpls`
    # reads, and an impl for an unused type rechecks almost nothing.
    work = tempfile.mkdtemp(prefix="lust-driver-user-impls-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    head = ("trait Tr {\n    fn go(&self) -> i64;\n    fn dflt(&self) -> i64 { 100 }\n}\n"
            "struct A;\nstruct B;\nimpl Tr for A {\n    fn go(&self) -> i64 { 1 }\n}\n"
            "fn generic<T: Tr>(t: &T) -> i64 { t.go() + t.dflt() }\n"
            "fn wrap<T: Copy>(t: T) -> (T, T) { (t, t) }\n")
    plain = "".join(f"fn p{i}(x: i64) -> i64 {{ wrap(x + {i}).0 }}\n" for i in range(30))
    main = "fn main() {\n    println!(\"{} {}\", generic(&A), " + " + ".join(f"p{i}(1)" for i in range(30)) + ");\n    //MAIN\n}\n"
    base = head + plain + main
    impl_b = "impl Tr for B {\n    fn go(&self) -> i64 { 2 }\n}\n"
    steps = [
        ("base", base, None),
        ("unused user trait impl", base + impl_b, "few"),
        ("the new impl in use", (base + impl_b).replace("    //MAIN\n", "    println!(\"{}\", generic(&B));\n"), None),
        ("default method overridden", (base + impl_b).replace("fn go(&self) -> i64 { 2 }\n}", "fn go(&self) -> i64 { 2 }\n    fn dflt(&self) -> i64 { 7 }\n}").replace("    //MAIN\n", "    println!(\"{}\", generic(&B));\n"), None),
        ("used impl removed", base.replace("impl Tr for A {\n    fn go(&self) -> i64 { 1 }\n}\n", ""), None),
        ("back", base, None),
        ("lang trait impl", base + "impl Clone for B {\n    fn clone(&self) -> B { B }\n}\n", "few"),
    ]

    def cold(text):
        cw = tempfile.mkdtemp(prefix="lust-driver-user-impls-cold-")
        csrc = os.path.join(cw, "x.rs")
        open(csrc, "w").write(text)
        env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(cw, "c"))
        c = lust_run([LUST, "check", csrc], cwd=cw, env=env, capture_output=True)
        b = lust_run([LUST, "build", csrc, "-o", os.path.join(cw, "x")], cwd=cw, env=env, capture_output=True)
        r = run_prog(os.path.join(cw, "x")) if b.returncode == 0 else None
        shutil.rmtree(cw, ignore_errors=True)
        return (c.returncode, r.stdout if r else None)

    d.start()
    for name, text, expect in steps:
        open(src, "w").write(text)
        c = d.run(["check", src], expect=None)
        st = parse_stats(c.stderr)
        b = d.run(["build", src, "-o", out], expect=None)
        r = run_prog(out) if b.returncode == 0 else None
        check(f"user trait impls, {name}: warm equals cold", (c.returncode, r.stdout if r else None) == cold(text), c.stderr[-200:])
        if expect == "few":
            check(f"user trait impls, {name}: no template check, little discovery",
                  st.get("templates") == 0 and st.get("discovered", 99) <= 2 and st.get("rechecked", 99) <= 2, str(st))
        elif expect == "full":
            check(f"user trait impls, {name}: full generic path", st.get("templates", 0) > 0, str(st))
    d.kill()


def case_inherent_impl_edits():
    # 21. Plain inherent impls (no associated const, no `dyn` Self) are
    # outside the trait-system key: method probes and the direct inherent
    # table lookups (`S::f` paths, inherent-first checks) record
    # `MethodSet` reads. An added inherent method runs no template check
    # and almost no discovery; one that shadows a trait method, a removed
    # assoc fn under `S::f` and `Self::f` callers, and back all equal
    # cold. An inherent impl with an associated const still takes the
    # full generic path.
    work = tempfile.mkdtemp(prefix="lust-driver-inherent-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    head = ("trait Tr {\n    fn go(&self) -> i64;\n}\n"
            "struct A { v: i64 }\nimpl Tr for A {\n    fn go(&self) -> i64 { 1 }\n}\n"
            "impl A {\n    fn new() -> A { A { v: 3 } }\n    fn make() -> A { Self::new() }\n}\n"
            "fn generic<T: Tr>(t: &T) -> i64 { t.go() }\n"
            "fn wrap<T: Copy>(t: T) -> (T, T) { (t, t) }\n")
    plain = "".join(f"fn p{i}(x: i64) -> i64 {{ wrap(x + {i}).0 }}\n" for i in range(30))
    main = ("fn main() {\n    let a = A::make();\n    println!(\"{} {} {}\", a.go(), generic(&a), "
            + " + ".join(f"p{i}(1)" for i in range(30)) + ");\n}\n")
    base = head + plain + main
    steps = [
        ("base", base, None),
        ("unused inherent method", base + "impl A {\n    fn twice(&self) -> i64 { self.v * 2 }\n}\n", "few"),
        ("inherent method shadows the trait method", base + "impl A {\n    fn go(&self) -> i64 { 50 }\n}\n", None),
        ("assoc fn removed under Self:: and path callers", base.replace("    fn new() -> A { A { v: 3 } }\n", ""), None),
        ("back", base, None),
        ("inherent assoc const", base + "impl A {\n    const K: i64 = 4;\n}\n", "full"),
    ]

    def cold(text):
        cw = tempfile.mkdtemp(prefix="lust-driver-inherent-cold-")
        csrc = os.path.join(cw, "x.rs")
        open(csrc, "w").write(text)
        env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(cw, "c"))
        c = lust_run([LUST, "check", csrc], cwd=cw, env=env, capture_output=True)
        b = lust_run([LUST, "build", csrc, "-o", os.path.join(cw, "x")], cwd=cw, env=env, capture_output=True)
        r = run_prog(os.path.join(cw, "x")) if b.returncode == 0 else None
        shutil.rmtree(cw, ignore_errors=True)
        return (c.returncode, r.stdout if r else None)

    d.start()
    for name, text, expect in steps:
        open(src, "w").write(text)
        c = d.run(["check", src], expect=None)
        st = parse_stats(c.stderr)
        b = d.run(["build", src, "-o", out], expect=None)
        r = run_prog(out) if b.returncode == 0 else None
        check(f"inherent impls, {name}: warm equals cold", (c.returncode, r.stdout if r else None) == cold(text), c.stderr[-200:])
        if expect == "few":
            check(f"inherent impls, {name}: no template check, little discovery",
                  st.get("templates") == 0 and st.get("discovered", 99) <= 2 and st.get("rechecked", 99) <= 2, str(st))
        elif expect == "full":
            check(f"inherent impls, {name}: full generic path", st.get("templates", 0) > 0, str(st))
    d.kill()


def case_dep_std_edits():
    # 22. The std splice sits in the root's file 0, but a dependency
    # resolves first; its std module paths must still reach the
    # hidden std modules, in the first daemon request and after an
    # edit in the dependency changes which std modules the plan holds.
    work = tempfile.mkdtemp(prefix="lust-driver-depstd-")
    app = os.path.join(work, "app")
    os.makedirs(os.path.join(app, "src"))
    os.makedirs(os.path.join(work, "util", "src"))
    open(os.path.join(app, "Cargo.toml"), "w").write(
        '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2024"\n\n'
        '[dependencies]\nutil = { path = "../util" }\n\n[workspace]\n')
    open(os.path.join(work, "util", "Cargo.toml"), "w").write(
        '[package]\nname = "util"\nversion = "0.1.0"\nedition = "2024"\n')
    main_rs = os.path.join(app, "src", "main.rs")
    lib_rs = os.path.join(work, "util", "src", "lib.rs")
    open(main_rs, "w").write('fn main() {\n    println!("{}", util::show());\n}\n')
    extra = ("--manifest-path", os.path.join(app, "Cargo.toml"), "--bin", "app")
    versions = [
        ("no std module", 'pub fn show() -> String {\n    format!("{}", 3)\n}\n', "3"),
        ("body uses std::time and VecDeque",
         "use std::collections::VecDeque;\nuse std::time::Duration;\n\n"
         "pub fn show() -> String {\n    let mut q = VecDeque::new();\n"
         "    q.push_back(Duration::from_millis(5));\n    format!(\"{:?}\", q)\n}\n", "[5ms]"),
        ("signature types and statics",
         "use std::marker::PhantomData;\nuse std::sync::LazyLock;\nuse std::time::Instant;\n\n"
         "static N: LazyLock<u32> = LazyLock::new(|| 7);\n\n"
         "pub struct Timer<T> {\n    pub at: Instant,\n    pub tag: PhantomData<T>,\n}\n\n"
         "pub fn timer() -> Timer<u8> {\n    Timer { at: Instant::now(), tag: PhantomData }\n}\n\n"
         "pub fn show() -> String {\n    let t = timer();\n"
         "    format!(\"{} {}\", *N, t.at.elapsed().as_secs() < 60)\n}\n", "7 true"),
        ("ControlFlow and Once",
         "use std::ops::ControlFlow;\nuse std::sync::Once;\n\nstatic INIT: Once = Once::new();\n\n"
         "fn first_even(xs: &[u32]) -> ControlFlow<u32> {\n    for &x in xs {\n"
         "        if x % 2 == 0 {\n            ControlFlow::Break(x)?;\n        }\n    }\n"
         "    ControlFlow::Continue(())\n}\n\n"
         "pub fn show() -> String {\n    INIT.call_once(|| {});\n"
         "    format!(\"{:?} {}\", first_even(&[3, 8, 5]), INIT.is_completed())\n}\n", "Break(8) true"),
        ("std modules removed again", 'pub fn show() -> String {\n    format!("{}", 4)\n}\n', "4"),
    ]
    d = Daemon(work)
    d.start()
    out = os.path.join(work, "x")
    cold_out = os.path.join(work, "c")
    cold_env = dict(d.env, LUST_NO_DAEMON="1")
    for i, (what, text, want) in enumerate(versions):
        open(lib_rs, "w").write(text)
        # Strictly rising mtimes: the daemon must see each edit.
        t = time.time() + 2 * (i + 1)
        os.utime(lib_rs, (t, t))
        chk = d.run(["check", main_rs, *extra], expect=None)
        check(f"dep std v{i + 1} ({what}) warm check", chk.returncode == 0, chk.stderr[-400:])
        _, prog = d.build(main_rs, out, extra)
        cold = lust_run([LUST, "build", main_rs, "-o", cold_out, *extra], cwd=work, env=cold_env,
                              capture_output=True, text=True)
        cprog = run_prog(cold_out) if cold.returncode == 0 else None
        check(f"dep std v{i + 1} ({what}) output", prog.stdout.strip() == want, prog.stdout + prog.stderr[-300:])
        check(f"dep std v{i + 1} ({what}) warm == cold",
              cprog is not None and cprog.stdout == prog.stdout, cold.stderr[-300:])
    d.kill()


def case_trait_default_nested():
    # 23. Fns nested in trait default method bodies. The token scan skipped
    # trait bodies, so the driver counted fewer AST fns than collect made
    # and registered the nested ones again as instance slots: "internal:
    # DefPathHash collision" (exit 3), cold and warm. Their def path also
    # fell back to the body item index, which moves when an earlier body
    # gains an item. Now they parent on the trait method: an item added to
    # f0 rechecks f0 and its own nested fns only.
    work = tempfile.mkdtemp(prefix="lust-driver-trait-nested-")
    d = Daemon(work)
    d.start()
    src = os.path.join(work, "x.rs")
    out, cold_out = os.path.join(work, "x"), os.path.join(work, "xc")

    def text(extra, factor="10"):
        return ("fn f0() -> i64 {\n" + extra + "    fn z() -> i64 { 7 }\n    9 + z()\n}\n"
                "trait Tr {\n    fn t(&self) -> i64;\n    fn d(&self) -> i64 {\n"
                "        fn h(x: &i64) -> bool { *x > 1 }\n        fn m(x: i64) -> i64 { x * " + factor + " }\n"
                "        self.t() + [1i64, 2, 3].iter().copied().filter(h).map(m).sum::<i64>()\n    }\n}\n"
                "trait Tr2 {\n    fn d(&self) -> i64 {\n        fn h(x: &i64) -> bool { *x > 2 }\n"
                "        [1i64, 2, 3].iter().copied().filter(h).count() as i64\n    }\n}\n"
                "impl Tr for i64 {\n    fn t(&self) -> i64 {\n        *self\n    }\n}\nimpl Tr2 for i64 {}\n"
                'fn main() {\n    println!("{} {}", f0() + Tr::d(&5i64), Tr2::d(&0i64));\n}\n')

    steps = (("base", text(""), "71 1"),
             ("item added to another body", text("    fn y() -> i64 { 3 }\n"), "71 1"),
             ("nested fn body edit in a default body", text("    fn y() -> i64 { 3 }\n", "100"), "521 1"),
             ("back", text(""), "71 1"))
    for i, (step, body, want) in enumerate(steps):
        open(src, "w").write(body)
        t = time.time() + 2 * (i + 1)
        os.utime(src, (t, t))
        c = d.run(["check", src], expect=None)
        stats = parse_stats(c.stderr)
        check(f"trait default nested, {step}: check", c.returncode == 0, c.stderr[-400:])
        if step == "item added to another body":
            # f0, its nested z (its enclosing body changed), and the new y.
            check("trait default nested: other bodies keep their slots", stats.get("rechecked") == 3, str(stats))
        b = d.run(["build", src, "-o", out], expect=None)
        prog = run_prog(out) if b.returncode == 0 else None
        cold_env = dict(d.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(work, f"cold{i}"))
        cold = lust_run([LUST, "build", src, "-o", cold_out], cwd=work, env=cold_env,
                        capture_output=True, text=True)
        cprog = run_prog(cold_out) if cold.returncode == 0 else None
        got = (b.returncode, prog.stdout.strip() if prog else b.stderr[-300:])
        check(f"trait default nested, {step}: warm output", got == (0, want), str(got))
        check(f"trait default nested, {step}: warm == cold",
              cprog is not None and prog is not None and cprog.stdout == prog.stdout, cold.stderr[-300:])
    d.kill()


def case_deep_type_edits():
    # 24. Fn and Adt signatures fold the types they name by identity: an
    # edit inside a type that a central struct holds dirties the bodies
    # that open it (field reads), hold it by value (`Layout` reads), or
    # ask an auto trait or drop glue of it, not every caller of the
    # struct's methods. Thirty `&Hub` callers of `Hub::get` stay clean
    # when `Leaf` gains a field; a new field with drop glue, and one that
    # makes `Hub` not `Sync`, answer as cold.
    work = tempfile.mkdtemp(prefix="lust-driver-deep-types-")
    d = Daemon(work)
    src = os.path.join(work, "x.rs")
    out = os.path.join(work, "x")
    leaf = "#[derive(Default)]\nstruct Leaf {\n    x: i64,\n}\n"
    head = ("#[derive(Default)]\nstruct Noisy;\nimpl Drop for Noisy {\n    fn drop(&mut self) { println!(\"drop\"); }\n}\n"
            "struct Hub {\n    leaf: Leaf,\n    n: i64,\n}\n"
            "impl Hub {\n    fn new() -> Hub { Hub { leaf: Leaf { x: 1, ..Default::default() }, n: 2 } }\n"
            "    fn get(&self) -> i64 { self.leaf.x + self.n }\n}\n"
            "fn need_sync<T: Sync>(_: &T) {}\n")
    callers = "".join(f"fn q{i}(h: &Hub) -> i64 {{ h.get() + {i} }}\n" for i in range(30))
    main = "fn main() {\n    let h = Hub::new();\n    need_sync(&h);\n    println!(\"{}\", " + " + ".join(f"q{i}(&h)" for i in range(30)) + ");\n}\n"
    base = leaf + head + callers + main
    steps = [
        ("base", base, None),
        ("a field added to Leaf", base.replace("    x: i64,\n}", "    x: i64,\n    y: u8,\n}", 1), "few"),
        ("a field with drop glue in Leaf", base.replace("    x: i64,\n}", "    x: i64,\n    z: Noisy,\n}", 1), "few"),
        ("a field that is not Sync in Leaf", base.replace("    x: i64,\n}", "    x: i64,\n    c: std::cell::Cell<i64>,\n}", 1), None),
        ("back", base, "few"),
    ]

    def cold(text):
        cw = tempfile.mkdtemp(prefix="lust-driver-deep-types-cold-")
        csrc = os.path.join(cw, "x.rs")
        open(csrc, "w").write(text)
        env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=os.path.join(cw, "c"))
        c = lust_run([LUST, "check", csrc], cwd=cw, env=env, capture_output=True)
        b = lust_run([LUST, "build", csrc, "-o", os.path.join(cw, "x")], cwd=cw, env=env, capture_output=True)
        r = run_prog(os.path.join(cw, "x")) if b.returncode == 0 else None
        shutil.rmtree(cw, ignore_errors=True)
        return (c.returncode, r.stdout if r else None)

    d.start()
    for name, text, expect in steps:
        open(src, "w").write(text)
        c = d.run(["check", src], expect=None)
        st = parse_stats(c.stderr)
        b = d.run(["build", src, "-o", out], expect=None)
        r = run_prog(out) if b.returncode == 0 else None
        check(f"deep type edits, {name}: warm equals cold", (c.returncode, r.stdout if r else None) == cold(text), c.stderr[-200:])
        if expect == "few":
            check(f"deep type edits, {name}: the &Hub callers stay clean", st.get("rechecked", 99) <= 8, str(st))
    d.kill()


def main():
    if not os.path.exists(LUST):
        sys.exit(f"{LUST} not built; run cargo build --release first")
    cases = (case_slots_and_reuse, case_reads_reuse, case_graph_refresh_and_std,
             case_multi_incremental, case_compaction_reemit, case_restart_add_file,
             case_empty_units_restart, case_positions, case_generic_reuse,
             case_moved_instance_ids, case_new_empty_unit, case_nested_items,
             case_moved_and_revived_slots, case_instance_calls_moved_slot,
             case_nested_identity_and_shadowing, case_stable_instance_ids,
             case_user_trait_impl_edits, case_inherent_impl_edits, case_dep_std_edits,
             case_trait_default_nested, case_deep_type_edits)
    # One case that raises (a failed assert, a timeout, an OS error)
    # fails that case only: the rest still run, and the summary line
    # below is always the last line, as the gate's need_ok requires.
    # Each case uses its own temp dir and daemon socket, so a daemon
    # left by a failed case cannot serve or block the next one.
    for case in cases:
        try:
            case()
        except Exception:
            print(f"FAIL {case.__name__}: raised\n{traceback.format_exc()}", flush=True)
            failures.append(case.__name__)
    if failures:
        print("failed: " + ", ".join(failures))
    print(f"test-driver: {passed} passed, {len(failures)} failed")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
