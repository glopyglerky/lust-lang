// feature: ext.tikv_jemalloc_ctl
// form: thread.arena and opt.narenas by name, as program.rs and jemalloc_layout.rs write them
// cargo-dep: tikv-jemalloc-ctl = "=0.7.0"
// cargo-dep: tikv-jemallocator = "=0.7.0"
use tikv_jemalloc_ctl::{Access, AsName};
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
fn main() {
    let arenas = Access::<u32>::read(b"opt.narenas\0".name()).unwrap();
    let own: Option<u32> = b"thread.arena\0".name().read().ok();
    let _ = b"thread.arena\0".name().write(own.unwrap());
    println!("{} {:?}", arenas >= 1, own);
}
