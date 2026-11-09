// expect: L0733
// rustc E0521: retain's `&K` is higher-ranked, so it cannot leave the predicate.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u32, u32> = HashMap::new();
    m.insert(1, 2);
    let mut out: Vec<&u32> = Vec::new();
    m.retain(|k, _| {
        out.push(k);
        true
    });
    println!("{}", out.len());
}
