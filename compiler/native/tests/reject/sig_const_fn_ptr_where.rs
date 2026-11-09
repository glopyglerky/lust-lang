// expect: L0630
// Valid Rust; lust's const evaluator refuses a fn with a where clause as a fn pointer value, reached through make() (sigedit: Sol's const-fn closure).
fn f() -> u32 where u32: Copy { 5 }
const fn make() -> fn() -> u32 { f }
static P: fn() -> u32 = make();
fn main() { let p = P; println!("{}", p()); }
