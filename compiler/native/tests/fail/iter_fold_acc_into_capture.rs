// expect: L0306
// rustc E0597: the body stores the accumulator, which holds the items'
// borrows of `s`, into `keep`.
fn main() {
    let mut keep: Vec<&str> = Vec::new();
    {
        let s = String::from("x");
        let v: Vec<&str> = vec![s.as_str()];
        let _ = v.iter().fold("", |a, x| {
            keep.push(a);
            *x
        });
    }
    println!("{:?}", keep);
}
