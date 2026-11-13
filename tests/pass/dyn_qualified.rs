// ARCH 2.8: `<dyn Tr as Tr>::m(d)`, `<dyn Tr>::m(d)`, supertrait and `&mut dyn` qualified calls.
trait Base { fn b(&self) -> u32 { 100 } }
trait Tr: Base { fn m(&self) -> u32; fn n(&self, k: u32) -> u32 { self.m() + k } fn bump(&mut self); }
struct A(u32);
impl Base for A {}
impl Tr for A { fn m(&self) -> u32 { self.0 } fn bump(&mut self) { self.0 += 1; } }
impl dyn Tr { fn hello(&self) -> u32 { self.m() * 10 } }
fn main() {
    let mut a = A(3);
    let d: &dyn Tr = &a;
    println!("{} {} {} {}", <dyn Tr as Tr>::m(d), <dyn Tr as Tr>::n(d, 4), <dyn Tr>::hello(d), <dyn Tr as Base>::b(d));
    println!("{}", <dyn Tr>::m(d));
    let e: &mut dyn Tr = &mut a;
    <dyn Tr as Tr>::bump(e);
    <dyn Tr>::bump(e);
    println!("{}", a.0);
}
