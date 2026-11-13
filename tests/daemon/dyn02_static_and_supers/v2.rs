// Dyn edits on one warm daemon: a `where Self: 'static` method bound, a
// trait-level `'static` bound, and the order of two generic supertrait refs
// (`B<u8> + B<u16>`), which moves the super slot a qualified call uses.
trait B<X> {
    fn b(&self, x: X) -> X;
}
trait T: B<u8> + B<u16> {
    fn h(&self) -> u8
    where
        Self: 'static,
    {
        8
    }
}
fn call(d: &dyn T) -> u8 {
    d.h()
}
struct A(u8);
impl B<u8> for A {
    fn b(&self, x: u8) -> u8 {
        x + self.0
    }
}
impl B<u16> for A {
    fn b(&self, x: u16) -> u16 {
        x * 2
    }
}
impl T for A {}
fn main() {
    let a = A(3);
    println!("{} {} {}", call(&a), <dyn T as B<u16>>::b(&a, 5), <dyn T as B<u8>>::b(&a, 5));
}
