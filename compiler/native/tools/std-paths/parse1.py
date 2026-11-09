# Reads rustc JSON diagnostics, prints suggested `use` paths.
import sys,json,re
seen=set()
for line in sys.stdin:
    try: d=json.loads(line)
    except: continue
    def walk(x):
        for c in x.get('children',[]):
            for s in c.get('spans',[]):
                r=s.get('suggested_replacement') or ''
                for m in re.finditer(r'use ((?:[A-Za-z_][A-Za-z0-9_]*::)+[A-Za-z_][A-Za-z0-9_]*);', r):
                    seen.add(m.group(1))
            walk(c)
    walk(d)
for p in sorted(seen): print(p)
