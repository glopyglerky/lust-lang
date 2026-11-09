fn f<const N: u8>(b: bool) -> u8 { if b { N + 200 } else { 1 } }
fn main() { println!("{}", f::<100>(false)); }
