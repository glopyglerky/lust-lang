// expect: L0506
// rustc E0432: upstream `epoch` is a unit struct; its `advance` is an
// associated fn and cannot be imported.
use tikv_jemalloc_ctl::epoch::advance;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    println!("{}", advance().unwrap() > 0);
}
