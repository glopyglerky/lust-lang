// expect: L0733
// rustc E0308: the output types must be equal, and `&'static u8` is not the higher-ranked `&'r u8` of `G: Fn(&u8) -> &u8`.
fn inner<G: Fn(&u8) -> &u8>(g: G) -> u8 {
    let x = 1u8;
    *g(&x)
}

fn outer<F: Fn(&u8) -> &'static u8>(f: F) -> u8 {
    inner(f)
}

fn main() {}
