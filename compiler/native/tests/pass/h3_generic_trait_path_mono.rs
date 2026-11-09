// A trait path call through a type parameter's bound (`T::clone(t)`,
// `<T as Clone>::clone(t)`) and a blanket impl over `Fn() -> i32` reached
// from a generic fn: both build at builtin types (i32, fn pointers, closures).
trait Tag { fn tag(&self) -> &'static str; }
impl<T: Fn() -> i32> Tag for T { fn tag(&self) -> &'static str { "fn" } }
fn gv<U: Tag>(u: U) -> &'static str { u.tag() }
fn f0() -> i32 { 42 }
fn g<T: Clone>(t: &T) -> T { T::clone(t) }
fn q<T: Clone>(t: &T) -> T { <T as Clone>::clone(t) }
#[derive(Clone, Debug)]
struct S(i32);
fn main() {
    let x: fn() -> i32 = f0;
    println!("{} {} {} {}", gv(x), gv(f0), gv(|| 3), x.tag());
    println!("{} {:?} {} {:?}", g(&5i32), g(&S(1)), q(&String::from("a")), q(&S(2)));
}
