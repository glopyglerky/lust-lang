// expect: L0507
// rustc E0659: a macro_rules! that a macro expansion defines may not
// shadow the std macro `matches` at a call outside that expansion.
macro_rules! defm { () => { macro_rules! matches { ($e:expr, $p:pat) => { true }; } }; }
defm!();
fn main() { let x = Some(1); println!("{} {}", matches!(x, Some(9)), 1); }
