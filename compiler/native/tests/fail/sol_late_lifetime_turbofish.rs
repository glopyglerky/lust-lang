// expect: L0731
// Sol review probe `late_lifetime_turbofish`; rustc: E0794 cannot specify lifetime arguments explicitly if late bound lifetime parameters are present
fn id<'a>(x: &'a i32) -> &'a i32 { x }
fn main() { let _ = id::<'static>(&1); }
