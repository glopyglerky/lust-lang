// reject: a `?` matcher repetition takes no separator (ARCH 6.2). Lust code: L0807.
// rustc-error: macro repetition operator does not take a separator
macro_rules! m { ($($i:ident),?) => { 1 } }
fn main() { let _ = m!(a); }
