// reject: fragment follow set: ty followed by an operator.
// rustc-error: is followed by `*`, which is not allowed for `ty` fragments
macro_rules! m { ($t:ty *) => { 0 } }
fn main() { let _ = m!(u8 *); }
