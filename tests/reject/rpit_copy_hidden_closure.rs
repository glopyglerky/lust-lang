// expect: L0602
// Valid Rust: the closure takes `it` by value (its type is not `Copy` by
// its bounds), but lust's revealed check sees a `Copy` hidden type and
// would let the closure borrow it, so lust rejects this form for now.
fn consume<T>(_t: T) {}
fn len_of(v: &Vec<u32>) -> impl Sized {
    v.len()
}
fn main() {
    let v = vec![1, 2];
    let it = len_of(&v);
    let f = || consume(it);
    f();
    println!("done");
}
