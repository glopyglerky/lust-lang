// expect: L0306
// rustc E0597: the value `replace` returns keeps the borrow of `s`.
fn main() {
    let r;
    {
        let s = String::from("a");
        let mut v = vec![s.as_str()];
        r = std::mem::replace(&mut v, vec![]);
    }
    println!("{:?}", r);
}
