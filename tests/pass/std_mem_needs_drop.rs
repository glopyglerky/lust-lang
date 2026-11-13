// `mem::needs_drop::<T>()` (W-STD-C): a check-time constant, as rustc's
// const fn answers: Drop impls, owned parts, `[T; 0]` (never), generic
// instances through their template fields, and generic fns per instance.
use std::mem;
use std::mem::needs_drop;
struct D(u8);
impl Drop for D { fn drop(&mut self) {} }
struct P { a: u8, b: (i32, char) }
struct Q { s: String }
#[derive(Clone, Copy)]
struct C(u8);
enum E { A(u8), B(Vec<u8>) }
enum F { A, B(u16) }
fn gnd<T>() -> bool { mem::needs_drop::<T>() }
fn main() {
    println!("{} {} {}", mem::needs_drop::<D>(), mem::needs_drop::<u8>(), mem::needs_drop::<String>());
    println!("{} {} {} {}", needs_drop::<P>(), needs_drop::<Q>(), needs_drop::<C>(), std::mem::needs_drop::<E>());
    println!("{} {} {}", needs_drop::<F>(), needs_drop::<&String>(), needs_drop::<[String; 0]>());
    println!("{} {} {}", needs_drop::<Option<Box<u8>>>(), needs_drop::<(u8, Vec<u8>)>(), needs_drop::<std::time::Duration>());
    println!("{} {} {}", gnd::<u8>(), gnd::<String>(), gnd::<Option<D>>());
    println!("{} {}", needs_drop::<std::rc::Rc<u8>>(), needs_drop::<[u8; 4]>());
    more();
}
struct Z { a: [String; 0], n: u8 }
struct W<T>(T);
impl<T> Drop for W<T> { fn drop(&mut self) {} }
struct G<T> { t: Option<T> }
enum Fl { A, B }
fn more() {
    println!("{} {} {}", needs_drop::<Z>(), needs_drop::<W<u8>>(), needs_drop::<G<u8>>());
    println!("{} {} {}", needs_drop::<G<String>>(), needs_drop::<Option<[String; 0]>>(), needs_drop::<Fl>());
    println!("{} {} {}", needs_drop::<str>(), needs_drop::<[String]>(), needs_drop::<Box<dyn Fn()>>());
    println!("{} {}", needs_drop::<&mut String>(), needs_drop::<Result<u8, String>>());
}
