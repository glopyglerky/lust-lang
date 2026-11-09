// expect: L0612
// rustc E0277: `String: From<&&str>` does not hold; `String::from` picks
// its impl by the argument's own type, with no deref coercion.
fn main() {
    let s = "a";
    let t = String::from(&s);
    println!("{}", t);
}
