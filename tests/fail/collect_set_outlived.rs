// expect: L0306
// rustc E0597: the set keeps the `&str` items that borrow `owned`.
use std::collections::HashSet;

fn main() {
    let s: HashSet<&str>;
    {
        let owned = String::from("x");
        let v = vec![owned.as_str()];
        s = v.iter().copied().collect();
    }
    println!("{s:?}");
}
