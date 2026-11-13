# Step 2: walks every std/core/alloc module with rustc. For a module M,
# `use ::M::*; use self::N as _;` for each pool name N: no error means
# M::N exists (E0658 on the line: unstable, dropped). Lowercase names
# whose glob works are modules (walked in turn), uppercase ones enums
# (their variants are probed with the uppercase pool).
import itertools, json, os, re, subprocess, sys, threading
from concurrent.futures import ThreadPoolExecutor
KW = set("as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn abstract become box do final macro override priv typeof unsized virtual yield try gen union _".split())
pool = sorted(set(l.strip() for f in sys.argv[1:] for l in open(f) if l.strip()) - KW)
upper = [n for n in pool if n[0].isupper()]
os.makedirs('w2', exist_ok=True)
_ids = itertools.count()
_lock = threading.Lock()

def next_id():
    with _lock:
        return next(_ids)

def head(mode):
    h = []
    if mode != 'std': h.append('#![no_std]')
    if mode == 'alloc': h.append('extern crate alloc;')
    return h

def rustc(path, text, n_probes):
    open(path, 'w').write(text)
    for attempt in range(3):
        r = subprocess.run(['rustc', '--edition', '2024', '--crate-type', 'lib', '--emit=metadata', '--error-format=json', '-o', path + '.rmeta', path], capture_output=True, text=True)
        # A probe set this big always has misses: no E0432 at all means
        # rustc died early (memory), not that every name exists.
        if n_probes < 50 or '"E0432"' in r.stderr:
            break
    else:
        sys.exit(f'probe {path} gave no E0432 three times')
    errs = {}
    for l in r.stderr.splitlines():
        try: d = json.loads(l)
        except Exception: continue
        if d.get('level') != 'error' or not d.get('spans'): continue
        code = (d.get('code') or {}).get('code') or 'other'
        for s in d['spans']:
            if s.get('is_primary'):
                errs.setdefault(s['line_start'], set()).add(code)
    return errs

def probe(mode, path, names):
    i = next_id()
    lines = head(mode) + ['#[no_implicit_prelude] mod t {', 'use ::%s::*;' % '::'.join(path)]
    base = len(lines)
    for n in names: lines.append('use self::%s as _;' % n)
    lines.append('}')
    errs = rustc('w2/p%d.rs' % i, '\n'.join(lines) + '\n', len(names))
    if base in errs:  # the glob itself failed (unstable or private)
        return None
    return [n for i, n in enumerate(names) if (base + 1 + i) not in errs]

def globbable(mode, path, names):
    i = next_id()
    lines = head(mode) + ['#[no_implicit_prelude] mod t {']
    base = len(lines)
    for k, n in enumerate(names): lines.append('mod g%d { use ::%s::%s::*; }' % (k, '::'.join(path), n))
    lines.append('}')
    errs = rustc('w2/g%d.rs' % i, '\n'.join(lines) + '\n', 0)
    return [n for i, n in enumerate(names) if (base + 1 + i) not in errs]

table = {}
frontier = [('std', ('std',)), ('core', ('core',)), ('alloc', ('alloc',))]
seen = set(frontier)
for r in frontier: table[r[1]] = 'm'
depth = 0
with ThreadPoolExecutor(12) as ex:
    while frontier and depth < 9:
        depth += 1
        found = list(ex.map(lambda mp: probe(mp[0], mp[1], pool), frontier))
        nxt, enums = [], []
        cands = []
        for (mode, path), names in zip(frontier, found):
            if names is None:
                table[path] = 'u'
                continue
            for n in names: table.setdefault(path + (n,), 'i')
            cands.append((mode, path, [n for n in names if n[0].islower() or n[0] == '_'], [n for n in names if n[0].isupper()]))
        globs = list(ex.map(lambda c: (globbable(c[0], c[1], c[2]), globbable(c[0], c[1], c[3])), cands))
        for (mode, path, _, _), (mods, ens) in zip(cands, globs):
            for n in mods:
                p = path + (n,)
                table[p] = 'm'
                if (mode, p) not in seen:
                    seen.add((mode, p)); nxt.append((mode, p))
            for n in ens:
                table[path + (n,)] = 'e'; enums.append((mode, path + (n,)))
        vs = list(ex.map(lambda mp: probe(mp[0], mp[1], upper), enums))
        for (mode, path), names in zip(enums, vs):
            for n in names or []: table.setdefault(path + (n,), 'v')
        print('depth', depth, 'modules', len(frontier), 'next', len(nxt), 'enums', len(enums), 'paths', len(table), flush=True)
        frontier = nxt
with open('table.txt', 'w') as f:
    for p in sorted(table):
        if table[p] != 'u': f.write('%s %s\n' % ('::'.join(p), table[p]))
print('done', len(table))
