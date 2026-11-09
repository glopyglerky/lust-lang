#[derive(Debug)]
pub struct Unit;
#[derive(Debug)]
pub enum E {
    A,
    B(u8),
}
const R: &u16 = &5;
const K: (u8, u8) = (6, 7);

pub fn nested() -> &'static &'static u8 {
    &&8
}
pub fn zst() -> (&'static (), &'static Unit, &'static [u32]) {
    (&(), &Unit, &[])
}
pub fn konst() -> (&'static u16, &'static (u8, u8), &'static E, u8) {
    (R, &K, &E::B(3), crate::b::three())
}
pub fn unit_variant() -> [&'static E; 2] {
    [&E::A, &E::B(1)]
}
