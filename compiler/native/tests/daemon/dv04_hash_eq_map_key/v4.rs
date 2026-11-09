use std::collections::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
enum K {
    A,
    B(u8),
}
fn count<T: std::hash::Hash + Eq + Copy>(ks: &[T]) -> HashMap<T, usize> {
    let mut m = HashMap::new();
    for k in ks {
        *m.entry(*k).or_insert(0) += 1;
    }
    m
}
fn main() {
    let m = count(&[K::A, K::B(1), K::A]);
    println!("{} {}", m[&K::A], m.len());
}
