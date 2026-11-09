// Macro-expanded macro_rules! on one warm daemon: a generated macro with
// no outer binding, one that shadows std `matches` (E0659), a call inside
// the generating expansion, one that shadows an earlier written macro
// (E0659), and the generator removed again.
fn main() { let x = Some(1); println!("{}", matches!(x, Some(9))); }
