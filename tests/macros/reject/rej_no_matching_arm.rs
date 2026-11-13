// reject: the call matches no macro arm.
// rustc-error: no rules expected `no`
macro_rules! m { (yes) => { 1 } }
fn main() { let _ = m!(no); }
