// `String::remove` at the length.
// panic: cannot remove a char from the end of a string
fn main() {
    let n = std::env::args().count();
    let mut s = String::from("ab");
    println!("{}", s.remove(n - 1));
    println!("{}", s.remove(n));
}
