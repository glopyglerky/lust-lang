// Deny-by-default lints lifted at the crate, module, and impl levels, by
// `allow`, `warn`, and `expect`; the value path `Self::V` is the variant.
#![warn(arithmetic_overflow)]
#![expect(ambiguous_associated_items)]
#![allow(unused)]

mod lits {
    #![allow(overflowing_literals)]
    pub fn wrapped() -> u8 {
        let x: u8 = 256;
        x
    }
}

#[allow(bindings_with_variant_name)]
mod pats {
    pub enum E {
        A,
        B,
    }
    pub fn pick(e: E) -> u32 {
        match e {
            A => 7,
        }
    }
}

enum K {
    V,
    W,
}
trait Tr {
    type V;
    fn f(&self) -> Self::V;
    fn g(&self) -> u8;
}
impl Tr for K {
    type V = u8;
    fn f(&self) -> Self::V {
        3
    }
    fn g(&self) -> u8 {
        match self {
            Self::V => 1,
            Self::W => 2,
        }
    }
}

fn main() {
    println!("{}", lits::wrapped());
    println!("{}", pats::pick(pats::E::B));
    println!("{} {} {}", K::W.f(), K::V.g(), K::W.g());
}
