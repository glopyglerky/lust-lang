// expect: L0303
// rustc E0502: the `&mut` autoref of an `FnMut` callee is not two-phase.
fn peek(_: &dyn FnMut(usize) -> usize) -> usize { 1 }
fn g(cb: &mut dyn FnMut(usize) -> usize) -> usize { cb(peek(&*cb)) }
fn main() { let mut n = 0; println!("{}", g(&mut |x| { n += x; n })); }
