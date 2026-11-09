// `type M<K, V> = Vec<(K, V)>` with a reference key type.
type M<K, V> = Vec<(K, V)>;
fn main() { let mut m: M<&str, u32> = Vec::new(); m.push(("a", 1)); println!("{:?}", m); }
