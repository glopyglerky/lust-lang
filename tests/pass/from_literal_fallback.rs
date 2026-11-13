// `M::from(1)` with two impls that fit the literal: rustc falls
// back to i32 (and f64 for a float literal), so the i32/f64 impl
// wins. With one impl, the literal takes that impl's type. A
// written trait fixes the input type directly.
struct M(u32);

impl From<u8> for M {
    fn from(x: u8) -> M {
        M(x as u32 + 100)
    }
}

impl From<i32> for M {
    fn from(x: i32) -> M {
        M(x as u32 + 200)
    }
}

impl From<f32> for M {
    fn from(x: f32) -> M {
        M(x as u32 + 10)
    }
}

impl From<f64> for M {
    fn from(x: f64) -> M {
        M(x as u32 + 20)
    }
}

struct N(u32);

impl From<u64> for N {
    fn from(x: u64) -> N {
        N(x as u32 + 7)
    }
}

fn main() {
    println!("{}", M::from(1).0);
    println!("{}", M::from(1.0).0);
    println!("{}", N::from(1).0);
    println!("{}", <M as From<i32>>::from(1).0);
}
