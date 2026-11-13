// expect: L0705
// rustc: lifetime may not live long enough. `gn`'s bound `Q: X` picks `impl X for dyn Lg`, whose
// object lifetime is `'static`, but `v` holds a `B<'_>` that borrows the parameter.
trait Lg { fn lg(&self) -> u32; }
struct B<'b>(&'b u32);
impl<'b> Lg for B<'b> { fn lg(&self) -> u32 { *self.0 + 10 } }
trait X { fn x(&self) -> u32; }
impl X for dyn Lg { fn x(&self) -> u32 { self.lg() + 2 } }
fn gn<Q: X + ?Sized>(q: &Q) -> u32 { q.x() + 100 }
fn run(n: &u32) -> u32 {
    let v: Box<dyn Lg> = Box::new(B(n));
    gn(&*v)
}
fn main() { let n = 7u32; println!("{}", run(&n)); }
