// expect: L0001
fn main() { let mut _y: u8 = 0; #[allow(arithmetic_overflow)] _y = 1; println!("x"); }
