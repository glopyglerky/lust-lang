// expect: L0307
// rustc E0515: a mutable borrow of a constant is not promoted, so the
// slice borrows a temporary of this function.
fn f() -> &'static [i32] {
    &mut [1, 2][..]
}
fn main() {
    println!("{:?}", f());
}
