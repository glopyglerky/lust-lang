// expect: L0001
// rustc: character constant must be escaped: `\t` (a tab between the quotes).
fn main() { let c = '	'; println!("{}", c as u32); }
