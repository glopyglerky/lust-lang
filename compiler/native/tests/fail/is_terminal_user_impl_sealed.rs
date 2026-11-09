// expect: L0612
// rustc E0277: IsTerminal is sealed; a user type cannot implement it.
use std::io::IsTerminal;
struct S;
impl IsTerminal for S { fn is_terminal(&self) -> bool { true } }
fn main() { println!("{}", S.is_terminal()); }
