// expect: L0306
// rustc E0597: the alias's `'static` binds the parameter.
type S = &'static str;

fn keep(x: S) -> usize {
    x.len()
}

fn main() {
    let s = String::from("ab");
    println!("{}", keep(&s));
}
