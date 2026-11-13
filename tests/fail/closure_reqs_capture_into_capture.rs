// expect: L0306
// rustc E0597: the body pushes the captured `&s` into `out`, so the
// creator needs the borrow of `s` to outlive `out`'s elements.
fn main() {
    let mut out: Vec<&String> = Vec::new();
    {
        let s = String::from("a");
        let mut f = || out.push(&s);
        f();
    }
    println!("{}", out.len());
}
