// expect: L0306
// rustc E0597: the map keeps the `&str` keys that borrow `owned`.
use std::collections::HashMap;

fn main() {
    let m: HashMap<&str, i32>;
    {
        let owned = String::from("x");
        let v = vec![owned.as_str()];
        m = v.iter().map(|k| (*k, 1)).collect();
    }
    println!("{m:?}");
}
