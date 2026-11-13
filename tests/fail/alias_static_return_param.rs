// expect: L0733
// rustc: lifetime may not live long enough. The alias's `'static` binds the return.
type S = &'static str;

fn f(x: &str) -> S {
    x
}

fn main() {
    let s = String::from("a");
    println!("{}", f(&s));
}
