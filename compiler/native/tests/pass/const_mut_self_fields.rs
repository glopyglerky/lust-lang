// Const fns with a `mut self` receiver that write into array fields (ts-rust
// KindSet::with), named-field index writes, and their use in statics.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
enum Kind { A, B, C = 70, D = 200 }
#[derive(Clone, Copy)]
struct KindSet([u64; 8]);
impl KindSet {
    const fn of(kinds: &[Kind]) -> Self {
        Self::EMPTY.with(kinds)
    }
    const EMPTY: Self = Self([0; 8]);
    const fn with(mut self, kinds: &[Kind]) -> Self {
        let mut i = 0;
        while i < kinds.len() {
            let k = kinds[i] as usize;
            self.0[k >> 6] |= 1 << (k & 63);
            i += 1;
        }
        self
    }
    const fn contains(&self, kind: Kind) -> bool {
        let k = kind as usize;
        self.0[(k >> 6) & 7] & (1 << (k & 63)) != 0
    }
}
const LIST: &[Kind] = &[Kind::A, Kind::C];
static SET: KindSet = KindSet::of(LIST);
static SET2: KindSet = KindSet::of(&[Kind::B, Kind::D]).with(LIST);
fn main() {
    tail();
    println!("{} {} {} {}", SET.contains(Kind::A), SET.contains(Kind::B), SET.contains(Kind::C), SET.contains(Kind::D));
    println!("{} {} {}", SET2.contains(Kind::A), SET2.contains(Kind::B), SET2.contains(Kind::D));
}
struct N {
    a: [u8; 3],
    b: u8,
}
const fn bump(mut n: N) -> N {
    n.a[2] += 5;
    n.b <<= 2;
    n
}
const NB: N = bump(N { a: [1; 3], b: 3 });
const fn self_assign(mut k: KindSet) -> KindSet {
    k = KindSet::EMPTY;
    k.0[7] = 9;
    k
}
static KS: KindSet = self_assign(KindSet([1; 8]));
fn tail() {
    println!("{:?} {} {:?}", NB.a, NB.b, KS.0);
}
