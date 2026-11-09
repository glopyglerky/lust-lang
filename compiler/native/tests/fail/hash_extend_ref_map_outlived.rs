// expect: L0306
// rustc E0597: the copied `&str` keys still borrow `owned`.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<&str, i32> = HashMap::new();
    {
        let owned = String::from("x");
        let mut other: HashMap<&str, i32> = HashMap::new();
        other.insert(owned.as_str(), 1);
        m.extend(&other);
    }
    println!("{m:?}");
}
