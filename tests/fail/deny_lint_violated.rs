// expect: L0531
// rustc rejects the unused variable at deny; lust does not check
// the lint, so it rejects the deny itself (L0531).
#![deny(unused_variables)]

fn main() {
    let x = 1;
    println!("hi");
}
