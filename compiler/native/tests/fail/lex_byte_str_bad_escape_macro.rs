// expect: L0001
// rustc: unknown byte escape, reported by the lexer, so in macro input too.
fn main() { println!("{}", stringify!(b"\q")); }
