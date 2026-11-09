# Name pools from the std source: every identifier (pool.txt) and the
# names the source declares pub or as a macro (pubnames.txt).
import os, re, sys
src = sys.argv[1]
KW = set("as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn abstract become box do final macro override priv typeof unsized virtual yield try gen union _".split())
ids, pubs = set(), set()
decl = re.compile(r'\bpub(?:\([^)]*\))?\s+(?:(?:unsafe|const|async|safe)\s+|extern\s+"[^"]*"\s+)*(?:fn|struct|enum|trait|type|mod|const|static|union|macro)\s+(r#)?([A-Za-z_]\w*)')
mac = re.compile(r'\bmacro_rules!\s*([A-Za-z_]\w*)|\bmacro\s+([A-Za-z_]\w*)')
asr = re.compile(r'\bas\s+([A-Za-z_]\w*)')
for d in ['core/src', 'alloc/src', 'std/src', 'stdarch/crates/core_arch/src', 'stdarch/crates/std_detect/src', 'std_detect/src']:
    for root, _, files in os.walk(os.path.join(src, d)):
        for f in files:
            if not f.endswith('.rs'):
                continue
            t = open(os.path.join(root, f), errors='ignore').read()
            ids.update(re.findall(r'\b[A-Za-z_][A-Za-z0-9_]*\b', t))
            for m in decl.finditer(t): pubs.add(m.group(2))
            for m in mac.finditer(t): pubs.add(m.group(1) or m.group(2))
            for line in t.splitlines():
                if 'use ' in line:
                    pubs.update(asr.findall(line))
ids = sorted(n for n in ids if n not in KW and len(n) <= 60)
pubs = sorted(n for n in pubs if n not in KW)
open('pool.txt', 'w').write('\n'.join(ids) + '\n')
open('pubnames.txt', 'w').write('\n'.join(pubs) + '\n')
print(len(ids), len(pubs))
