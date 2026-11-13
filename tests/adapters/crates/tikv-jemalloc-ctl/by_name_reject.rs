// lust-reject: L0104
// tikv-jemalloc-ctl 0.7.0: by-name reads of epoch, version,
// and thread.tcache.enabled. The typed epoch and version
// modules stay (their values match), but the by-name Access
// form cannot be kept for one key without the others, and
// tcache depends on the build config, so every by-name read
// rejects (w-cinterop3 audit). Cargo builds it fine.
use tikv_jemalloc_ctl::{Access, AsName};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    let e: u64 = b"epoch\0".name().read().unwrap();
    let v: &'static str = b"version\0".name().read().unwrap();
    let t: bool = b"thread.tcache.enabled\0".name().read().unwrap();
    println!("{} {} {}", e > 0, v.len(), t);
}
