// A generic body calls the trait method its bound names, also when the
// concrete type has an inherent method with the same name (rustc resolves
// methods once per generic body; instances reuse the template's pick).
struct S(u8);
impl S {
    fn get(self) -> u8 { self.0 + 100 }
    fn describe(&self) -> u8 { 99 }
    fn helper(&self) -> u8 { 50 }
    fn bump(&mut self) -> u8 { self.0 += 100; self.0 }
}
trait G { fn get(&self) -> u8; }
impl G for S { fn get(&self) -> u8 { self.0 } }
trait D {
    fn helper(&self) -> u8;
    fn describe(&self) -> u8 { 1 }
    fn run(&self) -> u8 { self.helper() + self.describe() }
}
impl D for S { fn helper(&self) -> u8 { 5 } }
trait Bump { fn bump(&mut self) -> u8; }
impl Bump for S { fn bump(&mut self) -> u8 { self.0 += 1; self.0 } }
trait Twice { fn twice(&self) -> u8; }
impl<T: G> Twice for T { fn twice(&self) -> u8 { self.get() * 2 } }
struct W<T>(T);
impl<T: G> W<T> { fn inner(&self) -> u8 { self.0.get() } }
fn by_value<T: G>(t: T) -> u8 { t.get() }
fn by_ref<T: G>(t: &T) -> u8 { t.get() }
fn by_mut<T: Bump>(t: &mut T) -> u8 { t.bump(); t.bump() }
fn default_body<T: D>(t: &T) -> u8 { t.describe() }
fn in_closure<T: G>(xs: &[T]) -> u8 { let f = |x: &T| x.get(); f(&xs[0]) }
fn twice_ref<U: G>(u: &U) -> u8 { by_ref(u) * 2 }
fn nested<T: G>(t: &T) -> u8 { by_ref(t) + twice_ref(t) }
fn main() {
    println!("{} {} {}", by_value(S(5)), by_ref(&S(6)), S(7).get());
    let mut s = S(0);
    println!("{} {}", by_mut(&mut s), s.0);
    println!("{} {} {}", default_body(&S(0)), S(0).describe(), S(0).run());
    println!("{} {} {}", S(4).twice(), W(S(9)).inner(), in_closure(&[S(3)]));
    println!("{}", nested(&S(8)));
}
