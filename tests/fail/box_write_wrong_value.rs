// expect: L0110
// rustc E0308: the written value must have the slot's type.
fn main() { let b = Box::write(Box::<u16>::new_uninit(), 1u8); println!("{}", b); }
