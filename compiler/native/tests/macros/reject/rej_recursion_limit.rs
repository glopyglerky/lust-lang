// reject: expansion cap: self-recursive macro hits the recursion limit (ARCH 6.1 caps). Lust code: L0801.
// rustc-error: recursion limit reached while expanding
macro_rules! m { () => { m!() } }
fn main() { m!(); }
