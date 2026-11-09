// expect: L0110
// rustc E0308: `String` has one `PartialOrd` impl (`Rhs = String`).
fn main() {
    let s = String::from("a");
    println!("{}", s.lt("b"));
}
