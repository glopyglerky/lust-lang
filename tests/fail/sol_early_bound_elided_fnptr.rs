// expect: L0110
// Sol review probe `early_bound_elided_fnptr`; rustc: E0308 mismatched types
fn early<'a: 'a>(x: &'a i32) -> &'a i32 { x }
fn main() { let _: fn(&i32) -> &i32 = early; }
