// reject: fragment follow set: `>` may follow a ty, `>=` may not (ARCH 6.2).
// rustc-error: is followed by `>=`, which is not allowed for `ty` fragments
macro_rules! m { ($t:ty >=) => { 0 } }
fn main() {}
