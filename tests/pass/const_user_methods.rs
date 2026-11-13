// Inherent const methods in const contexts: by-value and `&self`
// receivers, autoref and autoderef, UFCS, chains, assoc consts,
// inline const blocks, array lengths, and generic impls.
#[derive(Clone, Copy)]
struct F(u32);
impl F {
    const ONE: F = F(1);
    const fn bits(self) -> u32 {
        self.0
    }
    const fn or(self, o: F) -> F {
        F(self.0 | o.0)
    }
    const fn twice(&self) -> u32 {
        self.bits() * 2 + Self::ONE.bits()
    }
    const D: u32 = F(4).twice();
}
const A: u32 = F(2).or(F::ONE).or(F(8)).bits();
const R: &F = &F(6);
const RR: &&F = &&F(7);
const B: u32 = R.bits() + R.twice() + F::bits(F(1)) + F::twice(&F(2)) + RR.bits();
mod m {
    pub struct W {
        pub a: u32,
        b: u32,
    }
    impl W {
        pub const fn new(a: u32) -> W {
            W { a, b: a * 2 }
        }
        pub const fn sum(&self) -> u32 {
            self.a + self.b
        }
    }
}
const S: m::W = m::W::new(3);
const T: u32 = S.sum();
static U: u32 = m::W::new(5).sum();
#[derive(Clone, Copy)]
struct G<T>(T);
impl<T: Copy> G<T> {
    const fn get(&self) -> T {
        self.0
    }
}
const GX: u32 = G(5u32).get();
const GY: i8 = G(-3i8).get();
fn main() {
    let x = const { F(2).twice() };
    let a = [0u8; F(3).bits() as usize];
    println!("{} {} {} {} {} {}", A, B, F::D, x, a.len(), S.a);
    println!("{} {} {} {}", T, U, GX, GY);
}
