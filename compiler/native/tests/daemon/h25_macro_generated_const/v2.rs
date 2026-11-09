macro_rules! defk { () => { const K: u32 = 9; } }
defk!();
fn main() { println!("{}", K); }
