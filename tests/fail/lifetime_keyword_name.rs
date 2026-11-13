// expect: L0001
// rustc: lifetimes cannot use keyword names (a raw `'r#fn` may).
fn f<'fn>(x: &'fn u8) -> &'fn u8 { x }
fn main() { let _ = f(&1); }
