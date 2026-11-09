// ARCH 2.7 and 3.1 auto-trait parity: the std types lust has, user
// types through their fields, generic types through their arguments,
// references by the &T/&mut T table, and Unpin for Vec, Box, and Rc of
// any element type. Every bound here holds in rustc.
use std::rc::Rc;
fn send<T: Send>(_t: &T) -> u8 { 1 }
fn sync<T: Sync>(_t: &T) -> u8 { 1 }
fn unpin<T: Unpin>(_t: &T) -> u8 { 1 }
struct P { a: u8, s: String, v: Vec<Option<u16>> }
struct W<T> { t: T }
#[derive(Debug)]
enum E { A(Box<u8>), B((char, bool)) }
fn generic<T: Send + Sync>(t: T) -> u8 { send(&t) + sync(&t) + send(&&t) }
fn main() {
    let n = send(&String::new()) + sync(&vec![1u8]) + send(&Box::new(1u8)) + sync(&Some(String::new()))
        + send(&(1u8, String::new())) + sync(&[1u8, 2]) + send(&&5u8) + send(&&mut 5u8) + sync(&"ab".chars())
        + send(&P { a: 1, s: String::new(), v: vec![] }) + sync(&W { t: 1u64 }) + send(&E::A(Box::new(1)))
        + send(&1.cmp(&2)) + unpin(&Rc::new(1u8)) + unpin(&vec![Rc::new(1u8)]) + unpin(&Box::new(Rc::new(1u8)))
        + unpin(&std::env::args()) + generic(3u8);
    let r: Result<u8, String> = Ok(1);
    let m = send(&r) + sync(&(|| 1u8)) + send(&{ let x = 5u8; move || x });
    println!("{} {}", n, m);
}
