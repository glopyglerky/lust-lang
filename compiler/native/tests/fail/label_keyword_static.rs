// expect: L0001
// rustc: labels cannot use keyword names; `'static` and `'_` are lifetimes only.
fn main() { 'static: loop { break; } }
