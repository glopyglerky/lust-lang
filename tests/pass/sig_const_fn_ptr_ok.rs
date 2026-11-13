// A static fn pointer from a const fn that names f (sigedit: Sol's const-fn closure).
fn f() -> u32 { 5 }
const fn make() -> fn() -> u32 { f }
static P: fn() -> u32 = make();
fn main() { let p = P; println!("{}", p()); }
