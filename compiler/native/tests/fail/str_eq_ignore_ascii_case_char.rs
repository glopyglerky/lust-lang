// expect: L0110
// rustc E0308: str::eq_ignore_ascii_case takes a &str, not a char.
fn main() { println!("{}", "a".eq_ignore_ascii_case('A')); }
