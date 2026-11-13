// expect: L0733
// rustc E0308: `bad` returns its argument's lifetime, but `P: Fn(&D) -> &'static str` needs `'static`.
struct D {
    msg: String,
}

fn bad(d: &D) -> &str {
    &d.msg
}

fn by<P: Fn(&D) -> &'static str>(d: &D, p: &P) -> usize {
    p(d).len()
}

fn main() {
    let d = D { msg: String::from("mm") };
    println!("{}", by(&d, &bad));
}
