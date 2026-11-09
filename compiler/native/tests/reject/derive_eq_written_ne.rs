// expect: L0112
// Valid Rust, rejected on purpose: the eq glue answers `!=` as
// `!eq`, so a field type with its own `ne` cannot serve a derived
// `PartialEq` yet (as for non-generic written impls).
struct M {
    v: u8,
}
impl PartialEq for M {
    fn eq(&self, o: &M) -> bool {
        self.v == o.v
    }
    fn ne(&self, o: &M) -> bool {
        self.v + 1 != o.v
    }
}
#[derive(PartialEq)]
struct W {
    m: M,
}
fn main() {
    let a = W { m: M { v: 1 } };
    let b = W { m: M { v: 2 } };
    println!("{} {}", a == b, a != b);
}
