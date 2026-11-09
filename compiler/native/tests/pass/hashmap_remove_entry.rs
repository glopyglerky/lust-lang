// HashMap::remove_entry gives back the stored key and value.
use std::collections::HashMap;
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let mut m: HashMap<String, u8> = HashMap::new();
    m.insert("a".to_string(), 1);
    println!("{:?} {:?} {}", m.remove_entry("a"), m.remove_entry("b"), m.len());
    let mut d = HashMap::new();
    d.insert(1, D(1));
    d.insert(2, D(2));
    if let Some((k, v)) = d.remove_entry(&1) {
        println!("got {k} {}", v.0);
    }
    println!("left {}", d.len());
}
