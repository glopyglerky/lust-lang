// expect: L0104
// rustc E0599: `write!` to a String needs fmt::Write in scope.
fn main() {
    let mut s = String::new();
    let _ = write!(s, "{}", 1);
    println!("{}", s);
}
