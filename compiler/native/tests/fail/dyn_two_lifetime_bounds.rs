// expect: L0110
// rustc E0226: only a single explicit lifetime bound is permitted.
trait T { fn m(&self) -> u8; }
fn f(d: &(dyn T + 'static + 'static)) -> u8 { d.m() }
fn main() {}
