// expect: L0306
// rustc E0597: the copied `&str` items still borrow `owned`.
use std::collections::HashSet;

fn main() {
    let mut s: HashSet<&str> = HashSet::new();
    {
        let owned = String::from("x");
        let v = vec![owned.as_str()];
        s.extend(&v);
    }
    println!("{s:?}");
}
