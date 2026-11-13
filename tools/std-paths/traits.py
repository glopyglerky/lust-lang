# Step 4 (see gen.sh): marks the traits in table.txt (kind 't'), in
# place. A trait takes `use path::Trait::{};` as a module or enum
# does; any other item is E0432 there. Probes every uppercase item.
import json, subprocess, sys
path = sys.argv[1]
rows = [l.split() for l in open(path)]
cands = [p for p, k in rows if k == 'i' and p.rsplit('::', 1)[-1][:1].isupper()]
traits = set()
for root in ['std', 'core', 'alloc']:
    ps = [p for p in cands if p.split('::')[0] == root]
    head = [] if root == 'std' else ['#![no_std]'] + (['extern crate alloc;'] if root == 'alloc' else [])
    lines = head + ['#[no_implicit_prelude] mod t {']
    base = len(lines)
    lines += ['mod g%d { use ::%s::{}; }' % (i, p) for i, p in enumerate(ps)] + ['}']
    open(f'traits_{root}.rs', 'w').write('\n'.join(lines) + '\n')
    r = subprocess.run(['rustc', '--edition', '2024', '--crate-type', 'lib', '--emit=metadata', '-o', f'traits_{root}.rmeta', '--error-format=json', f'traits_{root}.rs'], capture_output=True, text=True)
    bad = set()
    for l in r.stderr.splitlines():
        try: d = json.loads(l)
        except Exception: continue
        if d.get('level') == 'error':
            bad.update(s['line_start'] for s in d.get('spans', []) if s.get('is_primary'))
    if ps and not bad:
        sys.exit(f'traits_{root}.rs: no errors at all, rustc died early?')
    traits.update(p for i, p in enumerate(ps) if base + 1 + i not in bad)
with open(path, 'w') as f:
    for p, k in rows:
        f.write('%s %s\n' % (p, 't' if p in traits else k))
print('traits', len(traits))
