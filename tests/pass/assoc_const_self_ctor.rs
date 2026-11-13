// `Self` in an inherent associated const initializer names the owning
// Adt (trait impl consts too): tuple ctor, variants, and struct literals (the scanner kernel's
// go_flags! consts are `Self(value as u32)`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum E {
    A,
    B(u8),
}
impl E {
    const X: Self = Self::B(3);
    const Y: Self = Self::A;
}
#[derive(Debug)]
struct P {
    x: u8,
}
impl P {
    const Z: Self = Self { x: 9 };
}
trait Tr {
    const C: Self;
}
impl Tr for P {
    const C: Self = Self { x: 1 };
}
#[derive(Clone, Copy, Debug)]
pub struct F(u32);
impl F {
    pub const A: Self = Self(1 as u32);
    pub const B: F = F(2);
}
fn main() {
    println!("{:?} {:?}", F::A, F::B);
    println!("{:?} {:?} {:?} {:?}", E::X, E::Y, P::Z, P::C);
}
