// expect: L0733
// rustc: `Some(c)` with no expected type is no coercion site, so the
// `&mut dyn P + 'a` inside the Option keeps `'a`; `'x` must then equal it.
trait P {
    fn id(&self) -> u32 {
        1
    }
}
fn f<'a: 'x, 'x>(c: &'a mut (dyn P + 'a), slot: &mut Option<&'x mut (dyn P + 'x)>) {
    let g = c;
    let o = Some(g);
    *slot = o;
}
fn main() {}
