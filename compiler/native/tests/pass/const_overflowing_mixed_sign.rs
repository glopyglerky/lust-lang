// The overflowing mixed-sign integer methods run in the const evaluator:
// the wrapped value and whether the exact result overflowed.
const A: (u8, bool) = 7u8.overflowing_add_signed(-9);
const B: (u32, bool) = 7u32.overflowing_sub_signed(-9);
const C: (i16, bool) = i16::MAX.overflowing_add_unsigned(1);
const D: (i64, bool) = (-5i64).overflowing_sub_unsigned(3);
const fn f(x: u16, y: i16) -> (u16, bool) {
    x.overflowing_add_signed(y)
}
const E: (u16, bool) = f(65535, 1);
fn main() {
    println!("{A:?} {B:?} {C:?} {D:?} {E:?} {:?}", f(3, -4));
}
