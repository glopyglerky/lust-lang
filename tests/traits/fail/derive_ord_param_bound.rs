// rustc-error: E0599
// lust-expect: L0104
// ARCH 2.2 derives: derived Ord bounds T: Ord, so W<N> has no cmp when
// N is only PartialOrd.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct W<T>(T);
#[derive(PartialEq, Eq, PartialOrd)]
struct N(u8);
fn main() {
    let _ = W(N(1)).cmp(&W(N(2)));
}
