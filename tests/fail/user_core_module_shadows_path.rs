// expect: L0102
// rustc E0433: at the root, a user `mod core` shadows the `core` crate in a
// plain path too.
mod core {
    pub fn f() -> u32 {
        7
    }
}
fn main() {
    let o: core::cmp::Ordering = 1.cmp(&2);
    println!("{:?} {}", o, core::f());
}
