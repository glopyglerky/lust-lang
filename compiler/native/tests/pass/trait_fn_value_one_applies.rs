// Two traits name `f`, but only one applies to `S` (the other is for
// `&S`, another type, or out of scope), so `S::f` is that trait's fn.
mod m {
    pub trait Tr3 { fn f(&self) -> i32 { 3 } }
    impl Tr3 for super::S {}
}
trait Tr { fn f(&self) -> i32 { println!("Tr"); 1 } }
trait Tr2 { fn f(&self) -> i32 { 2 } }
struct S(i32);
struct O;
impl Tr for S {}
impl Tr2 for &S {}
impl Tr2 for O {}
fn ap<F: Fn(&S) -> i32>(f: F) -> i32 { f(&S(9)) }
fn main() {
    let g = S::f;
    println!("{}", g(&S(3)));
    println!("{}", S::f(&S(4)));
    println!("{}", ap(S::f));
    println!("{}", <&S as Tr2>::f(&&S(5)));
}
