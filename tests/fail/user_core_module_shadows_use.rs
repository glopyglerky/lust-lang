// expect: L0506
// rustc E0432: at the root, a user `mod core` shadows the `core` crate in a
// `use` path, and it has no `time` module.
mod core {
    pub fn f() -> u32 {
        7
    }
}
use core::time::Duration;
fn main() {
    println!("{} {:?}", core::f(), Duration::from_millis(3));
}
