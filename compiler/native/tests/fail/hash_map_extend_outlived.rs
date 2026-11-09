// expect: L0306
// rustc E0597: extend from a Vec of pairs that borrow `w`.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<&str, u32> = HashMap::new();
    {
        let w = String::from("k");
        let pairs = vec![(w.as_str(), 1u32)];
        m.extend(pairs);
    }
    println!("{}", m.len());
}
