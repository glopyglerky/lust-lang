// expect: L0104
// rustc E0283: two impls fit the literal and neither is the i32
// fallback, so the call stays ambiguous.
struct M(u32);

impl From<u8> for M {
    fn from(x: u8) -> M {
        M(x as u32 + 100)
    }
}

impl From<u16> for M {
    fn from(x: u16) -> M {
        M(x as u32 + 200)
    }
}

fn main() {
    println!("{}", M::from(1).0);
}
