// expect: L0110
// rustc E0107: `M` takes two type arguments.
type M<K, V> = Vec<(K, V)>;
fn main() { let m: M<u32> = Vec::new(); println!("{}", m.len()); }
