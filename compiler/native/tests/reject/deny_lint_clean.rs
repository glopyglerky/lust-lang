// expect: L0531
// Valid Rust (no lint fires), but lust cannot prove that: a rust
// lint at deny or forbid is L0531 unless lust checks it.
#![deny(unused_variables)]

fn main() {
    println!("hi");
}
