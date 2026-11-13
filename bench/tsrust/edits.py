#!/usr/bin/env python3
"""The paired benchmark edits for a COPY of ts-rust.

This is /tmp/lust-ts/baseline/apply_edits.py with the source root as an
argument instead of a hard-coded path. The same edit list drives the rustc
column today and the lust column when lust can build the crate: paired
edits, same anchors, same order.

Usage:
  edits.py <src-root> <e1|e2|e3|e4|e5|e5fix>   apply one edit (cumulative)
  edits.py <src-root> --check                   verify all start anchors, write nothing

Safety: refuses to touch the original ts-rust checkout. Pass a copy.
Each edit asserts its anchor text occurs exactly once. e5 is a type error;
e5fix reverts it. Apply in order e1, e2, e3, e4, e5, e5fix.
"""
import os
import sys

ORIGINAL = "/home/theo/.t3/projects/ts-rust"

BODY_OLD = """pub fn token_is_identifier_or_keyword(token: SyntaxKind) -> bool {
    (token as u16) >= (SyntaxKind::Identifier as u16)
}"""
BODY_NEW = """pub fn token_is_identifier_or_keyword(token: SyntaxKind) -> bool {
    let kind = token as u16;
    kind >= (SyntaxKind::Identifier as u16)
}"""
BODY_BAD = """pub fn token_is_identifier_or_keyword(token: SyntaxKind) -> bool {
    let kind = token as u16;
    kind >= "Identifier"
}"""

NEW_FN = BODY_NEW + """

// Baseline benchmark edit: a new helper next to the function it wraps.
pub fn baseline_probe_token_is_identifier(token: SyntaxKind) -> bool {
    token_is_identifier_or_keyword(token) && (token as u16) > 0
}"""

SIG_OLD = """pub fn is_syntactic_default(node: Node) -> bool {
    (is_export_assignment(node) && !node.is_export_equals())
        || has_syntactic_modifier(node, ModifierFlags::DEFAULT)
        || is_export_specifier(node)
        || is_namespace_export(node)
}"""
SIG_NEW = """pub fn is_syntactic_default(node: Node, include_specifiers: bool) -> bool {
    (is_export_assignment(node) && !node.is_export_equals())
        || has_syntactic_modifier(node, ModifierFlags::DEFAULT)
        || (include_specifiers && is_export_specifier(node))
        || is_namespace_export(node)
}"""
CALL_OLD = ".any(|&d| is_syntactic_default(d))"
CALL_NEW = ".any(|&d| is_syntactic_default(d, true))"

TYPE_OLD = "pub struct ExhaustiveState(pub u8);"
TYPE_NEW = "pub struct ExhaustiveState(pub u16);"


def paths(root):
    base = os.path.join(root, "crates/ts_goport/src/checker")
    return (os.path.join(base, "utilities_p1.rs"),
            os.path.join(base, "types.rs"),
            os.path.join(base, "checker_p16.rs"))


def patch(path, old, new):
    text = open(path).read()
    n = text.count(old)
    assert n == 1, f"anchor occurs {n}x in {path}: {old[:60]!r}"
    open(path, "w").write(text.replace(old, new))
    print(f"patched {path}")


def check_anchor(path, old, label):
    n = open(path).read().count(old)
    status = "ok" if n == 1 else f"FAIL (occurs {n}x)"
    print(f"{label}: {status} in {os.path.basename(path)}")
    return n == 1


def main():
    root, edit = sys.argv[1], sys.argv[2]
    root = os.path.realpath(root)
    assert root != os.path.realpath(ORIGINAL), f"refusing to edit the original ts-rust at {ORIGINAL}; pass a copy"
    u, t, c16 = paths(root)
    if edit == "--check":
        # Start-of-loop state: pristine sources. e5's anchor (BODY_NEW) only
        # exists after e1, so it is not checked here.
        ok = check_anchor(u, BODY_OLD, "e1")
        ok = check_anchor(u, SIG_OLD, "e3 signature") and ok
        ok = check_anchor(c16, CALL_OLD, "e3 call site") and ok
        ok = check_anchor(t, TYPE_OLD, "e4") and ok
        sys.exit(0 if ok else 1)
    if edit == "e1":
        patch(u, BODY_OLD, BODY_NEW)
    elif edit == "e2":
        patch(u, BODY_NEW, NEW_FN)
    elif edit == "e3":
        patch(u, SIG_OLD, SIG_NEW)
        patch(c16, CALL_OLD, CALL_NEW)
    elif edit == "e4":
        patch(t, TYPE_OLD, TYPE_NEW)
    elif edit == "e5":
        # e2 left BODY_NEW plus the new helper in the file; BODY_NEW still
        # occurs exactly once as the start of that text.
        patch(u, BODY_NEW, BODY_BAD)
    elif edit == "e5fix":
        patch(u, BODY_BAD, BODY_NEW)
    else:
        sys.exit(f"unknown edit {edit}")


if __name__ == "__main__":
    main()
