// expect: L0733
// rustc: lifetime may not live long enough: the pointer's output is `'static`, so the closure
// cannot return a borrow of its argument.
fn main() {
    let f: fn(&Vec<u8>) -> &'static u8 = |v| &v[0];
    let v = vec![1u8];
    println!("{}", f(&v));
}
