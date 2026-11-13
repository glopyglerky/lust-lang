// expect: L0806
// rustc: no rules expected `;`
// A braced item ends at its brace; the `;` after it is not part of it.
macro_rules! m { ($s:stmt ; [$($t:tt)*]) => { () }; }
fn main() { m!(fn h() {} ; ; [x]); }
