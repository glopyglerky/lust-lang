// expect: L0720
// Two traits may apply to the literal and neither impl is known yet
// (rustc E0034).
trait A { fn m(self) -> u32; }
trait B { fn m(self) -> u32; }
impl A for u8 { fn m(self) -> u32 { 1 } }
impl A for u16 { fn m(self) -> u32 { 2 } }
impl B for i8 { fn m(self) -> u32 { 3 } }
impl B for i16 { fn m(self) -> u32 { 4 } }
fn main() { let x = 5; println!("{}", x.m()); }
