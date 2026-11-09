// A tuple struct's where clause sits after the fields, before the
// `;` (W-BENCH3 bug 14). The bound is checked at uses as usual.
struct Store<V>(Vec<V>)
where
    V: Default;
struct Pair<A, B>(A, B)
where
    A: Clone,
    B: Clone;
fn len<V: Default>(s: &Store<V>) -> usize {
    s.0.len()
}
fn main() {
    let s: Store<u8> = Store(Vec::new());
    let p: Pair<u8, String> = Pair(3, String::from("x"));
    println!("{} {} {}", s.0.len(), len(&s), p.0);
}
