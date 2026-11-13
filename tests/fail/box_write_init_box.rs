// expect: L0110
// rustc E0308: Box::write takes a Box<MaybeUninit<T>>, not a Box<T>.
fn main() { let b = Box::new(5u8); let c = Box::write(b, 6); println!("{}", c); }
