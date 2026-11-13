// expect: L0306
// rustc E0597: the set keeps a borrow of `t`.
use std::collections::HashSet;

fn main() {
    let mut s: HashSet<&str> = HashSet::new();
    {
        let t = String::from("x");
        s.insert(&t);
    }
    println!("{:?}", s);
}
