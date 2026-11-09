// ARCH 2.8: `impl dyn Tr` and `impl X for dyn Tr` need a `'static` object
// lifetime. A body-local dyn value can have one, and so can a param written
// `&(dyn Tr + 'static)` or `&'static dyn Tr`.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } fn quad(&self) -> u32 { self.double() * 2 } }
struct Sq { side: u32 }
impl Shape for Sq { fn area(&self) -> u32 { self.side * self.side } }
trait Describe { fn describe(&self) -> u32; }
impl Describe for dyn Shape { fn describe(&self) -> u32 { self.area() + 100 } }
fn show<T: Describe + ?Sized>(x: &T) -> u32 { x.describe() }
fn local(s: &Sq) -> u32 { let d: &dyn Shape = s; d.double() }
fn stat(d: &(dyn Shape + 'static)) -> u32 { d.double() + show(d) }
fn stat_ref(d: &'static dyn Shape) -> u32 { d.quad() }
static S: Sq = Sq { side: 4 };
fn main() {
    let s = Sq { side: 3 };
    let d: &dyn Shape = &s;
    let k: &(dyn Shape + 'static) = d;
    println!("{} {} {} {}", d.double(), d.quad(), d.describe(), show(d));
    println!("{} {} {} {}", local(&s), stat(&s), stat_ref(&S), k.area());
}
