// expect: L0306
// rustc E0597: retain's &T reaches a Cell, so the predicate stores &t in the Vec.
use std::cell::Cell;
fn main() {
    let mut v = vec![Cell::new("a")];
    {
        let t = String::from("x");
        v.retain(|k| { k.set(&t); true });
    }
    println!("{}", v[0].get());
}
