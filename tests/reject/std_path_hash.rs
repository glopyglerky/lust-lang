// expect: L0612
// Path hashes by components in std; lust's Path and PathBuf have no
// Hash impl yet, so a PathBuf map key has no `Hash` (L0612).
use std::collections::HashMap;
use std::path::PathBuf;
fn main() {
    let mut s: HashMap<PathBuf, u8> = HashMap::new();
    s.insert("a".into(), 1);
    println!("{}", s.len());
}
