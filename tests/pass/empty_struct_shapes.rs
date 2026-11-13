// An empty struct keeps its written shape: `struct U;` is a value and a
// unit pattern, `struct B {}` takes braces only (its name in a pattern is
// a fresh binding), `struct T();` has a constructor fn.
#![allow(non_snake_case, irrefutable_let_patterns)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
struct U;
#[derive(Debug, PartialEq, Default, Clone, Copy)]
struct B {}
#[derive(Debug, PartialEq, Default, Clone, Copy)]
struct T();
fn main() {
    let u = U;
    let b = B {};
    let t = T();
    let mk: fn() -> T = T;
    let ts: Vec<T> = (0..2).map(|_| T()).collect();
    println!("{:?} {:?} {:?} {:?} {}", u, b, t, mk(), ts.len());
    println!("{:?} {:?} {:?}", U {}, T {}, B::default());
    let B {} = b;
    let T() = t;
    let U = u;
    match b {
        B { .. } => println!("brace rest"),
    }
    let B = 7;
    println!("{} {}", B, b == B {});
    println!("{} {} {}", std::mem::size_of::<U>(), std::mem::size_of::<B>(), std::mem::size_of::<T>());
}
