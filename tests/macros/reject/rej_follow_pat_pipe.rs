// reject: fragment follow set: edition 2024 `pat` eats a top-level `|`, so `|` may not follow it (ARCH 6.2).
// rustc-error: is followed by `|`, which is not allowed for `pat` fragments
macro_rules! m { ($p:pat |) => { 0 } }
fn main() {}
