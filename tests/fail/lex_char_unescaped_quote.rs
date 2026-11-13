// expect: L0001
// rustc: character constant must be escaped: `'`, also in macro input.
fn main() { println!("{}", stringify!(''')); }
