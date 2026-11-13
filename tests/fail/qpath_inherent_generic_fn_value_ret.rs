// expect: L0110
// rustc E0308: <W<u16>>::get returns u16.
struct W<T>(T);
impl<T: Copy> W<T> { fn get(&self) -> T { self.0 } }
fn main() { let f = <W<u16>>::get; let x: u8 = f(&W(3)); println!("{}", x); }
