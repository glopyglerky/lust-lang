// expect: L0102
// A type path from `alloc` with no `extern crate alloc;` at the crate
// root is rustc E0433, in a single-file crate with no `use` too.
fn main() {
    let v: alloc::vec::Vec<i32> = Vec::new();
    println!("{}", v.len());
}
