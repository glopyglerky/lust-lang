// expect: L0306
// rustc E0597: the map keeps the `&str` that borrows `owned`.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<i32, &str> = HashMap::new();
    {
        let owned = String::from("x");
        m.extend([(1, owned.as_str())]);
    }
    println!("{m:?}");
}
