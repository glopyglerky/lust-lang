// A Debug impl that panics inside a Vec, Option, struct, or map builder:
// unwinding leaves the builder at once, so println! writes the pieces
// before the panic and no closing `]`, `)`, or `}` (tf-panicloc).
use std::collections::BTreeMap;
struct P(usize);
impl std::fmt::Debug for P {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", [1u8][self.0])
    }
}
#[derive(Debug)]
struct W {
    a: P,
    b: P,
}
fn main() {
    let n = std::env::args().count();
    let r = std::panic::catch_unwind(|| println!("{:?}", vec![P(0), P(n)]));
    println!("\n{}", r.is_err());
    let r = std::panic::catch_unwind(|| println!("{:?}", Some(P(n))));
    println!("\n{}", r.is_err());
    let r = std::panic::catch_unwind(|| println!("{:#?}", W { a: P(0), b: P(n) }));
    println!("\n{}", r.is_err());
    let mut m = BTreeMap::new();
    m.insert(1, P(n));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| println!("{:?}", m)));
    println!("\n{}", r.is_err());
}
