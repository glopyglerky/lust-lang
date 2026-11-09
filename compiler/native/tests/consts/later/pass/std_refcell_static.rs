// stage: W-STD-B
// ARCH 2.13: the real source-std form of the ts-rust shape at
// ast/store.rs:1157, a thread_local RefCell built with
// RefCell::new over an aggregate with a Vec. Gates with
// W-STD-B in wave B, where RefCell lands. The A1 form is
// tests/consts/pass/local_cell_static.rs.
use std::cell::RefCell;
struct BuildStores {
    base: u32,
    stores: Vec<u32>,
}
thread_local! {
    static BUILD: RefCell<BuildStores> = const {
        RefCell::new(BuildStores { base: 3, stores: Vec::new() })
    };
}
fn main() {
    BUILD.with(|b| {
        let mut b = b.borrow_mut();
        let next = b.base + 1;
        b.stores.push(next);
        println!("{} {}", b.base, b.stores[0]);
    });
}
