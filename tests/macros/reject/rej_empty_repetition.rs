// reject: a repetition whose body can match empty makes no progress (ARCH 6.2). Lust code: L0807.
// rustc-error: repetition matches empty token tree
macro_rules! m { ($($v:vis)*) => { 1 } }
fn main() { let _ = m!(); }
