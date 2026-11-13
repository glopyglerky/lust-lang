// `type WV<T> = W<Vec<T>>` in a param and a let annotation.
struct W<T> { t: T }
type WV<T> = W<Vec<T>>;
fn total(w: &WV<u32>) -> u32 { w.t.iter().sum() }
fn main() { let w: WV<u32> = W { t: vec![1, 2, 3] }; println!("{}", total(&w)); }
