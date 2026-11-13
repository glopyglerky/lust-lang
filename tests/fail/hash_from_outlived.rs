// expect: L0306
// rustc E0597: the map keeps the `&str` key that borrows `owned`.
use std::collections::HashMap;
fn main() {
    let m;
    {
        let owned = String::from("x");
        m = HashMap::from([(owned.as_str(), 1)]);
    }
    println!("{m:?}");
}
