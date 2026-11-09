// expect: L0306
// rustc E0597: a write through an iter_mut item stores a borrow of `s`,
// which dies before the Vec is printed.
fn main() {
    let mut v: Vec<&str> = vec!["a"];
    {
        let s = String::from("x");
        for r in v.iter_mut() {
            *r = &s;
        }
    }
    println!("{:?}", v);
}
