// impl Trait callers: generic definers, a trait default method, a hidden type that is another fn's opaque, Vec and Option of opaque values, impl FnMut and impl Fn closures (W-CHECK-G3).
use std::fmt::Display;
struct W<T>(T);
trait Shape { fn area(&self) -> u32; fn name(&self) -> String { format!("shape{}", self.area()) } }
impl<T: Display> Shape for W<T> { fn area(&self) -> u32 { format!("{}", self.0).len() as u32 } }
fn wrap<T: Display>(t: T) -> impl Shape { W(t) }
fn inner() -> impl Shape { wrap(12345u32) }
fn outer() -> impl Shape { inner() }
fn sum_all<S: Shape>(xs: &[S]) -> u32 { let mut s = 0; for x in xs { s += x.area(); } s }
fn counter(start: u32) -> impl FnMut() -> String { let mut c = String::from("x"); let _ = start; move || { c.push('y'); c.clone() } }
fn adder(k: String) -> impl Fn(i64) -> String { move |x| format!("{k}{x}") }
fn main() {
    let a = wrap("hello");
    let b = wrap(3.5f64);
    println!("{} {} {}", a.area(), b.area(), a.name());
    println!("{} {}", outer().area(), outer().name());
    let v = vec![wrap(1u8), wrap(22u8)];
    println!("{}", sum_all(&v));
    let mut c = counter(5);
    println!("{} {}", c(), c());
    let f = adder(String::from("k"));
    println!("{}", f(5));
    let o: Option<_> = Some(inner());
    if let Some(s) = &o { println!("{}", s.area()); }
}
