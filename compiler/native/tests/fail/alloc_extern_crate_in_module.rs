// expect: L0102
// `extern crate alloc;` in a module binds `alloc` in that module only:
// another module's `::alloc::` path is rustc E0433.
mod m {
    extern crate alloc;
    pub use alloc::vec::Vec;
}

mod n {
    pub fn f() -> usize {
        let v: ::alloc::vec::Vec<u8> = super::m::Vec::new();
        v.len()
    }
}

fn main() {
    println!("{}", n::f());
}
