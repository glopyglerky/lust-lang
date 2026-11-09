// A field type whose `PartialEq` is a written generic impl with a
// generic right-hand side (`impl<S, S2> PartialEq<M<S2>> for M<S>`,
// indexmap's shape): deriving `PartialEq` over it calls that impl
// at `S2 = S`, including where its `eq` is not the structural one
// (W-BENCH2 bug 3). The same for a generic wrapping struct.
struct M<S> {
    v: Vec<u8>,
    s: S,
}
impl<S, S2> PartialEq<M<S2>> for M<S> {
    fn eq(&self, o: &M<S2>) -> bool {
        self.v == o.v
    }
}
#[derive(PartialEq)]
struct W {
    m: M<u8>,
    k: u32,
}
#[derive(PartialEq)]
struct G<T> {
    m: M<T>,
}
fn main() {
    let a = W { m: M { v: vec![1], s: 0 }, k: 1 };
    let b = W { m: M { v: vec![1], s: 9 }, k: 1 };
    let c = W { m: M { v: vec![2], s: 0 }, k: 1 };
    let g1 = G { m: M { v: vec![1], s: "a".to_string() } };
    let g2 = G { m: M { v: vec![1], s: "b".to_string() } };
    // The cross-type comparison itself, both ways.
    let x = M { v: vec![1], s: 0u8 };
    let y = M { v: vec![2], s: 0u16 };
    println!("{} {} {} {} {}", a == b, a == c, a != c, g1 == g2, x != y);
}
