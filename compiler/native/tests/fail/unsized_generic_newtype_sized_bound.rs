// expect: L0612
// rustc E0277: S<u8> is not Sized, and g needs T: Sized.
#![allow(unused)]
struct S<T> { x: [T] }
struct W<T: ?Sized> { x: T }
fn g<T>(t: &T) {} fn f(s: &S<u8>) { g(s) }
fn main() {}
