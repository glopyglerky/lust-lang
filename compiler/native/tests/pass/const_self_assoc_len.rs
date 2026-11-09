// `Self::N` in an anonymous const where `Self` has no generic parameter:
// a struct or enum field length, an inherent impl fn signature, an impl of
// one instance of a generic type, and an associated const whose value reads
// another through `Self` (ts-rust inference_p1.rs DefinitelyUnrelatedCache).
struct Cache {
    slots: Box<[(u64, u64); Self::SLOTS]>,
}
impl Cache {
    const SLOTS: usize = 16;
    const HALF: usize = Self::SLOTS / 2;
    fn halves(&self) -> [u8; Self::HALF] {
        [1; Self::HALF]
    }
}
enum E {
    A([u16; Self::N]),
    B,
}
impl E {
    const N: usize = 2;
}
struct G<T>(T);
impl G<u8> {
    const K: usize = 3;
    fn k(&self) -> [u8; Self::K] {
        [self.0; Self::K]
    }
}
fn main() {
    let c = Cache { slots: Box::new([(0, 0); Cache::SLOTS]) };
    println!("{} {}", c.slots.len(), c.halves().len());
    if let E::A(x) = E::A([1, 2]) {
        println!("{}", x.len());
    }
    let _ = E::B;
    println!("{:?}", G(7u8).k());
}
