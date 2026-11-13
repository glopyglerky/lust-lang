// expect: L9001
// Valid Rust (`impl From<&String> for String`). Lust's `String::from`
// as a value takes `&str`, `char`, or `String`.
fn main() {
    let names = vec![String::from("a")];
    let copies: Vec<String> = names.iter().map(String::from).collect();
    println!("{copies:?}");
}
