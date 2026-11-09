// expect: L0733
// rustc E0308: `F: Fn(&u8) -> &u8` returns its argument's lifetime, which `G: Fn(&u8) -> &'static u8` needs to be `'static`.
fn inner<G: Fn(&u8) -> &'static u8>(g: G) -> u8 {
    let x = 1u8;
    *g(&x)
}

fn outer<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    inner(f)
}

fn main() {}
