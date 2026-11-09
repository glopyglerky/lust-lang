// Outer attributes where rustc takes them on expressions: call and method
// arguments, tuple and array elements, vec! elements, struct literal fields,
// closure parameters, statements, and a block tail (also in parentheses).
// A cfg that is off removes the element; each value prints.
#![allow(unused)]
struct S { a: i32, b: i32 }
struct M;
impl M { fn m(&self, x: i32, y: i32) -> i32 { x * 10 + y } }
fn h(x: i32, y: i32) -> i32 { x * 10 + y }
fn g(x: i32) -> i32 { x }
fn main() {
    let x = 3;
    println!("{}", h(#[allow(unused)] 1, #[cfg(all())] 2));
    println!("{}", h(1, #[cfg(any())] 7, 2));
    println!("{}", M.m(#[allow(unused_parens)] (x + 1), 5));
    println!("{}", g(#[allow(unused)] -x));
    println!("{:?}", (#[allow(unused)] 1, #[cfg(any())] 2, 3));
    println!("{:?}", (#[cfg(any())] 1, 2));
    println!("{:?}", (#[cfg(all())] x.pow(2),));
    println!("{:?}", [#[allow(unused)] 1, #[cfg(any())] 2, 3]);
    println!("{:?}", vec![#[cfg(all())] 4, #[cfg(any())] 5, #[allow(unused)] { 6 }]);
    let s = S { #[allow(unused)] a: 1, #[cfg(all())] b: 2 };
    println!("{}", s.a + s.b);
    let f = |#[allow(unused)] p: i32, #[cfg(all())] q: i32| p - q;
    println!("{}", f(9, 4));
    let k = [1, 2].iter().fold(0, |acc, #[cfg(all())] v| acc + v);
    println!("{k}");
    let t = { #[allow(unused_parens)] (x - 1) };
    println!("{t}");
    let u = { #[cfg(all())] g(x) };
    println!("{u}");
    #[allow(unused)] g(1);
    /// a doc comment on a call argument only warns
    println!("{}", g(
        /// doc
        8,
    ));
}
