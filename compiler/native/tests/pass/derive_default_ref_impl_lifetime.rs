// A written `impl Default for &'static E` serves a `&'a E` field of a
// derived Default: the `&'static E` it returns coerces to `&'a E`.
pub struct E(u8);
static ONE: E = E(1);
impl Default for &'static E {
    fn default() -> Self { &ONE }
}
#[derive(Default)]
struct S<'a> {
    e: &'a E,
}
fn main() {
    let s = S::default();
    println!("{}", s.e.0);
}
