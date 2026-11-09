// expect: L0612
// Sol review probe `adt_static_well_formed`; rustc: E0310 the parameter type `T` may not live long enough
struct Hold<T: 'static>(T);
fn bad<T>(x: T) { let _ = Hold(x); }
fn main() { bad(1); }
