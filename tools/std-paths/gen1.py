# Step 1: probe files that make rustc suggest an import for each name.
# usage: gen1.py names.txt mode(std|core|alloc) shard nshards > file.rs (see gen.sh)
import sys
KW = set("as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn abstract become box do final macro override priv typeof unsized virtual yield try gen union _".split())
names=[l.strip() for l in open(sys.argv[1]) if l.strip() and l.strip() not in KW]
mode=sys.argv[2]; k=int(sys.argv[3]); n=int(sys.argv[4])
names=names[k::n]
out=[]
if mode!='std': out.append('#![no_std]')
if mode=='alloc': out.append('extern crate alloc;')
out.append('#[no_implicit_prelude] mod t {')
for i,nm in enumerate(names):
    out.append(f'fn f{i}() {{ let _: {nm}; let _ = {nm}; }}')
out.append('}')
if mode!='std': out.append('#[panic_handler] fn ph(_: &core::panic::PanicInfo) -> ! { loop {} }')
out.append('fn main() {}' if mode=='std' else '')
print('\n'.join(out))
