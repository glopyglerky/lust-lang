#!/usr/bin/env python3
"""K4 audit: classify FxHashMap/FxHashSet iteration sites in ts-rust.

Static, name-based heuristic (no type inference):
1. Collect type aliases that name an Fx map/set, and field/local/param names
   whose declared type or initializer names an Fx map/set (including
   HashMap<.., FxBuildHasher> spellings).
2. Find iteration uses of those names in the same file:
   .iter()/iter_mut()/keys()/values()/values_mut()/into_iter()/drain(..)/
   retain(..)/extract_if(..), `for x in name`, `for x in &name`, `&name` in
   for, `name.clone().into_iter()` etc, plus `expr.field.iter()` for fields
   collected globally.
3. Classify the statement that contains the iteration:
   sorted     - chain calls sort* before the statement ends
   reduced    - chain ends in any/all/count/sum/product/min/max/is_empty
   selection  - chain ends in find/position/first/next/min_by/max_by (pick one)
   other      - everything else (for bodies, collect to Vec, emit, extend...)

Limits are stated in AUDIT.md. This script only reads ts-rust.
Usage: audit.py <ts-rust-root> [out.md]
"""
import re, sys, os
from collections import Counter, defaultdict

ROOT = sys.argv[1]
FX = r'(?:FxHashMap|FxHashSet|HashMap\s*<[^;{}]*?FxBuildHasher\s*>|HashSet\s*<[^;{}]*?FxBuildHasher\s*>)'

files = []
for dp, dn, fn in os.walk(os.path.join(ROOT, 'crates')):
    if 'target' in dp:
        continue
    for f in fn:
        if f.endswith('.rs'):
            files.append(os.path.join(dp, f))
files.sort()

texts = {}
for p in files:
    with open(p, errors='replace') as fh:
        texts[p] = fh.read()

def strip_comments(s):
    s = re.sub(r'//[^\n]*', '', s)
    s = re.sub(r'/\*.*?\*/', '', s, flags=re.S)
    return s

clean = {p: strip_comments(t) for p, t in texts.items()}

# Pass 1: type aliases to Fx maps, and global Fx field names.
aliases = set()
for p, t in clean.items():
    for m in re.finditer(r'\btype\s+(\w+)(?:<[^=]*?>)?\s*=\s*[^;]*?' + FX, t):
        aliases.add(m.group(1))
# second order aliases (type A = B<..> where B alias)
changed = True
while changed:
    changed = False
    for p, t in clean.items():
        for m in re.finditer(r'\btype\s+(\w+)(?:<[^=]*?>)?\s*=\s*(\w+)', t):
            if m.group(2) in aliases and m.group(1) not in aliases:
                aliases.add(m.group(1)); changed = True

ALIASFX = FX if not aliases else r'(?:' + FX[4:-1] + r'|' + '|'.join(sorted(aliases)) + r')'

def is_test(p, t):
    return ('/tests/' in p or p.endswith('_test.rs') or '/test' in p)

field_names = Counter()  # field name -> count of Fx-typed declarations
decls_by_file = {}
total_fx_type_mentions = 0
for p, t in clean.items():
    names = set()
    # named declarations:  name: Type  (fields, params, let with type)
    for m in re.finditer(r'\b(\w+)\s*:\s*(?:&(?:\w+\s+)?(?:mut\s+)?)?(?:[\w:]+::)?' + ALIASFX + r'\b', t):
        names.add(m.group(1)); field_names[m.group(1)] += 1
    # let name = FxHashMap::... / ::default() / collect::<Fx..>()
    for m in re.finditer(r'\blet\s+(?:mut\s+)?(\w+)\b[^=;]*=\s*(?:[\w:]+::)?(?:FxHashMap|FxHashSet)\s*(?:::|<)', t):
        names.add(m.group(1))
    for m in re.finditer(r'\blet\s+(?:mut\s+)?(\w+)\b[^;]*?\.collect\s*::\s*<\s*(?:[\w:]+::)?' + ALIASFX, t):
        names.add(m.group(1))
    for m in re.finditer(r'\blet\s+(?:mut\s+)?(\w+)\s*:\s*[^=;]*?' + ALIASFX, t):
        names.add(m.group(1))
    decls_by_file[p] = names
    total_fx_type_mentions += len(re.findall(r'\b(?:FxHashMap|FxHashSet)\b', t))

# Unambiguous global field names: every `name: TypeHead` declaration head in
# the tree must be Fx-ish (or constructor noise). Names like `members` or
# `parameters` are usually AST Vec fields and are excluded here.
NOISE_HEADS = {'self','new','default','with_hasher','with_capacity','with_capacity_and_hasher',
               'clone','None','Some','vec','seed','a','s','impl','change','parser','string_map'}
all_heads = {}
for p, t in clean.items():
    for m in re.finditer(r'\b(\w+)\s*:\s*(?:&[\w\s]*?)?(?:mut\s+)?([A-Za-z_][\w:]*)', t):
        all_heads.setdefault(m.group(1), Counter())[m.group(2).split('::')[-1]] += 1
FX_HEADS = {'FxHashMap','FxHashSet','HashMap','HashSet'} | aliases
unambig_fields = set()
for name in field_names:
    heads = {h for h in all_heads.get(name, {}) if h not in NOISE_HEADS}
    if heads and heads <= FX_HEADS:
        # HashMap head alone could be std SipHash; require an Fx head somewhere
        if heads & ({'FxHashMap','FxHashSet'} | aliases):
            unambig_fields.add(name)

ITER_METHODS = r'(?:iter|iter_mut|keys|values|values_mut|into_iter|drain|retain|extract_if)'

def statement_after(t, pos):
    # crude: up to 600 chars, stop at ';' at paren depth 0 or closing block of for-body start '{'
    s = t[pos:pos+900]
    return s

def classify(stmt, is_for):
    head = stmt[:700]
    if re.search(r'\.(?:sort|sort_by|sort_unstable|sort_unstable_by|sort_by_key|sort_by_cached_key|sorted)\b', head):
        return 'sorted'
    if re.search(r'\.(?:any|all|count|sum|product)\s*\(', head):
        return 'reduced'
    if re.search(r'\.(?:find|position|find_map|first|last|next|min_by|max_by|min_by_key|max_by_key)\s*\(', head):
        return 'selection'
    if re.search(r'\.(?:min|max)\s*\(', head):
        return 'reduced'
    return 'other'

sites = []  # (file, line, var, method, class, prod?)
vars_with_iter = defaultdict(set)
vars_declared = 0
for p, t in clean.items():
    names = decls_by_file[p]
    vars_declared += len(names)
    if not names and not field_names:
        continue
    lines = t.split('\n')
    for i, line in enumerate(lines):
        # direct named uses
        for name in names:
            for m in re.finditer(r'\b' + re.escape(name) + r'\s*\.\s*(' + ITER_METHODS + r')\s*\(', line):
                stmt = '\n'.join(lines[i:i+14])
                sites.append((p, i+1, name, m.group(1), classify(stmt, False)))
                vars_with_iter[p].add(name)
            for m in re.finditer(r'\bfor\s+[^{]*?\bin\s+(?:&\s*(?:mut\s+)?|&)?(?:\w+\.)*' + re.escape(name) + r'\b(?!\s*\.)', line):
                blk = '\n'.join(lines[i:i+20])
                sites.append((p, i+1, name, 'for-in', classify(blk, True)))
                vars_with_iter[p].add(name)
        # field uses: .field.iter() where field is a known Fx field name (any file)
        for m in re.finditer(r'\.(\w+)\s*\.\s*(' + ITER_METHODS + r')\s*\(', line):
            fname = m.group(1)
            if fname in unambig_fields and fname not in names:
                # avoid double count when receiver is a named local handled above
                before = line[:m.start()]
                if any(re.search(r'\b' + re.escape(n) + r'\s*$', before) for n in names):
                    continue
                stmt = '\n'.join(lines[i:i+14])
                sites.append((p, i+1, '.'+fname, m.group(2), classify(stmt, False)))
        for m in re.finditer(r'\bfor\s+[^{]*?\bin\s+(?:&\s*)?[\w.]+\.(\w+)\b(?!\s*\.)', line):
            fname = m.group(1)
            if fname in unambig_fields and fname not in names:
                blk = '\n'.join(lines[i:i+20])
                sites.append((p, i+1, '.'+fname, 'for-in', classify(blk, True)))

# dedup exact
sites = sorted(set(sites))
rel = lambda p: os.path.relpath(p, ROOT)

prod_sites = [s for s in sites if not is_test(s[0], texts[s[0]])]
test_sites = [s for s in sites if is_test(s[0], texts[s[0]])]

print(f"files scanned: {len(files)}")
print(f"FxHashMap/FxHashSet type mentions: {total_fx_type_mentions}")
print(f"type aliases to Fx maps: {sorted(aliases)}")
print(f"named Fx vars/fields/params found (per file, summed): {vars_declared}")
files_with_fx = [p for p in files if re.search(r'\bFxHash(?:Map|Set)\b', clean[p])]
print(f"files mentioning FxHashMap/FxHashSet: {len(files_with_fx)}")
print(f"unambiguous Fx field names (tier B): {sorted(unambig_fields)}")
tierb = [s for s in prod_sites if s[2].startswith('.')]
print(f"tier A (same-file name) sites: {len(prod_sites)-len(tierb)}, tier B (unambiguous field) sites: {len(tierb)}")
print(f"iteration sites (prod): {len(prod_sites)}  (test paths: {len(test_sites)})")
by_class = Counter(s[4] for s in prod_sites)
for k, v in by_class.most_common():
    print(f"  {k}: {v}")
by_method = Counter(s[3] for s in prod_sites)
for k, v in by_method.most_common():
    print(f"  method {k}: {v}")

# vars with no iteration found in their file
lookup_only = 0
for p, names in decls_by_file.items():
    lookup_only += len([n for n in names if n not in vars_with_iter[p]])
print(f"named Fx vars with no in-file iteration site (lookup-only candidates): {lookup_only}")

# IndexMap + std HashMap + BTree for context
for pat, label in [(r'\bIndexMap\b','IndexMap'), (r'\bIndexSet\b','IndexSet'),
                   (r'\bstd::collections::HashMap\b|collections::HashMap\b','std HashMap path'),
                   (r'\bBTreeMap\b','BTreeMap'), (r'\bBTreeSet\b','BTreeSet')]:
    n = sum(len(re.findall(pat, clean[p])) for p in files)
    nf = sum(1 for p in files if re.search(pat, clean[p]))
    print(f"{label}: {n} mentions in {nf} files")

# output-area risk: files under emitter/checker/lsp/etc with 'other' sites
RISKY_DIRS = ('emitter', 'printer', 'checker', 'lsp', 'lsproto', 'diagnostic', 'execute', 'compiler', 'transform', 'binder', 'parser', 'scanner', 'api', 'json', 'sourcemap', 'output')
print("\n'other'/'selection' sites in output-area paths (first 120):")
risky = [s for s in prod_sites if s[4] in ('other','selection') and any(d in rel(s[0]).lower() for d in RISKY_DIRS)]
for s in risky[:120]:
    print(f"  {rel(s[0])}:{s[1]} {s[2]}.{s[3]} [{s[4]}]")
print(f"risky-area other/selection sites: {len(risky)} of {sum(1 for s in prod_sites if s[4] in ('other','selection'))} other/selection total")

if len(sys.argv) > 2:
    with open(sys.argv[2], 'w') as fh:
        fh.write("# all prod iteration sites\n")
        for s in prod_sites:
            fh.write(f"{rel(s[0])}:{s[1]}\t{s[2]}\t{s[3]}\t{s[4]}\n")
    print(f"\nwrote {sys.argv[2]}")
