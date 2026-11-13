// expect: L0001
// rustc: byte constant must be escaped: `\n` (a newline between the quotes).
fn main() { println!("{}", stringify!(b'
')); }
