// expect: L0110
// rustc E0308: the turbofish fixes `Box<u8>`; the argument is `Box<u16>`.
fn main() { let b = Box::new(5u16); let l = Box::<u8>::leak(b); println!("{}", l); }
