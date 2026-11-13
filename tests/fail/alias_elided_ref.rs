// expect: L0202
// rustc E0106: a reference in a type alias needs a lifetime.
type A = &str;

fn main() {
    let x: A = "a";
    println!("{}", x);
}
