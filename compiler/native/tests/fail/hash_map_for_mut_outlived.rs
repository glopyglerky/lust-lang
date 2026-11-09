// expect: L0306
// rustc E0597: a for loop over &mut map stores a borrow of `s`.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    m.insert(1, "a");
    {
        let s = String::from("x");
        for (_, v) in &mut m {
            *v = &s;
        }
    }
    println!("{:?}", m);
}
