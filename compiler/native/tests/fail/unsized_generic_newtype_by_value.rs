// expect: L0110
// rustc E0277: a by-value parameter must be Sized.
#![allow(unused)]
struct S<T> { x: [T] }
struct W<T: ?Sized> { x: T }
fn f<T>(s: S<T>) {}
fn main() {}
