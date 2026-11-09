// expect: L0733
// rustc: lifetime may not live long enough. The `&mut` reborrow arm used to skip the object lifetime.
trait T {}
fn bad(d: &mut dyn T) -> &mut (dyn T + 'static) { d }
fn main() {}
