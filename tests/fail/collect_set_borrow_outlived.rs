// expect: L0306
// rustc E0597: the `&&str` items borrow `v` itself.
use std::collections::HashSet;

fn main() {
    let s: HashSet<&&str>;
    {
        let v = vec!["a"];
        s = v.iter().collect();
    }
    println!("{s:?}");
}
