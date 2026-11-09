// expect: L0603
// rustc E0658: the `!` type is experimental outside a fn return type (hunt5).
#![allow(unused)]
enum Empty {}
enum E { A(std::cell::Cell<Empty>), B(u8) }
fn main() { println!("{} {} {}", std::mem::size_of::<std::cell::Cell<Empty>>(), std::mem::size_of::<E>(), std::mem::size_of::<std::cell::Cell<!>>()); }
