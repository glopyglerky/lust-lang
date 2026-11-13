// expect: L0104
// rustc E0599: Borrow is not in the prelude; `borrow` needs the use.
fn main() {
    let s = String::from("a");
    let r: &str = s.borrow();
    println!("{}", r);
}
