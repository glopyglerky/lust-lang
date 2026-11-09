// Patterns that match through a reference bind by reference in const
// evaluation (rustc's default binding mode), as `&self` methods need.
enum E {
    A(u32),
    B,
}
impl E {
    const fn v(&self) -> u32 {
        match self {
            E::A(x) => *x,
            E::B => 0,
        }
    }
    const fn is_a(&self) -> bool {
        matches!(self, E::A(_))
    }
}
struct P(u32, u32);
const fn sum(p: &P) -> u32 {
    let P(a, b) = p;
    *a + *b
}
const X: u32 = E::A(9).v() + E::B.v();
const Y: bool = E::B.is_a();
const Z: u32 = sum(&P(1, 2));
const T: u32 = {
    let (a, b) = &(1u32, 2u32);
    *a * 10 + *b
};
const O: u32 = match &Some(4u32) {
    Some(v) => *v,
    None => 0,
};
fn main() {
    println!("{} {} {} {} {}", X, Y, Z, T, O);
}
