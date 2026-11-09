// expect: L0306
// rustc E0597: the closure stores `&s[..n]` into `out`, which outlives
// `s`.
fn main() {
    let mut out: Vec<&str> = Vec::new();
    {
        let s = String::from("hello");
        let mut add = |n: usize| out.push(&s[..n]);
        add(1);
    }
    println!("{:?}", out);
}
