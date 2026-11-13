fn f<const N: u8>(b: bool) -> u8 { if b { 10 / N } else { 3 } }
fn main() { println!("{}", f::<0>(false)); }
