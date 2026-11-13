// expect: L0203
// Sol review probe `generic_outlives_forward`; rustc: E0309 the parameter type `T` may not live long enough
fn need<'a, T: 'a>(r: &'a (), _x: T) -> &'a () { r }
fn bad<'a, T>(r: &'a (), x: T) -> &'a () { need(r, x) }
fn main() { let _ = bad(&(), 1); }
