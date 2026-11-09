// `'static` bounds on params, implied by `trait Tr: 'static`, and on impls.
trait Tr: 'static { fn tr(&self) -> u32; }
struct A(u32);
impl Tr for A { fn tr(&self) -> u32 { self.0 } }
struct W<X>(X);
impl<X: 'static + Tr> Tr for W<X> { fn tr(&self) -> u32 { self.0.tr() + 10 } }
fn need<X: 'static>(x: X) -> X { x }
fn via<X: Tr>(x: X) -> u32 { need(x).tr() }
fn where_form<X>(x: X) -> X where X: 'static { need(x) }
trait Show { fn show(&self) -> u32 { 5 } }
impl Show for dyn Tr {}
fn main() {
    let c = need(move || 3);
    println!("{} {} {}", via(A(1)), via(W(A(2))), c());
    println!("{}", where_form(A(4)).tr());
    let d: &(dyn Tr + 'static) = &A(6);
    println!("{} {}", d.show(), d.tr());
}
