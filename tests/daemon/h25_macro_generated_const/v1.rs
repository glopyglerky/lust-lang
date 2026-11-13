macro_rules! defk { () => { const K: u32 = 3; } }
defk!();
fn main() { println!("{}", K); }
