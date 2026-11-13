// ARCH 2.8: tuple elements under a tuple expectation are coercion sites,
// also inside `Some(..)`, returns, and arguments.
trait Tr { fn f(&self) -> u32; }
struct A(u32);
struct B(u8);
impl Tr for A { fn f(&self) -> u32 { self.0 } }
impl Tr for B { fn f(&self) -> u32 { self.0 as u32 * 3 } }
fn pair(a: &A) -> (&dyn Tr, u32) { (a, 2) }
fn take(t: (&dyn Tr, &mut dyn Tr)) -> u32 { t.0.f() + t.1.f() }
fn main() {
    let a = A(1);
    let mut b = B(2);
    let t: (&dyn Tr, u32) = (&a, 2);
    let n: ((&dyn Tr, u8), u8) = ((&b, 1), 2);
    let o: Option<(&dyn Tr, u32)> = Some((&a, 5));
    println!("{} {} {}", t.0.f() + t.1, (n.0).0.f() + n.1 as u32, pair(&a).0.f());
    if let Some((d, k)) = o { println!("{} {}", d.f(), k); }
    println!("{}", take((&a, &mut b)));
}
