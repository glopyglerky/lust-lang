// ARCH 2.8, rustc: a method bounded by `Self: Sized` (directly or through Clone or Copy) is not in the
// vtable, so the trait stays dyn compatible; sized types still call it.
trait Tr {
    fn get(&self) -> u8;
    fn dup(&self) -> Self where Self: Sized;
    fn pair<X>(&self, x: X) -> (u8, X) where Self: Sized { (self.get(), x) }
    fn cl(&self) -> Self where Self: Clone { self.clone() }
    fn by_val(self) -> u8 where Self: Sized { 9 }
    fn after(&self) -> u8 { self.get() + 1 }
}
#[derive(Clone)]
struct S(u8);
impl Tr for S { fn get(&self) -> u8 { self.0 } fn dup(&self) -> Self { S(self.0 + 10) } }
fn main() {
    let s = S(3);
    let d: &dyn Tr = &s;
    println!("{} {}", d.get(), d.after());
    let t = s.dup();
    println!("{} {:?} {}", t.0, s.pair('x'), s.cl().0);
    println!("{}", t.by_val());
}
