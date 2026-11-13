// expect: L0110
// rustc E0308: the closure returns `&str`, the `dyn Fn` an `i32`.
fn main() { let f: &dyn Fn(i32) -> i32 = &|_n| "x"; println!("{}", f(1)); }
