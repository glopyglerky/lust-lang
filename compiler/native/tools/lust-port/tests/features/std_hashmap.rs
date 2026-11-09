// feature: std.hashmap
// form: std HashMap and HashSet with the default hasher, insert get and entry
use std::collections::{HashMap, HashSet};
fn main() {
    let mut m: HashMap<String, i32> = HashMap::new();
    m.insert("a".to_string(), 1);
    *m.entry("b".to_string()).or_insert(0) += 2;
    let mut s: HashSet<i32> = HashSet::new();
    s.insert(5);
    println!("{} {} {}", m.get("a").copied().unwrap_or(0), m.get("b").copied().unwrap_or(0), s.contains(&5));
}
