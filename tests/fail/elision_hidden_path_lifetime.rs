// expect: L0202
// rustc E0106: `&Holder` has two input lifetimes (the reference's and the
// one hidden in `Holder`), so the output cannot pick.
struct Holder<'a> {
    s: &'a str,
}

fn get(h: &Holder) -> &str {
    h.s
}

fn main() {
    let x = String::from("x");
    let h = Holder { s: &x };
    println!("{}", get(&h));
}
