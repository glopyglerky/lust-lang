// `map[&q]` with a temporary key: the String temporary lives to the end
// of the statement and is dropped once (W-STD-B).
use std::collections::HashMap;

struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let mut n: HashMap<String, u32> = HashMap::new();
    n.insert(String::from("x"), 5);
    println!("{}", n[&String::from("x")] + 1);
    let v = n[&format!("{}", "x")];
    println!("{}", v);
    let mut w: HashMap<u8, Noisy> = HashMap::new();
    w.insert(1, Noisy("one"));
    println!("{}", w[&1].0);
    let k = 1u8;
    let s = { let t = w[&k].0; t };
    println!("{}", s);
}
