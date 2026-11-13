// rustc accepts: the one hidden lifetime in `Holder` is the only input
// lifetime, so the output takes it.
struct Holder<'a> {
    s: &'a str,
}

fn get(h: Holder) -> &str {
    h.s
}

fn main() {
    let x = String::from("x");
    println!("{}", get(Holder { s: &x }));
}
