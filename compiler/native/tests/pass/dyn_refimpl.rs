// ARCH 2.8: an impl for `&u8`: `&&u8` unsizes with `&u8` as the concrete type.
trait Tr { fn m(&self) -> u8; }
impl Tr for u8 { fn m(&self) -> u8 { *self } }
impl Tr for &u8 { fn m(&self) -> u8 { **self + 1 } }
fn main() {
    let x = 5u8;
    let r = &x;
    let d: &dyn Tr = &r;
    let e: &dyn Tr = r;
    println!("{} {}", d.m(), e.m());
}
