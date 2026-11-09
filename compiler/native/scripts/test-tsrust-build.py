#!/usr/bin/env python3
# Unit tests for the text transforms of scripts/tsrust_build.py: cfg(test) on, test attribute
# markers, `pub(crate)` insertion, dev-dependencies as dependencies.
# Usage: python3 scripts/test-tsrust-build.py   (exit 0 = pass)
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "bench/slices"))
import tsrust_build as tb  # noqa: E402

failures = []


def eq(name, got, want):
    if got != want:
        failures.append(f"{name}:\n  got  {got!r}\n  want {want!r}")


eq("cfg test", tb.cfg_on("#[cfg(test)]\nmod t;"), "#[cfg(all())]\nmod t;")
eq("cfg not test", tb.cfg_on("#[cfg(not(test))] fn a() {}"), "#[cfg(not(all()))] fn a() {}")
eq("cfg any", tb.cfg_on('#[cfg(any(test, feature = "test"))]'), '#[cfg(any(all(), feature = "test"))]')
eq("cfg_attr", tb.cfg_on("#[cfg_attr(test, derive(Debug))]"), "#[cfg_attr(all(), derive(Debug))]")
eq("cfg macro", tb.cfg_on("if cfg!(test) { tests() }"), "if cfg!(all()) { tests() }")
eq("not a cfg", tb.cfg_on("fn test() { test(); }"), "fn test() { test(); }")
eq("marks", tb.mark_attrs("    #[test]\n    #[should_panic(expected = \"x\")]\n    #[ignore]\n    fn a() {}"),
   '    #[doc = "lust-test"]\n    #[doc = "lust-should-panic"]\n    #[doc = "lust-ignore"]\n    fn a() {}')
eq("mark only at line start", tb.mark_attrs('let s = "#[test]";'), 'let s = "#[test]";')

with tempfile.TemporaryDirectory() as d:
    f = Path(d) / "a.rs"
    f.write_text("mod tests {\n    fn a() {}\n    const fn b() {}\n    pub fn c() {}\n#[cfg(all())] mod inner {}\n}\n")
    edits = {}
    for line, word, name in [(1, "mod", "tests"), (2, "fn", "a"), (3, "fn", "b"), (4, "fn", "c"), (5, "mod", "inner")]:
        tb.add_text(edits, str(f), line, word, name, "pub(crate) ")
    tb.apply_edits(edits)
    eq("vis edits", f.read_text(),
       "pub(crate) mod tests {\n    pub(crate) fn a() {}\n    pub(crate) const fn b() {}\n    pub fn c() {}\n#[cfg(all())] pub(crate) mod inner {}\n}\n")
    m = Path(d) / "Cargo.toml"
    m.write_text('[package]\nname = "p"\n\n[dependencies]\nx = "1"\n\n[dev-dependencies]\nx = "1"\ny = "2"\n')
    tb.manifests_with_dev_deps(Path(d))
    eq("dev deps", m.read_text(), '[package]\nname = "p"\n\n[dependencies]\ny = "2"\nx = "1"\n\n')

# Deep census: a private leaf in a use group goes alone and reports its rename (the census
# removed the whole group for "`new_program` is private", and `IncrementalProgram` went with it).
with tempfile.TemporaryDirectory() as d:
    f = Path(d) / "a.rs"
    f.write_text("use a::program::{\n    Program as IncrementalProgram, build_info_program,\n    new_program as new_incremental_program,\n};\n")
    eq("private leaf", tb.prune.drop_use_leaf(str(f), (1, 4, 0, "use", "-"), "L0510", "`new_program` is private"), "new_incremental_program")
    eq("private leaf text", f.read_text(), "use a::program::{\n    Program as IncrementalProgram, build_info_program,\n    \n};\n")
    eq("other code", tb.prune.drop_use_leaf(str(f), (1, 3, 0, "use", "-"), "L0102", "`build_info_program` x"), False)

# The census helpers tsrust_build.py calls (tsrust_deep.py and prune.py change under other packages).
import inspect  # noqa: E402

for mod, name, nargs in [(tb.tsrust_deep, "errors", 4), (tb.tsrust_deep, "in_item", 2), (tb.tsrust_deep, "names_in", 1),
                         (tb.tsrust_deep, "remove_users", 5), (tb.prune, "prune_round", 6), (tb.prune, "located", 2)]:
    f = getattr(mod, name, None)
    eq(f"{mod.__name__}.{name}", f is not None and len(inspect.signature(f).parameters), nargs)

if failures:
    print("\n".join(failures))
    sys.exit(1)
print("test-tsrust-build: all passed")
