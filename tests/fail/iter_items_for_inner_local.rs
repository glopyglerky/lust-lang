// expect: L0306
// rustc E0597: the items' inner references borrow `s`, which ends first.
fn main() {
    let mut out: Vec<&str> = Vec::new();
    {
        let s = String::from("x");
        let v: Vec<&str> = vec![s.as_str()];
        for x in v.iter() {
            out.push(*x);
        }
    }
    println!("{:?}", out);
}
