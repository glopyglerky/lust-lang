// expect: L0104
// rustc E0599: Box::write is on Box<MaybeUninit<T>>, not Box<u8>.
fn main() { let b = Box::<u8>::write(Box::new_uninit(), 1u8); println!("{}", b); }
