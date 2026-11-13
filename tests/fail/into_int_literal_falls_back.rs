// expect: L0612
// rustc E0277: the literal falls back to i32, and `u64: From<i32>` does not hold.
fn f<T: Into<u64>>(t: T) -> u64 {
    t.into()
}
fn main() {
    println!("{}", f(300));
}
