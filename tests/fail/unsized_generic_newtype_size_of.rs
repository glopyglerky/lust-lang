// expect: L0652
// rustc E0277: size_of needs a Sized type.
#![allow(unused)]
struct S<T> { x: [T] }
struct W<T: ?Sized> { x: T }
fn f() -> usize { std::mem::size_of::<S<u8>>() }
fn main() {}
