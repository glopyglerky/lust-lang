// expect: L0110
// Closure int type fixed by a later use overflows i32; rustc E0271-class reject. Pin (cl17).
fn main() { let c = |x| x + 1; let r: u64 = c(4294967295); println!("{}", r); }
