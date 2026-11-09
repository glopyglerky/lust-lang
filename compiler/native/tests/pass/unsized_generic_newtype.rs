// A generic unsized newtype over [T] (R15): declared and used behind references (no safe constructor exists).
#![allow(unused)]
struct S<T> { x: [T] }
fn f<T>(s: &S<T>) -> usize { s.x.len() }
fn len_of<T>(s: &S<T>) -> usize { s.x.len() }
fn parts<T>(s: &S<T>) { let t = &s.x[..]; let u = s.x.len(); }
fn owned<T: Clone>(s: &S<T>) -> Vec<T> { s.x.to_vec() }
fn main() { let _ = (len_of::<u8> as fn(&S<u8>) -> usize, owned::<u8> as fn(&S<u8>) -> Vec<u8>); println!("ok"); }
