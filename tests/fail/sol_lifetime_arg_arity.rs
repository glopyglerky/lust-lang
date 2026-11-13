// expect: L0110
// Sol review probe `lifetime_arg_arity`; rustc: E0107 struct takes 1 lifetime argument but 2 lifetime arguments were supplied
struct Hold<'a>(&'a i32);
fn bad<'a>(x: &'a i32) -> Hold<'a, 'a> { Hold(x) }
fn main() {}
