// ARCH 2.8: a user impl of another trait for `dyn Tr` dispatches statically.
trait Tr { fn m(&self) -> u32; }
trait Describe { fn describe(&self) -> String; }
impl Describe for dyn Tr { fn describe(&self) -> String { format!("tr({})", self.m()) } }
impl Describe for u8 { fn describe(&self) -> String { format!("u8({})", self) } }
struct A(u32);
impl Tr for A { fn m(&self) -> u32 { self.0 } }
fn show<T: Describe + ?Sized>(x: &T) -> String { x.describe() }
fn main() {
    let a = A(4);
    let d: &dyn Tr = &a;
    println!("{} {} {}", d.describe(), show(d), show(&7u8));
}
