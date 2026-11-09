// `[T; 1]: Default` as the argument of a `V: Default` parameter,
// built in an expression before inference names `T` (W-BENCH3
// bug 15): the obligation must wait, not fail on `[_; _]`.
struct Store<V: Default>(Vec<V>);
struct C<T: Default> {
    s: Store<[T; 1]>,
}
fn main() {
    let c: C<u8> = C { s: Store(Vec::new()) };
    println!("{}", c.s.0.len());
    let c2: C<String> = C { s: Store(vec![[String::from("a")]]) };
    println!("{} {}", c2.s.0.len(), c2.s.0[0][0]);
}
