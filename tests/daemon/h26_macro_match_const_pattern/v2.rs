const K: u8 = 4;
macro_rules! m { ($x:expr) => { match $x { K => 1u8, _ => 0u8 } } }
fn main() { println!("{} {}", m!(3u8), m!(4u8)); }
