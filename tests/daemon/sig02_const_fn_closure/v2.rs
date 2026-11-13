fn f() -> u32 where u32: Copy { 5 }
const fn make() -> fn() -> u32 { f }
static P: fn() -> u32 = make();
fn main() { let p = P; println!("{}", p()); }
