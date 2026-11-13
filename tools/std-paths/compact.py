# table.txt (`<path> <kind>` per line, kind m/e/t/i/v) to the compact
# std_paths.txt format of src/resolve/stdpaths.rs: one line per module
# or enum, `<path> <name>[/m|/e|/t]...`; a container whose item list equals
# an earlier one's is written `<path> = <earlier path>`.
import sys
kinds = {}
for line in open(sys.argv[1]):
    p, k = line.split()
    kinds[p] = k
kids = {}
for p, k in kinds.items():
    if '::' not in p:
        continue
    parent, name = p.rsplit('::', 1)
    if kinds.get(parent) in ('m', 'e'):
        kids.setdefault(parent, []).append(name + {'m': '/m', 'e': '/e', 't': '/t'}.get(k, ''))
roots = [r for r in ('std', 'core', 'alloc') if kinds.get(r) == 'm']
order = sorted(kids, key=lambda p: ({'core': 0, 'alloc': 1, 'std': 2}[p.split('::')[0]], p))
seen = {}
out = []
for p in order:
    items = ' '.join(sorted(kids[p]))
    if items in seen:
        out.append(f'{p} = {seen[items]}')
    else:
        seen[items] = p
        out.append(f'{p} {items}')
open(sys.argv[2], 'w').write('\n'.join(out) + '\n')
print(len(out), sum(len(l) for l in out))
