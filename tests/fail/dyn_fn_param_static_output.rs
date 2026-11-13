// expect: L0733
// rustc E0308: `F` returns `&'static u8`, but `dyn Fn(&u8) -> &u8` returns its argument's lifetime.
fn coerce<F: Fn(&u8) -> &'static u8>(g: &F) -> u8 {
    let d: &dyn Fn(&u8) -> &u8 = g;
    let l = 1u8;
    *d(&l)
}

fn main() {}
