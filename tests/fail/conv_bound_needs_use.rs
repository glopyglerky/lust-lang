// expect: L0102
// rustc E0405: a bare non-prelude trait name in a bound needs a use.
fn f<K: Borrow<str>>(k: K) -> usize {
    let _ = k;
    0
}
fn main() {
    println!("{}", f(String::from("x")));
}
