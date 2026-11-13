type A = u8;
fn main() { let x: A = 7; println!("{} {}", x, std::mem::size_of::<A>()); }
