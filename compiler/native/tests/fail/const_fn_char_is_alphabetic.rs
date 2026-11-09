// expect: L0631
// rustc E0015: is_alphabetic is not a const fn, also inside a const fn.
const fn f(c: char) -> bool {
    c.is_alphabetic()
}
fn main() {
    println!("{}", f('é'));
}
