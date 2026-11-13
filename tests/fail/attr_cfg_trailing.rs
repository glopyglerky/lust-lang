// expect: L0001
// rustc: expected `]`, found `x`. The expander must not act on this cfg.
#[cfg(all()) x]
fn main() {}
