// A shift amount's type never pins the shifted value's type in a
// const (W-BENCH3 bug 11): `4 << Self::SHIFT` with `SHIFT: u32`
// inside a u64 newtype computes in u64, as rustc infers it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id(pub u64);
impl Id {
    const KIND_SHIFT: u32 = 61;
    pub const KIND_LOCALS: Self = Self(0 << Self::KIND_SHIFT);
    pub const KIND_MASK: Self = Self(4 << Self::KIND_SHIFT);
}
const S: u32 = 61;
const A: u64 = 4 << S;
const B: u64 = 4 << 61;
const C: i64 = -16 >> S;
fn main() {
    println!("{:?} {:?} {} {} {}", Id::KIND_LOCALS, Id::KIND_MASK, A, B, C);
    let v: u64 = 4 << S;
    println!("{}", v);
}
