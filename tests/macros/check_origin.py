#!/usr/bin/env python3
"""Check that copied macro text still matches ts-rust exactly.

Usage: TS_RUST=/path/to/ts-rust python3 tests/macros/check_origin.py

For each pass fixture, reads the `// macro:` and `// source:`
headers, extracts the macro_rules! body for each named macro from
the fixture and from the ts-rust file the header points at, and
compares them with whitespace collapsed. A definition counts as a
match when it equals any definition of that name in the source file
(ts-rust defines some names twice).

Exit 0 when all match or when the ts-rust tree is not present
(prints SKIPPED). Exit 1 on a mismatch.
"""
import glob
import os
import re
import sys

TS = os.environ.get("TS_RUST", "/home/theo/.t3/projects/ts-rust")
ROOT = os.path.dirname(os.path.abspath(__file__))


def extract_defs(text, name):
    """All macro_rules! bodies for `name` in `text`, balanced."""
    out = []
    pairs = {"{": "}", "[": "]", "(": ")"}
    for m in re.finditer(r"macro_rules!\s*" + re.escape(name) + r"\b", text):
        i = m.start()
        while text[i] not in pairs:
            i += 1
        stack, j = [], i
        instr = esc = inchar = incomment = False
        while j < len(text):
            c = text[j]
            nxt = text[j + 1] if j + 1 < len(text) else ""
            if incomment:
                if c == "\n":
                    incomment = False
            elif instr:
                if esc:
                    esc = False
                elif c == "\\":
                    esc = True
                elif c == '"':
                    instr = False
            elif inchar:
                if esc:
                    esc = False
                elif c == "\\":
                    esc = True
                elif c == "'":
                    inchar = False
            else:
                if c == "/" and nxt == "/":
                    incomment = True
                elif c == '"':
                    instr = True
                elif c == "'" and nxt and (nxt == "\\" or (j + 2 < len(text) and text[j + 2] == "'")):
                    inchar = True
                elif c in pairs:
                    stack.append(pairs[c])
                elif stack and c == stack[-1]:
                    stack.pop()
                    if not stack:
                        out.append(text[m.start():j + 1])
                        break
            j += 1
    return out


def norm(s):
    return re.sub(r"\s+", " ", s).strip()


def main():
    if not os.path.isdir(TS):
        print(f"SKIPPED: ts-rust tree not found at {TS} (set TS_RUST)")
        return 0
    bad = 0
    checked = 0
    for path in sorted(glob.glob(os.path.join(ROOT, "pass", "**", "*.rs"), recursive=True)):
        text = open(path).read()
        names = re.findall(r"^// macro: (.*)", text, re.M)
        sources = re.findall(r"^// source: (\S+?):\d+", text, re.M)
        if not names:
            continue
        for name in [x.strip() for x in names[0].split(",")]:
            fixture_defs = [norm(d) for d in extract_defs(text, name)]
            ts_defs = []
            for rel in sources:
                full = os.path.join(TS, rel)
                if os.path.isfile(full):
                    ts_defs += [norm(d) for d in extract_defs(open(full).read(), name)]
            if not fixture_defs:
                print(f"MISMATCH {path}: macro {name} named in header but no definition in fixture")
                bad += 1
            elif not ts_defs:
                print(f"MISMATCH {path}: macro {name} not found in its // source: files")
                bad += 1
            elif not any(d in ts_defs for d in fixture_defs):
                print(f"MISMATCH {path}: macro {name} text differs from ts-rust")
                bad += 1
            else:
                checked += 1
    print(f"check-origin: {checked} exact, {bad} mismatched")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
