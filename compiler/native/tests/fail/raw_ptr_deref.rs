// expect: L0113
// A raw pointer deref needs an unsafe block in Rust; lust has no
// unsafe, so the deref itself is the error.
fn main() {
    let x = 4;
    let p: *const i32 = &x;
    println!("{}", *p);
}
