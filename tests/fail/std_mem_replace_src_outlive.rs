// expect: L0306
// rustc E0597: `src` moves into `*dest`, which outlives `s`.
fn main() {
    let mut v: Vec<&str> = vec![];
    {
        let s = String::from("x");
        let old = std::mem::replace(&mut v, vec![&s]);
        println!("{}", old.len());
    }
    println!("{:?}", v);
}
