// expect: L0001
// rustc: null characters in C string literals are not supported (lexer error).
fn main() { println!("{}", stringify!(c"a\x00b")); }
