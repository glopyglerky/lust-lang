// lust-reject: L0104
// tikv-jemalloc-ctl 0.7.0: the by-name arena form ts-rust
// writes (program.rs, jemalloc_layout.rs). The real values
// depend on the machine and the build (opt.narenas is 4 per
// CPU by default, a spawned thread's arena is not 0), so the
// adapter has no Access impls and lust must reject this at
// compile time (w-cinterop3 audit). Cargo builds it fine.
use tikv_jemalloc_ctl::{Access, AsName};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    let arenas = Access::<u32>::read(b"opt.narenas\0".name()).unwrap();
    let own: Option<u32> = b"thread.arena\0".name().read().ok();
    let _ = b"thread.arena\0".name().write(own.unwrap());
    println!("{} {:?}", arenas >= 1, own);
}
