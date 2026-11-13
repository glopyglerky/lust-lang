// ARCH 2.8: a dyn method's returned reference borrows from the receiver; NLL ends the borrow at last use.
trait T { fn get(&self) -> &u8; fn get_mut(&mut self) -> &mut u8; }
struct A(u8);
impl T for A { fn get(&self) -> &u8 { &self.0 } fn get_mut(&mut self) -> &mut u8 { &mut self.0 } }
fn main() { let mut a = A(1); let d: &mut dyn T = &mut a; *d.get_mut() += 5; let r = d.get(); println!("{}", r); *d.get_mut() += 1; println!("{}", a.0); }
