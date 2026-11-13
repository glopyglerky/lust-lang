// expect: L0306
// rustc E0597: `copied` items are the inner `&str`, which borrows `s`.
fn main() {
    let r: Option<&str>;
    {
        let s = String::from("x");
        let v: Vec<&str> = vec![s.as_str()];
        r = v.iter().copied().next();
    }
    println!("{:?}", r);
}
