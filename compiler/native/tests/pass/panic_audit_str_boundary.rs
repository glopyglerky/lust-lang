// A str slice inside a char names the char and its bytes.
// panic: end byte index 3 is not a char boundary; it is inside '😀' (bytes 1..5 of string)
fn main() {
    let n = std::env::args().count();
    let s = String::from("a😀b");
    println!("{} {}", &s[..n], &s[n + 4..]);
    println!("{}", &s[n - 1..n + 2]);
}
