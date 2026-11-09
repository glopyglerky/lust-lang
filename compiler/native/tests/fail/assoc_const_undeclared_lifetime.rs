// expect: L0203
// rustc E0261: an inherent assoc const sees only the impl's lifetimes.
struct W<'a>(&'a u8);
impl<'a> W<'a> {
    const C: Option<&'b u8> = None;
}
fn main() {
    let _ = W::C;
}
