// expect: L0110
// rustc E0308: only std and adapter source may turn &[T] into a reference to an unsized newtype (a pointer cast in Rust).
#![allow(unused)]
struct S<T> { x: [T] }
struct W<T: ?Sized> { x: T }
fn f(s: &[u8]) -> &S<u8> { s }
fn main() {}
