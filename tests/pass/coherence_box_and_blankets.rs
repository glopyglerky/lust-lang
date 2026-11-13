// Coherence and selection with blanket impls (ARCH 2.4, 2.6): `Box` is a
// fundamental type, so `Box<My>: Display` cannot hold for a local `My`
// with no Display impl; a blanket whose clause cannot hold for a generic
// local type drops out at selection, through a bound too.
use std::fmt::Display;
trait Tr { fn v(&self) -> u8; }
impl<T: Display> Tr for T { fn v(&self) -> u8 { 1 } }
struct My;
impl Tr for Box<My> { fn v(&self) -> u8 { 2 } }
trait A {}
trait Tr2 { fn w(&self) -> u8; }
impl<T: A> Tr2 for T { fn w(&self) -> u8 { 1 } }
struct Wrap<U>(U);
impl<U> Tr2 for Wrap<U> { fn w(&self) -> u8 { 2 } }
fn via<X: Tr2>(x: &X) -> u8 { x.w() }
fn main() {
    let w: Wrap<u8> = Wrap(1);
    println!("{} {} {} {} {}", Box::new(My).v(), 5u8.v(), Wrap(1u8).w(), <Wrap<u8> as Tr2>::w(&w), via(&w));
}
