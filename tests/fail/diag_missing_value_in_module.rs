// expect: L0101
mod a { pub fn present() -> u8 { 1 } }
fn main() { println!("{}", a::gone()); }
