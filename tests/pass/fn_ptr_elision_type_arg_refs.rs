// Fn pointer elision counts only the lifetimes written in the inputs: a
// generic `fn(&(K, V)) -> (&K, &V)` instantiated with `K = &str` still
// has one input lifetime (rustc), so the output elides to it.
fn pr2<K, V>(p: &(K, V)) -> (&K, &V) {
    (&p.0, &p.1)
}
fn it<K, V>(v: &Vec<(K, V)>) -> usize {
    v.iter().map(pr2::<K, V> as fn(&(K, V)) -> (&K, &V)).count()
}
fn main() {
    println!("{} {}", it(&vec![("a", 2)]), it(&vec![(1u8, "b"), (2, "c")]));
}
