// expect: L0110
// rustc E0308: `std::boxed::Box::<u8>::new` takes a u8.
fn main() { let b = std::boxed::Box::<u8>::new(300u16); println!("{}", b); }
