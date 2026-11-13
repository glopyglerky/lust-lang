// ARCH 2.8: `Tr::m(d, ..)` with `d: &dyn Tr` (Self inferred from the argument) calls through the
// vtable, for side-effect calls, `&mut self` methods, values, and supertrait methods.
trait Base { fn base(&self) -> u8; }
trait Tr: Base { fn note(&self); fn set(&mut self, x: u8); fn get(&self) -> u8; }
struct S(u8);
impl Base for S { fn base(&self) -> u8 { 40 + self.0 } }
impl Tr for S {
    fn note(&self) { println!("note {}", self.0); }
    fn set(&mut self, x: u8) { self.0 = x; }
    fn get(&self) -> u8 { self.0 }
}
fn id(x: u8) -> u8 { x }
fn main() {
    let mut s = S(1);
    {
        let d: &mut dyn Tr = &mut s;
        Tr::set(d, 7);
        Tr::note(d);
    }
    let d: &dyn Tr = &s;
    Tr::note(d);
    let v = Tr::get(d);
    println!("{} {} {}", v, id(Tr::get(d)), Base::base(d));
}
