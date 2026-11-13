// ARCH 2.8: `&X` for a static `X` coerces to `&(dyn Tr + 'static)`. Mono checks bodies before
// statics resolve, so only the eager vtable exists; the lookup keys on the principal alone.
trait Tr { fn get(&self) -> u8; }
impl Tr for u8 { fn get(&self) -> u8 { *self } }
struct S { v: u8 }
impl Tr for S { fn get(&self) -> u8 { self.v + 1 } }
static X: u8 = 5;
static Y: S = S { v: 7 };
fn keep(d: &(dyn Tr + 'static)) -> u8 { d.get() }
fn main() {
    let d: &(dyn Tr + 'static) = &Y;
    println!("{} {} {}", keep(&X), keep(&Y), d.get());
}
