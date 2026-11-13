// The Drain type by its std path, and a drain through a &mut parameter (R20).
use std::collections::HashMap; fn named() { let mut m: HashMap<u32, u32> = HashMap::new(); m.insert(1, 1); let mut d = m.drain(); let x = d.next(); let y: std::collections::hash_map::Drain<u32, u32> = d; println!("{:?} {}", x, y.len()); }
fn f(m: &mut HashMap<String, u32>) -> u32 { m.drain().map(|(_, v)| v).sum() } fn main() { named(); let mut m = HashMap::new(); m.insert("a".to_string(), 3); m.insert("b".to_string(), 4); println!("{} {}", f(&mut m), m.len()); }
