// expect: L0303
// ARCH 2.8, rustc E0502: `r` borrows from `*d` through a dyn call, so `d` cannot be borrowed mutably.
trait T { fn get(&self) -> &u8; fn get_mut(&mut self) -> &mut u8; }
struct A(u8);
impl T for A { fn get(&self) -> &u8 { &self.0 } fn get_mut(&mut self) -> &mut u8 { &mut self.0 } }
fn main() { let mut a = A(1); let d: &mut dyn T = &mut a; let r = d.get(); *d.get_mut() += 5; println!("{}", r); }
