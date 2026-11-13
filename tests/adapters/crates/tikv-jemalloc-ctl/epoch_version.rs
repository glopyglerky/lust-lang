// tikv-jemalloc-ctl 0.7.0: the kept surface (w-cinterop3
// audit). version is the exact string of the pinned jemalloc
// build; epoch prints absolute counts (a fresh real process
// reads 1 first, each advance adds one - probed against the
// real crate); Name and AsName are pure.
use tikv_jemalloc_ctl::{AsName, epoch, version};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    println!("{:?}", version::read().unwrap());
    println!("{}", version::read().unwrap().len());
    println!("{}", epoch::read().unwrap());
    println!("{}", epoch::advance().unwrap());
    println!("{}", epoch::advance().unwrap());
    println!("{}", epoch::read().unwrap());
    println!("{}", epoch::update(0).unwrap());
    epoch::write(99).unwrap();
    println!("{}", epoch::read().unwrap());
    // Pure Name forms.
    println!("{}", b"version\0".name().value_type_str());
    println!("{}", b"epoch\0".name().value_type_str());
    println!("{}", b"arena.0.dss\0".name().value_type_str());
    println!("{}", b"stats.arenas.0.dss\0".name().value_type_str());
    println!("{}", b"thread.arena\0".name().value_type_str());
    println!("{}", b"opt.narenas\0".name().as_bytes().len());
    println!("{}", "epoch\0".name().as_bytes().len());
}
